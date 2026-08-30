use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::DomainError;
use crate::domain::models::ApiKeyView;

pub async fn list(pool: &SqlitePool) -> Result<Vec<ApiKeyView>, DomainError> {
    let rows = sqlx::query_as::<_, (Uuid, String, String, Option<chrono::DateTime<chrono::Utc>>, chrono::DateTime<chrono::Utc>, bool, bool)>(
        "SELECT id,name,prefix,last_used_at,created_at,unrestricted,(secret_ciphertext IS NOT NULL) AS retrievable FROM api_keys ORDER BY created_at DESC",
    ).fetch_all(pool).await?;
    let mut result = Vec::with_capacity(rows.len());
    for (id, name, prefix, last_used_at, created_at, unrestricted, retrievable) in rows {
        let destination_ids = sqlx::query_scalar::<_, Uuid>(
            "SELECT destination_id FROM api_key_destinations WHERE api_key_id=$1 ORDER BY destination_id",
        ).bind(id).fetch_all(pool).await?;
        result.push(ApiKeyView {
            id,
            name,
            prefix,
            last_used_at,
            created_at,
            unrestricted,
            retrievable,
            destination_ids,
        });
    }
    Ok(result)
}

pub async fn create(
    pool: &SqlitePool,
    encryption_key: &[u8; 32],
    name: String,
    destination_ids: Vec<Uuid>,
) -> Result<(Uuid, String), DomainError> {
    let id = Uuid::new_v4();
    let secret = crate::domain::secrets::new_secret("pj_one_");
    let prefix = secret.chars().take(18).collect::<String>();
    let ciphertext = crate::domain::secrets::encrypt_secret(encryption_key, &secret)?;
    let unrestricted = destination_ids.is_empty();
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO api_keys (id,name,prefix,secret_hash,secret_ciphertext,unrestricted) VALUES ($1,$2,$3,$4,$5,$6)",
    )
    .bind(id)
    .bind(name)
    .bind(prefix)
    .bind(crate::domain::secrets::hash_secret(&secret))
    .bind(ciphertext)
    .bind(unrestricted)
    .execute(&mut *tx)
    .await?;
    for destination_id in destination_ids {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM destinations WHERE id=?1 AND deleted_at IS NULL)",
        )
        .bind(destination_id)
        .fetch_one(&mut *tx)
        .await?;
        if !exists {
            return Err(DomainError::not_found("destination scope not found"));
        }
        sqlx::query("INSERT INTO api_key_destinations (api_key_id,destination_id) VALUES ($1,$2)")
            .bind(id)
            .bind(destination_id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok((id, secret))
}

pub async fn reveal(
    pool: &SqlitePool,
    encryption_key: &[u8; 32],
    id: Uuid,
) -> Result<String, DomainError> {
    let ciphertext = sqlx::query_scalar::<_, Option<Vec<u8>>>(
        "SELECT secret_ciphertext FROM api_keys WHERE id=$1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    match ciphertext {
        None => Err(DomainError::not_found("API key not found")),
        Some(None) => Err(DomainError::conflict(
            "full key unavailable for this legacy record; revoke it and create a replacement",
        )),
        Some(Some(ciphertext)) => {
            crate::domain::secrets::decrypt_secret(encryption_key, &ciphertext)
        }
    }
}

pub async fn delete(pool: &SqlitePool, id: Uuid) -> Result<(), DomainError> {
    let changed = sqlx::query("DELETE FROM api_keys WHERE id=$1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    if changed == 0 {
        return Err(DomainError::not_found("API key not found"));
    }
    Ok(())
}
