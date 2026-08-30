use anyhow::{Context, Result};
use opentelemetry::global;
use opentelemetry::logs::Severity;
use opentelemetry::metrics::{Counter, Gauge, Histogram, UpDownCounter};
use opentelemetry::propagation::{Extractor, Injector};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{KeyValue, metrics::MeterProvider as _};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::logs::{LogBatch, LogExporter};
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::metrics::exporter::PushMetricExporter;
use opentelemetry_sdk::metrics::{SdkMeterProvider, Temporality};
use opentelemetry_sdk::trace::{SdkTracerProvider, SpanData, SpanExporter};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;

pub struct TelemetryGuard {
    tracer: Option<SdkTracerProvider>,
    meter: Option<SdkMeterProvider>,
    logger: Option<SdkLoggerProvider>,
}

pub struct RelayMetrics {
    accepted: Counter<u64>,
    rejected: Counter<u64>,
    attempts: Counter<u64>,
    duration_ms: Histogram<f64>,
    queue_delay_ms: Histogram<f64>,
    queue_depth: Gauge<u64>,
    in_flight: UpDownCounter<i64>,
    retries: Counter<u64>,
    delivered: Counter<u64>,
    expired: Counter<u64>,
    recovery: Counter<u64>,
    cleanup: Counter<u64>,
}

static METRICS: OnceLock<RelayMetrics> = OnceLock::new();
static EXPORT_STATUS: OnceLock<Mutex<ExportStatus>> = OnceLock::new();

#[derive(Clone, Default)]
pub struct ExportStatus {
    pub last_successful_export_at: Option<SystemTime>,
    pub last_error: Option<String>,
    last_error_log_at: Option<SystemTime>,
}

fn record_export(signal: &'static str, result: &OTelSdkResult) {
    let state = EXPORT_STATUS.get_or_init(|| Mutex::new(ExportStatus::default()));
    let mut state = state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    match result {
        Ok(()) => state.last_successful_export_at = Some(SystemTime::now()),
        Err(_) => {
            state.last_error = Some(format!("{signal} export failed"));
            let now = SystemTime::now();
            let should_log = state.last_error_log_at.is_none_or(|last| {
                now.duration_since(last).unwrap_or_default() >= Duration::from_secs(60)
            });
            if should_log {
                state.last_error_log_at = Some(now);
                tracing::warn!(signal, "OpenTelemetry export failed");
            }
        }
    }
}

pub fn export_status() -> ExportStatus {
    EXPORT_STATUS
        .get()
        .map(|state| {
            state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone()
        })
        .unwrap_or_default()
}

#[derive(Debug)]
struct TrackingSpanExporter(opentelemetry_otlp::SpanExporter);

impl SpanExporter for TrackingSpanExporter {
    async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult {
        let result = self.0.export(batch).await;
        record_export("traces", &result);
        result
    }

    fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult {
        self.0.shutdown_with_timeout(timeout)
    }

    fn force_flush(&self) -> OTelSdkResult {
        self.0.force_flush()
    }

    fn set_resource(&mut self, resource: &Resource) {
        self.0.set_resource(resource);
    }
}

struct TrackingMetricExporter(opentelemetry_otlp::MetricExporter);

impl PushMetricExporter for TrackingMetricExporter {
    async fn export(&self, metrics: &ResourceMetrics) -> OTelSdkResult {
        let result = self.0.export(metrics).await;
        record_export("metrics", &result);
        result
    }

    fn force_flush(&self) -> OTelSdkResult {
        self.0.force_flush()
    }

    fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult {
        self.0.shutdown_with_timeout(timeout)
    }

    fn temporality(&self) -> Temporality {
        self.0.temporality()
    }
}

#[derive(Debug)]
struct TrackingLogExporter(opentelemetry_otlp::LogExporter);

impl LogExporter for TrackingLogExporter {
    async fn export(&self, batch: LogBatch<'_>) -> OTelSdkResult {
        let result = self.0.export(batch).await;
        record_export("logs", &result);
        result
    }

    fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult {
        self.0.shutdown_with_timeout(timeout)
    }

    fn event_enabled(&self, level: Severity, target: &str, name: Option<&str>) -> bool {
        self.0.event_enabled(level, target, name)
    }

    fn set_resource(&mut self, resource: &Resource) {
        self.0.set_resource(resource);
    }
}

pub fn init(config: &Config) -> Result<TelemetryGuard> {
    let fmt_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "promptjang_relay_one=info,tower_http=info".into());
    let fmt_layer = tracing_subscriber::fmt::layer().with_filter(fmt_filter);
    if !config.otel_enabled {
        tracing_subscriber::registry().with(fmt_layer).init();
        return Ok(TelemetryGuard {
            tracer: None,
            meter: None,
            logger: None,
        });
    }

    std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .context("OTEL_EXPORTER_OTLP_ENDPOINT is required when telemetry is enabled")?;
    let resource = Resource::builder()
        .with_service_name(
            std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "promptjang-relay".into()),
        )
        .with_attribute(KeyValue::new("service.version", env!("CARGO_PKG_VERSION")))
        .build();
    global::set_text_map_propagator(opentelemetry_sdk::propagation::TraceContextPropagator::new());

    let span_exporter = TrackingSpanExporter(
        opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .build()?,
    );
    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(resource.clone())
        .with_batch_exporter(span_exporter)
        .build();
    global::set_tracer_provider(tracer_provider.clone());
    let tracer = tracer_provider.tracer("promptjang-relay");

    let metric_exporter = TrackingMetricExporter(
        opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .build()?,
    );
    let meter_provider = SdkMeterProvider::builder()
        .with_resource(resource.clone())
        .with_periodic_exporter(metric_exporter)
        .build();
    global::set_meter_provider(meter_provider.clone());
    install_metrics(&meter_provider);

    let log_exporter = TrackingLogExporter(
        opentelemetry_otlp::LogExporter::builder()
            .with_http()
            .build()?,
    );
    let logger_provider = SdkLoggerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(log_exporter)
        .build();
    let log_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "promptjang_relay_one=info,tower_http=info".into())
        .add_directive("opentelemetry=off".parse()?)
        .add_directive("reqwest=off".parse()?)
        .add_directive("hyper=off".parse()?);
    let log_layer = OpenTelemetryTracingBridge::new(&logger_provider).with_filter(log_filter);
    let trace_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(trace_layer)
        .with(log_layer)
        .init();
    Ok(TelemetryGuard {
        tracer: Some(tracer_provider),
        meter: Some(meter_provider),
        logger: Some(logger_provider),
    })
}

