use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::api::AppState;
use crate::api::error::ApiResult;
use crate::api::handlers::session;
use crate::domain::secrets;
use crate::domain::validation::{validate_destination_url, validate_name};
use crate::store;

#[derive(Deserialize)]
pub struct EndpointInput {
    name: String,
    url: String,
    enabled: Option<bool>,
}

pub async fn list_endpoints(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<serde_json::Value>> {
    session(&headers, &state.pool).await?;
    let endpoints = store::endpoints::list(&state.pool).await?;
    Ok(Json(
        json!({"destinations":endpoints,"endpoints":endpoints}),
    ))
}

pub async fn create_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<EndpointInput>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    session(&headers, &state.pool).await?;
    validate_name(&input.name)?;
    validate_destination_url(&input.url, &state.config).await?;
    let signing_secret = secrets::new_webhook_secret();
    let (id, secret) = store::endpoints::create(
        &state.pool,
        &state.config.encryption_key,
        input.name,
        input.url,
        signing_secret,
        input.enabled.unwrap_or(true),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(json!({"id":id,"secret":secret}))))
}

pub async fn update_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<EndpointInput>,
) -> ApiResult<Json<serde_json::Value>> {
    session(&headers, &state.pool).await?;
    validate_name(&input.name)?;
    validate_destination_url(&input.url, &state.config).await?;
    store::endpoints::update(&state.pool, id, input.name, input.url, input.enabled).await?;
    Ok(Json(json!({"updated":true})))
}

pub async fn rotate_secret(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    session(&headers, &state.pool).await?;
    let secret =
        store::endpoints::rotate_secret(&state.pool, &state.config.encryption_key, id).await?;
    Ok(Json(json!({"secret":secret})))
}

pub async fn test_destination(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    session(&headers, &state.pool).await?;
    let raw = br#"{"source":"promptjang-relay","type":"destination.test"}"#.to_vec();
    let payload = serde_json::from_slice(&raw).map_err(|error| anyhow::anyhow!(error))?;
    let outcome = store::events::ingest(
        &state.pool,
        id,
        payload,
        raw.clone(),
        crate::domain::secrets::hash_bytes(&raw),
        None,
        Some("destination.test".into()),
        None,
        "application/json".into(),
        None,
        None,
        state.config.rate_limit_per_minute,
        state.config.retry_delays_seconds.len() as i32,
    )
    .await?;
    let event_id = match outcome {
        store::events::IngestOutcome::Created { id } => id,
        store::events::IngestOutcome::IdempotentReplay { id, .. } => id,
    };
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({"id":event_id,"status":"QUEUED"})),
    ))
}

pub async fn finish_rotation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    session(&headers, &state.pool).await?;
    store::endpoints::finish_rotation(&state.pool, id).await?;
    Ok(Json(json!({"finished":true})))
}

pub async fn delete_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    session(&headers, &state.pool).await?;
    store::endpoints::delete(&state.pool, id).await?;
    Ok(Json(json!({"deleted":true})))
}
