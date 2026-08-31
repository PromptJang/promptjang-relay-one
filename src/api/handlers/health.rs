use crate::api::AppState;
use crate::api::error::ApiResult;
use crate::domain::DomainError;
use axum::{Json, extract::State};
use serde_json::json;

pub async fn health() -> Json<serde_json::Value> {
    Json(json!({"status":"ok"}))
}

pub async fn ready(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .map_err(DomainError::from)?;
    Ok(Json(json!({"status":"ready"})))
}

pub async fn system(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    let mailboxes = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM mailboxes")
        .fetch_one(&state.pool)
        .await
        .map_err(DomainError::from)?;
    let (unread,claimed,acknowledged) = sqlx::query_as::<_,(i64,i64,i64)>("SELECT COALESCE(sum(CASE WHEN status='UNREAD' THEN 1 ELSE 0 END),0),COALESCE(sum(CASE WHEN status='CLAIMED' THEN 1 ELSE 0 END),0),COALESCE(sum(CASE WHEN status='ACKNOWLEDGED' THEN 1 ELSE 0 END),0) FROM mailbox_messages")
        .fetch_one(&state.pool)
        .await
        .map_err(DomainError::from)?;
    let database_bytes = std::fs::metadata(&state.config.database_path)
        .map(|m| m.len())
        .unwrap_or(0);
    Ok(Json(json!({
        "version":env!("CARGO_PKG_VERSION"),
        "runtime":"local-sqlite",
        "surface":if state.config.desktop_mode { "desktop" } else { "browser" },
        "database_path":state.config.database_path,
        "database_bytes":database_bytes,
        "mailboxes":mailboxes,
        "messages":{"unread":unread,"claimed":claimed,"acknowledged":acknowledged},
        "update_check_enabled":state.config.update_check_enabled,
        "limits":{"max_payload_bytes":state.config.max_payload_bytes,"retention_days":state.config.retention_days,"max_claim_batch":state.config.mailbox_claim_limit}
    })))
}
