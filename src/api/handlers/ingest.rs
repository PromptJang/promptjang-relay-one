use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use serde_json::json;
use tracing::Instrument;
use uuid::Uuid;

use crate::api::AppState;
use crate::api::error::{ApiResult, AppError};
use crate::domain::secrets;
use crate::domain::validation::{ensure_payload_size, extract_header, idempotency_key};
use crate::store;
use crate::store::events::IngestOutcome;
use crate::telemetry;

#[tracing::instrument(skip_all, fields(destination_id=%endpoint_id))]
pub async fn ingest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(endpoint_id): Path<Uuid>,
    body: Bytes,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    store::auth::require_api_key(&headers, &state.pool, endpoint_id)
        .await
        .map_err(|_| AppError::unauthorized("invalid API key"))?;
    if let Err(error) = ensure_payload_size(body.len(), state.config.max_payload_bytes) {
        telemetry::rejected("payload_too_large");
        return Err(error.into());
    }
    let payload: serde_json::Value = serde_json::from_slice(&body).map_err(|_| {
        AppError::from(crate::domain::DomainError::bad_request(
            "payload must be valid JSON",
        ))
    })?;
    let payload_hash = secrets::hash_bytes(&body);
    let idempotency = idempotency_key(&headers)?;
    let key_hash = idempotency.as_deref().map(secrets::hash_secret);
    let event_type = extract_header(&headers, "X-Event-Type");
    let correlation_id = extract_header(&headers, "X-Correlation-ID");
    let content_type = headers
        .get("Content-Type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/json")
        .to_string();
    let mut traceparent = extract_header(&headers, "traceparent");
    let mut tracestate = extract_header(&headers, "tracestate");
    let acceptance = tracing::info_span!(
        "relay.event.accept",
        destination_id = %endpoint_id,
        idempotency_present = key_hash.is_some()
    );
    if state.config.otel_enabled {
        telemetry::set_span_parent(&acceptance, traceparent.as_deref(), tracestate.as_deref());
        let generated = telemetry::trace_headers_for_span(&acceptance);
        traceparent = traceparent.or_else(|| generated.get("traceparent").cloned());
        tracestate = tracestate.or_else(|| generated.get("tracestate").cloned());
    }
    match store::events::ingest(
        &state.pool,
        endpoint_id,
        payload,
        body.to_vec(),
        payload_hash,
        key_hash,
        event_type,
        correlation_id,
        content_type,
        traceparent,
        tracestate,
        state.config.rate_limit_per_minute,
        state.config.retry_delays_seconds.len() as i32,
    )
    .instrument(acceptance)
    .await?
    {
        IngestOutcome::Created { id } => {
            telemetry::accepted(false);
            Ok((
                StatusCode::ACCEPTED,
                Json(json!({"id":id,"status":"QUEUED"})),
            ))
        }
        IngestOutcome::IdempotentReplay { id, status } => Ok((
            StatusCode::ACCEPTED,
            Json(json!({"id":id,"status":status,"idempotent_replay":true})),
        )),
    }
}