fn install_metrics(provider: &SdkMeterProvider) {
    let meter = provider.meter("promptjang-relay");
    let metrics = RelayMetrics {
        accepted: meter
            .u64_counter("promptjang.relay.events.accepted")
            .build(),
        rejected: meter
            .u64_counter("promptjang.relay.events.rejected")
            .build(),
        attempts: meter
            .u64_counter("promptjang.relay.delivery.attempts")
            .build(),
        duration_ms: meter
            .f64_histogram("promptjang.relay.delivery.duration")
            .with_unit("ms")
            .build(),
        queue_delay_ms: meter
            .f64_histogram("promptjang.relay.queue.delay")
            .with_unit("ms")
            .build(),
        queue_depth: meter.u64_gauge("promptjang.relay.queue.depth").build(),
        in_flight: meter
            .i64_up_down_counter("promptjang.relay.worker.in_flight")
            .build(),
        retries: meter
            .u64_counter("promptjang.relay.retries.scheduled")
            .build(),
        delivered: meter
            .u64_counter("promptjang.relay.events.delivered")
            .build(),
        expired: meter.u64_counter("promptjang.relay.events.expired").build(),
        recovery: meter
            .u64_counter("promptjang.relay.recovery.events")
            .build(),
        cleanup: meter.u64_counter("promptjang.relay.cleanup.events").build(),
    };
    let _ = METRICS.set(metrics);
}

pub fn accepted(replay: bool) {
    if let Some(m) = METRICS.get() {
        m.accepted.add(1, &[KeyValue::new("replay", replay)]);
    }
}
pub fn rejected(reason: &'static str) {
    if let Some(m) = METRICS.get() {
        m.rejected.add(1, &[KeyValue::new("reason", reason)]);
    }
}
pub fn attempt(duration_ms: f64, queue_delay_ms: f64, outcome: &'static str) {
    if let Some(m) = METRICS.get() {
        let attributes = [KeyValue::new("outcome", outcome)];
        m.attempts.add(1, &attributes);
        m.duration_ms.record(duration_ms, &attributes);
        m.queue_delay_ms.record(queue_delay_ms, &[]);
        if outcome == "delivered" {
            m.delivered.add(1, &[]);
        }
        if outcome == "expired" {
            m.expired.add(1, &[]);
        }
    }
}
pub fn retry_scheduled() {
    if let Some(m) = METRICS.get() {
        m.retries.add(1, &[]);
    }
}
pub fn queue_depth(value: u64) {
    if let Some(m) = METRICS.get() {
        m.queue_depth.record(value, &[]);
    }
}
pub fn in_flight(delta: i64) {
    if let Some(m) = METRICS.get() {
        m.in_flight.add(delta, &[]);
    }
}
pub fn recovered(value: u64) {
    if let Some(m) = METRICS.get() {
        m.recovery.add(value, &[]);
    }
}
pub fn cleaned(value: u64) {
    if let Some(m) = METRICS.get() {
        m.cleanup.add(value, &[]);
    }
}

struct TraceCarrier(HashMap<String, String>);
impl Extractor for TraceCarrier {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }
    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(String::as_str).collect()
    }
}
impl Injector for TraceCarrier {
    fn set(&mut self, key: &str, value: String) {
        self.0.insert(key.to_string(), value);
    }
}

pub fn set_span_parent(span: &tracing::Span, traceparent: Option<&str>, tracestate: Option<&str>) {
    let Some(traceparent) = traceparent else {
        return;
    };
    let mut values = HashMap::from([("traceparent".to_string(), traceparent.to_string())]);
    if let Some(tracestate) = tracestate {
        values.insert("tracestate".to_string(), tracestate.to_string());
    }
    let carrier = TraceCarrier(values);
    let context = global::get_text_map_propagator(|propagator| propagator.extract(&carrier));
    let _ = span.set_parent(context);
}

pub fn trace_headers_for_span(span: &tracing::Span) -> HashMap<String, String> {
    let mut carrier = TraceCarrier(HashMap::new());
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&span.context(), &mut carrier)
    });
    carrier.0
}

impl TelemetryGuard {
    pub async fn shutdown(self) {
        let shutdown = tokio::task::spawn_blocking(move || {
            if let Some(provider) = self.logger {
                let _ = provider.shutdown();
            }
            if let Some(provider) = self.meter {
                let _ = provider.shutdown();
            }
            if let Some(provider) = self.tracer {
                let _ = provider.shutdown();
            }
        });
        if tokio::time::timeout(std::time::Duration::from_secs(5), shutdown)
            .await
            .is_err()
        {
            tracing::warn!("OpenTelemetry shutdown exceeded five seconds");
        }
    }
}
