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

async fn key_id(headers: &HeaderMap, pool: &SqlitePool) -> Result<(Uuid, bool), DomainError> {
    let hash = secrets::hash_secret(bearer(headers)?);
    let row = sqlx::query_as::<_, (Uuid, bool)>(
        "SELECT id,unrestricted FROM api_keys WHERE secret_hash=?1",
    )
    .bind(hash)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| DomainError::bad_request("invalid API key"))?;
    sqlx::query("UPDATE api_keys SET last_used_at=CURRENT_TIMESTAMP WHERE id=?1")
        .bind(row.0)
        .execute(pool)
        .await?;
    Ok(row)
}

pub async fn require_api_key(
    headers: &HeaderMap,
    pool: &SqlitePool,
    destination_id: Uuid,
) -> Result<Uuid, DomainError> {
    let (id, unrestricted) = key_id(headers, pool).await?;
    if unrestricted {
        return Ok(id);
    }
    let allowed = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM api_key_destinations WHERE api_key_id=?1 AND destination_id=?2",
    )
    .bind(id)
    .bind(destination_id)
    .fetch_one(pool)
    .await?;
    if allowed == 1 {
        Ok(id)
    } else {
        Err(DomainError::bad_request("API key does not allow this destination"))
    }
}

pub async fn require_unscoped_api_key(
    headers: &HeaderMap,
    pool: &SqlitePool,
) -> Result<Uuid, DomainError> {
    let (id, unrestricted) = key_id(headers, pool).await?;
    if unrestricted {
        Ok(id)
    } else {
        Err(DomainError::bad_request("mailbox access requires an unrestricted API key"))
    }
}
