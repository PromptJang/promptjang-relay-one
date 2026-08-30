use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use tracing::Instrument;

use crate::api::AppState;
use crate::api::error::{ApiResult, AppError};
use crate::domain::secrets;
use crate::domain::validation::{ensure_payload_size, extract_header, idempotency_key};
use crate::store;
use crate::store::mail::{self, MailPushOutcome};
use crate::telemetry;

async fn api_key(headers: &HeaderMap, state: &AppState) -> Result<(), AppError> {
    store::auth::require_unscoped_api_key(headers, &state.pool)
        .await
        .map(|_| ())
        .map_err(|_| AppError::unauthorized("invalid API key"))
}

#[derive(Deserialize)]
pub struct ClaimInput {
    limit: Option<i64>,
    lease_seconds: Option<i64>,
}

#[derive(Deserialize)]
pub struct AckInput {
    claim_token: String,
}

fn message_view(message: &mail::MailboxMessage, include_token: bool) -> serde_json::Value {
    let payload_text = String::from_utf8_lossy(&message.payload_raw);
    let mut view = json!({
        "id": message.id,
        "status": message.status,
        "content_type": message.content_type,
        "payload": payload_text,
        "payload_json": serde_json::from_slice::<serde_json::Value>(&message.payload_raw).ok(),
        "payload_sha256": message.payload_sha256,
        "traceparent": message.traceparent,
        "claim_count": message.claim_count,
        "created_at": message.created_at,
        "updated_at": message.updated_at,
    });
    if include_token {
        view["claim_token"] = json!(message.claim_token);
        view["claimed_until"] = json!(message.claimed_until);
    }
    view
}

#[tracing::instrument(skip_all, fields(mailbox=%name))]
pub async fn push(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(name): Path<String>,
    body: Bytes,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    api_key(&headers, &state).await?;
    mail::validate_mailbox_name(&name)?;
    if let Err(error) = ensure_payload_size(body.len(), state.config.max_payload_bytes) {
        telemetry::rejected("payload_too_large");
        return Err(error.into());
    }
    let payload_hash = secrets::hash_bytes(&body);
    let idempotency = idempotency_key(&headers)?;
    let key_hash = idempotency.as_deref().map(secrets::hash_secret);
    let content_type = headers
        .get("Content-Type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/json")
        .to_string();
    let mut traceparent = extract_header(&headers, "traceparent");
    let mut tracestate = extract_header(&headers, "tracestate");
    let acceptance = tracing::info_span!(
        "relay.mail.accept",
        mailbox = %name,
        idempotency_present = key_hash.is_some()
    );
    if state.config.otel_enabled {
        telemetry::set_span_parent(&acceptance, traceparent.as_deref(), tracestate.as_deref());
        let generated = telemetry::trace_headers_for_span(&acceptance);
        traceparent = traceparent.or_else(|| generated.get("traceparent").cloned());
        tracestate = tracestate.or_else(|| generated.get("tracestate").cloned());
    }
    let payload = serde_json::from_slice::<serde_json::Value>(&body).ok();
    let outcome = async {
        mail::push(
            &state.pool,
            &name,
            mail::IncomingMessage {
                payload_raw: body.to_vec(),
                payload,
                content_type,
                payload_sha256: payload_hash,
                idempotency_key_hash: key_hash,
                traceparent,
                tracestate,
            },
        )
        .await
    }
    .instrument(acceptance)
    .await?;
    telemetry::accepted(false);
    match outcome {
        MailPushOutcome::Created { id } => Ok((
            StatusCode::ACCEPTED,
            Json(json!({"id":id,"mailbox":name,"status":"UNREAD"})),
        )),
        MailPushOutcome::IdempotentReplay { id, status } => Ok((
            StatusCode::ACCEPTED,
            Json(json!({"id":id,"mailbox":name,"status":status,"idempotent_replay":true})),
        )),
    }
}

pub async fn claim(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(input): Json<ClaimInput>,
) -> ApiResult<Json<serde_json::Value>> {
    api_key(&headers, &state).await?;
    mail::validate_mailbox_name(&name)?;
    let limit = mail::normalize_claim_limit(input.limit);
    let lease = mail::normalize_lease(input.lease_seconds);
    let messages = store::mail::claim(&state.pool, &name, limit, lease.seconds).await?;
    let views: Vec<_> = messages.iter().map(|m| message_view(m, true)).collect();
    Ok(Json(
        json!({"mailbox":name,"lease_seconds":lease.seconds,"messages":views}),
    ))
}

pub async fn acknowledge(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((name, id)): Path<(String, uuid::Uuid)>,
    Json(input): Json<AckInput>,
) -> ApiResult<Json<serde_json::Value>> {
    api_key(&headers, &state).await?;
    mail::validate_mailbox_name(&name)?;
    let ok = store::mail::acknowledge(&state.pool, &name, id, &input.claim_token, true).await?;
    if ok {
        Ok(Json(json!({"id":id,"status":"ACKNOWLEDGED"})))
    } else {
        Err(AppError::from(crate::domain::DomainError::conflict(
            "message is not claimed with this token (already acknowledged or lease expired)",
        )))
    }
}

pub async fn nack(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((name, id)): Path<(String, uuid::Uuid)>,
    Json(input): Json<AckInput>,
) -> ApiResult<Json<serde_json::Value>> {
    api_key(&headers, &state).await?;
    mail::validate_mailbox_name(&name)?;
    let ok = store::mail::acknowledge(&state.pool, &name, id, &input.claim_token, false).await?;
    if ok {
        Ok(Json(json!({"id":id,"status":"UNREAD"})))
    } else {
        Err(AppError::from(crate::domain::DomainError::conflict(
            "message is not claimed with this token (already acknowledged or lease expired)",
        )))
    }
}

pub async fn list_mailboxes(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<serde_json::Value>> {
    crate::api::handlers::session(&headers, &state.pool).await?;
    let mailboxes = store::mail::list_mailboxes(&state.pool).await?;
    Ok(Json(json!({"mailboxes":mailboxes})))
}

pub async fn delete_mailbox(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    crate::api::handlers::session(&headers, &state.pool).await?;
    mail::validate_mailbox_name(&name)?;
    let deleted = store::mail::delete_mailbox(&state.pool, &name).await?;
    if deleted {
        Ok(Json(json!({"deleted":true})))
    } else {
        Err(AppError::from(crate::domain::DomainError::not_found(
            "mailbox not found",
        )))
    }
}

pub async fn list_messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> ApiResult<Json<serde_json::Value>> {
    crate::api::handlers::session(&headers, &state.pool).await?;
    mail::validate_mailbox_name(&name)?;
    let limit = query
        .get("limit")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(100)
        .clamp(1, 500);
    let status = query.get("status").map(String::as_str);
    let messages = store::mail::list_messages(&state.pool, &name, status, limit).await?;
    let views: Vec<_> = messages.iter().map(|m| message_view(m, false)).collect();
    Ok(Json(json!({"mailbox":name,"messages":views})))
}
