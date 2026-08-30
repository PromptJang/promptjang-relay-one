use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::api::AppState;
use crate::api::error::ApiResult;
use crate::api::handlers::session;
use crate::domain::validation::validate_name;
use crate::store;

#[derive(Deserialize)]
pub struct KeyInput {
    name: String,
    #[serde(default)]
    destination_ids: Vec<Uuid>,
}

pub async fn list_keys(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<serde_json::Value>> {
    session(&headers, &state.pool).await?;
    let keys = store::keys::list(&state.pool).await?;
    Ok(Json(json!({"keys":keys})))
}

pub async fn create_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<KeyInput>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    session(&headers, &state.pool).await?;
    validate_name(&input.name)?;
    let (id, secret) = store::keys::create(
        &state.pool,
        &state.config.encryption_key,
        input.name,
        input.destination_ids,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(json!({"id":id,"key":secret}))))
}

pub async fn reveal_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> ApiResult<Response> {
    session(&headers, &state.pool).await?;
    let secret = store::keys::reveal(&state.pool, &state.config.encryption_key, id).await?;
    let mut response = Json(json!({"key":secret})).into_response();
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    Ok(response)
}

pub async fn delete_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    session(&headers, &state.pool).await?;
    store::keys::delete(&state.pool, id).await?;
    Ok(Json(json!({"deleted":true})))
}
