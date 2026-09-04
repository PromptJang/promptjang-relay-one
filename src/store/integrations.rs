use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::DomainError;

#[derive(Debug, Serialize)]
pub struct McpInstallation {
    pub client: String,
    pub key_id: Uuid,
    pub configured_at: DateTime<Utc>,
    pub adapter_verified_at: Option<DateTime<Utc>>,
    pub last_activity_at: Option<DateTime<Utc>>,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<McpInstallation>, DomainError> {
    let rows = sqlx::query_as::<
        _,
        (
            String,
            Uuid,
            DateTime<Utc>,
            Option<DateTime<Utc>>,
            Option<DateTime<Utc>>,
        ),
    >(
        "SELECT client,key_id,configured_at,adapter_verified_at,last_activity_at FROM mcp_installations ORDER BY client",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(client, key_id, configured_at, adapter_verified_at, last_activity_at)| {
                McpInstallation {
                    client,
                    key_id,
                    configured_at,
                    adapter_verified_at,
                    last_activity_at,
                }
            },
        )
        .collect())
}

pub async fn configured(pool: &SqlitePool, client: &str, key_id: Uuid) -> Result<(), DomainError> {
    sqlx::query(
        "INSERT INTO mcp_installations(client,key_id,configured_at,adapter_verified_at,last_activity_at)
         VALUES(?1,?2,CURRENT_TIMESTAMP,NULL,NULL)
         ON CONFLICT(client) DO UPDATE SET key_id=excluded.key_id,configured_at=CURRENT_TIMESTAMP,adapter_verified_at=NULL,last_activity_at=NULL",
    )
    .bind(client)
    .bind(key_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn adapter_verified(
    pool: &SqlitePool,
    client: &str,
    key_id: Uuid,
) -> Result<bool, DomainError> {
    let changed = sqlx::query(
        "UPDATE mcp_installations SET adapter_verified_at=CURRENT_TIMESTAMP WHERE client=?1 AND key_id=?2",
    )
    .bind(client)
    .bind(key_id)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(changed == 1)
}

pub async fn activity(pool: &SqlitePool, client: &str, key_id: Uuid) -> Result<(), DomainError> {
    if !matches!(
        client,
        "claude-desktop" | "claude-code" | "codex" | "opencode"
    ) {
        return Ok(());
    }
    sqlx::query(
        "UPDATE mcp_installations SET last_activity_at=CURRENT_TIMESTAMP WHERE client=?1 AND key_id=?2",
    )
    .bind(client)
    .bind(key_id)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, runtime, store};

    #[tokio::test]
    async fn tracks_configuration_verification_and_known_client_activity() {
        let directory =
            std::env::temp_dir().join(format!("relay-one-mcp-state-{}", Uuid::new_v4()));
        let config = Config::load(Some(directory.clone()), 0).expect("create test config");
        let pool = runtime::open_pool(&config)
            .await
            .expect("open test database");
        let (key_id, _) = store::keys::create(&pool, &config.encryption_key, "agent".into())
            .await
            .expect("create API key");

        configured(&pool, "codex", key_id)
            .await
            .expect("record setup");
        assert!(
            adapter_verified(&pool, "codex", key_id)
                .await
                .expect("record adapter check")
        );
        activity(&pool, "unknown-client", key_id)
            .await
            .expect("ignore unknown client");
        activity(&pool, "codex", key_id)
            .await
            .expect("record client activity");

        let installations = list(&pool).await.expect("list installations");
        assert_eq!(installations.len(), 1);
        assert_eq!(installations[0].client, "codex");
        assert_eq!(installations[0].key_id, key_id);
        assert!(installations[0].adapter_verified_at.is_some());
        assert!(installations[0].last_activity_at.is_some());

        pool.close().await;
        std::fs::remove_dir_all(directory).expect("remove test directory");
    }
}
