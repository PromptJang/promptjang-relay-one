pub mod docs;
pub mod endpoints;
pub mod events;
pub mod health;
pub mod ingest;
pub mod keys;
pub mod mail;

use axum::http::HeaderMap;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::api::error::AppError;
pub(crate) async fn session(_headers: &HeaderMap, _pool: &SqlitePool) -> Result<Uuid, AppError> {
    Ok(Uuid::nil())
}
