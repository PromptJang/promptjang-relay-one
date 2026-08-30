# OpenTelemetry

Relay emits no telemetry network activity when `PJ_OTEL_ENABLED=false` or `OTEL_SDK_DISABLED=true`. When enabled, `OTEL_EXPORTER_OTLP_ENDPOINT` is required and Relay initializes OTLP/HTTP traces, metrics, and logs while retaining structured stdout logs.

```env
PJ_OTEL_ENABLED=true
OTEL_SERVICE_NAME=promptjang-relay
OTEL_RESOURCE_ATTRIBUTES=deployment.environment=production
OTEL_EXPORTER_OTLP_ENDPOINT=https://collector.example.com
OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf
OTEL_EXPORTER_OTLP_HEADERS=authorization=Bearer%20REDACTED
OTEL_TRACES_EXPORTER=otlp
OTEL_METRICS_EXPORTER=otlp
OTEL_LOGS_EXPORTER=otlp
OTEL_PROPAGATORS=tracecontext
OTEL_METRIC_EXPORT_INTERVAL=60000
```

Signal-specific endpoint variables override the base endpoint. Any OTLP-compatible vendor can receive the signals; Relay has no vendor-specific integration. The [local Collector example](../examples/observability/README.md) routes traces to Jaeger, exposes Collector-translated metrics for Prometheus, and writes logs to both Collector debug output and a file volume.

Payloads, credentials, cookies, authorization headers, and OTLP headers are excluded. Event and destination IDs may appear in operational logs and traces, but never as metric labels. Collector failure is isolated from acceptance, delivery, and readiness.
