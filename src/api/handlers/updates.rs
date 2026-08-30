use crate::api::AppState;
use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct UpdateQuery {
    refresh: Option<bool>,
}

pub async fn check_update(
    State(state): State<AppState>,
    Query(query): Query<UpdateQuery>,
) -> Json<serde_json::Value> {
    Json(json!(
        crate::updates::check(
            state.config.update_check_enabled,
            query.refresh.unwrap_or(false),
            &state.config.data_dir.join("update-cache.json"),
        )
        .await
    ))
}
