use std::collections::HashMap;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use chrono::{DateTime, Utc};
use serde_json::json;
use uuid::Uuid;

use crate::api::AppState;
use crate::api::error::{ApiResult, AppError};
use crate::api::handlers::session;
use crate::store;

pub async fn list_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> ApiResult<Json<serde_json::Value>> {
    session(&headers, &state.pool).await?;
    let limit = query
        .get("limit")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(100)
        .clamp(1, 100);
    let cursor = query
        .get("cursor")
        .and_then(|value| value.parse::<DateTime<Utc>>().ok());
    let destination_id = query
        .get("destination_id")
        .or_else(|| query.get("endpoint_id"))
        .and_then(|value| value.parse::<Uuid>().ok());
    let status = query
        .get("status")
        .cloned()
        .filter(|value| !value.is_empty());
    let event_type = query
        .get("event_type")
        .cloned()
        .filter(|value| !value.is_empty());
    let events = store::events::list(
        &state.pool,
        store::events::EventFilters {
            limit,
            cursor,
            destination_id,
            status,
            event_type,
        },
    )
    .await?;
    let next_cursor = events.last().map(|event| event.created_at.to_rfc3339());
    Ok(Json(json!({"events":events,"next_cursor":next_cursor})))
}

pub async fn get_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    session(&headers, &state.pool).await?;
    let (event, attempts) = store::events::get(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::from(crate::domain::DomainError::not_found("event not found")))?;
    Ok(Json(json!({"event":event,"attempts":attempts})))
}

#[tracing::instrument(skip_all, fields(source_event_id=%id))]
pub async fn replay_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    session(&headers, &state.pool).await?;
    let replay = store::events::replay(&state.pool, id).await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({"id":replay,"status":"QUEUED","is_replay":true})),
    ))
}
