use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::DomainError;
use crate::domain::models::ApiKeyView;

pub async fn list(pool: &SqlitePool) -> Result<Vec<ApiKeyView>, DomainError> {
    let rows = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            String,
            Option<chrono::DateTime<chrono::Utc>>,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        "SELECT id,name,prefix,last_used_at,created_at FROM api_keys ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, name, prefix, last_used_at, created_at)| ApiKeyView {
            id,
            name,
            prefix,
            last_used_at,
            created_at,
            retrievable: true,
        })
        .collect())
}

pub async fn create(
    pool: &SqlitePool,
    encryption_key: &[u8; 32],
    name: String,
) -> Result<(Uuid, String), DomainError> {
    let id = Uuid::new_v4();
    let secret = crate::domain::secrets::new_secret("pj_one_");
    let prefix = secret.chars().take(16).collect::<String>();
    let ciphertext = crate::domain::secrets::encrypt_secret(encryption_key, &secret)?;
    sqlx::query(
        "INSERT INTO api_keys(id,name,prefix,secret_hash,secret_ciphertext) VALUES(?1,?2,?3,?4,?5)",
    )
    .bind(id)
    .bind(name)
    .bind(prefix)
    .bind(crate::domain::secrets::hash_secret(&secret))
    .bind(ciphertext)
    .execute(pool)
    .await?;
    Ok((id, secret))
}

pub async fn reveal(
    pool: &SqlitePool,
    encryption_key: &[u8; 32],
    id: Uuid,
) -> Result<String, DomainError> {
    let ciphertext =
        sqlx::query_scalar::<_, Vec<u8>>("SELECT secret_ciphertext FROM api_keys WHERE id=?1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| DomainError::not_found("API key not found"))?;
    crate::domain::secrets::decrypt_secret(encryption_key, &ciphertext)
}

pub async fn delete(pool: &SqlitePool, id: Uuid) -> Result<(), DomainError> {
    let changed = sqlx::query("DELETE FROM api_keys WHERE id=?1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    if changed == 0 {
        return Err(DomainError::not_found("API key not found"));
    }
    Ok(())
}
