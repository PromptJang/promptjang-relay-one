use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use serde::Deserialize;
use serde_json::json;

use crate::api::AppState;
use crate::api::error::{ApiResult, AppError};
use crate::store;

#[derive(Deserialize)]
pub struct LoginInput {
    username: Option<String>,
    #[serde(rename = "email")]
    email_alias: Option<String>,
    password: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> ApiResult<Json<serde_json::Value>> {
    let owner_id = store::auth::verify_login(
        &state.pool,
        input
            .username
            .as_deref()
            .or(input.email_alias.as_deref())
            .unwrap_or_default(),
        &input.password,
    )
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::unauthorized("invalid credentials"))?;
    let token = store::auth::issue_session(&state.pool, owner_id, state.config.session_ttl_seconds)
        .await
        .map_err(AppError::from)?;
    Ok(Json(
        json!({"token":token,"expires_in":state.config.session_ttl_seconds}),
    ))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<serde_json::Value>> {
    store::auth::revoke_session(&headers, &state.pool)
        .await
        .map_err(AppError::from)?;
    Ok(Json(json!({"revoked":true})))
}
