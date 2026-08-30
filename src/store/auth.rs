use axum::http::HeaderMap;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::{DomainError, secrets};

fn bearer(headers: &HeaderMap) -> Result<&str, DomainError> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| value.starts_with("pj_one_"))
        .ok_or_else(|| DomainError::bad_request("invalid API key"))
}

async fn key_id(headers: &HeaderMap, pool: &SqlitePool) -> Result<Uuid, DomainError> {
    let hash = secrets::hash_secret(bearer(headers)?);
    let id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM api_keys WHERE secret_hash=?1")
        .bind(hash)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| DomainError::bad_request("invalid API key"))?;
    sqlx::query("UPDATE api_keys SET last_used_at=CURRENT_TIMESTAMP WHERE id=?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(id)
}

pub async fn require_unscoped_api_key(
    headers: &HeaderMap,
    pool: &SqlitePool,
) -> Result<Uuid, DomainError> {
    key_id(headers, pool).await
}
