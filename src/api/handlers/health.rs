use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use serde_json::json;

use crate::api::handlers::session;
use crate::store;

use crate::api::AppState;
use crate::api::error::{ApiResult, AppError};

pub async fn health() -> Json<serde_json::Value> {
    Json(json!({"status":"ok"}))
}

pub async fn system(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<serde_json::Value>> {
    session(&headers, &state.pool).await?;
    let (queued, delivered, expired, retrying) = store::events::summary(&state.pool).await?;
    let collector = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .ok()
        .and_then(|raw| url::Url::parse(&raw).ok())
        .and_then(|url| url.host_str().map(str::to_string));
    let export_status = crate::telemetry::export_status();
    let last_successful_export_at = export_status
        .last_successful_export_at
        .map(chrono::DateTime::<chrono::Utc>::from)
        .map(|value| value.to_rfc3339());
    Ok(Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "queue":{"active":queued,"retrying":retrying,"delivered":delivered,"expired":expired},
        "limits":{
            "max_payload_bytes":state.config.max_payload_bytes,
            "rate_per_destination_per_minute":state.config.rate_limit_per_minute,
            "retention_days":state.config.retention_days,
            "worker_concurrency":state.config.worker_concurrency,
            "delivery_timeout_seconds":state.config.delivery_timeout_seconds,
            "retry_delays_seconds":state.config.retry_delays_seconds,
        },
        "telemetry":{
            "enabled":state.config.otel_enabled,
            "signals": if state.config.otel_enabled { vec!["traces","metrics","logs"] } else { vec![] },
            "protocol":std::env::var("OTEL_EXPORTER_OTLP_PROTOCOL").unwrap_or_else(|_| "http/protobuf".into()),
            "collector_host":collector,
            "last_successful_export_at":last_successful_export_at,
            "last_export_error":export_status.last_error,
        }
    })))
}

pub async fn ready(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .map_err(|error| AppError::from(anyhow::anyhow!(error.to_string())))?;
    Ok(Json(json!({"status":"ready"})))
}
