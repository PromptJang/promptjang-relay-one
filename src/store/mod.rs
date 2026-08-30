pub mod auth;
pub mod endpoints;
pub mod events;
pub mod keys;
pub mod mail;

pub async fn checkpoint(path: &std::path::Path) -> anyhow::Result<()> {
    let options = sqlx::sqlite::SqliteConnectOptions::new().filename(path);
    let pool = sqlx::SqlitePool::connect_with(options).await?;
    sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)").execute(&pool).await?;
    pool.close().await;
    Ok(())
}
