pub mod docs;
pub mod health;
pub mod integrations;
pub mod keys;
pub mod mail;
pub mod updates;

use axum::http::HeaderMap;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::api::error::AppError;
pub(crate) async fn session(_headers: &HeaderMap, _pool: &SqlitePool) -> Result<Uuid, AppError> {
    Ok(Uuid::nil())
}
