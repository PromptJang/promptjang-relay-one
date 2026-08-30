use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::DomainError;
use crate::domain::models::{AttemptView, EventView};

pub enum IngestOutcome {
    Created { id: Uuid },
    IdempotentReplay { id: Uuid, status: String },
}

#[derive(Default)]
pub struct EventFilters {
    pub limit: i64,
    pub cursor: Option<DateTime<Utc>>,
    pub destination_id: Option<Uuid>,
    pub status: Option<String>,
    pub event_type: Option<String>,
}

const EVENT_COLUMNS: &str = "id,destination_id,destination_id AS endpoint_id,status,event_type,correlation_id,payload,content_type,traceparent,tracestate,retry_count,max_retries,is_replay,source_event_id,next_attempt_at,last_error,created_at,updated_at";

pub async fn list(pool: &SqlitePool, filters: EventFilters) -> Result<Vec<EventView>, DomainError> {
    let query = format!(
        "SELECT {EVENT_COLUMNS} FROM events WHERE (?1 IS NULL OR created_at<?1) AND (?2 IS NULL OR destination_id=?2) AND (?3 IS NULL OR status=?3) AND (?4 IS NULL OR event_type=?4) ORDER BY created_at DESC LIMIT ?5"
    );
    sqlx::query_as::<_, EventView>(&query)
        .bind(filters.cursor)
        .bind(filters.destination_id)
        .bind(filters.status)
        .bind(filters.event_type)
        .bind(filters.limit.clamp(1, 100))
        .fetch_all(pool)
        .await
        .map_err(DomainError::from)
}

pub async fn get(
    pool: &SqlitePool,
    id: Uuid,
) -> Result<Option<(EventView, Vec<AttemptView>)>, DomainError> {
    let query = format!("SELECT {EVENT_COLUMNS} FROM events WHERE id=$1");
    let event = sqlx::query_as::<_, EventView>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await?;
    let Some(event) = event else {
        return Ok(None);
    };
    let attempts = sqlx::query_as::<_, AttemptView>("SELECT id,event_id,status_code,response_body,duration_ms,error,attempted_at FROM delivery_attempts WHERE event_id=$1 ORDER BY attempted_at")
        .bind(id).fetch_all(pool).await?;
    Ok(Some((event, attempts)))
}

pub async fn replay(pool: &SqlitePool, source_id: Uuid) -> Result<Uuid, DomainError> {
    let source = sqlx::query_as::<_, (Uuid, Value, Vec<u8>, String, String, Option<String>, Option<String>, Option<String>, Option<String>)>(
        "SELECT destination_id,payload,payload_raw,payload_sha256,content_type,event_type,correlation_id,traceparent,tracestate FROM events WHERE id=$1",
    ).bind(source_id).fetch_optional(pool).await?;
    let Some(source) = source else {
        return Err(DomainError::not_found("event not found"));
    };
    let replay_id = Uuid::new_v4();
    sqlx::query("INSERT INTO events(id,destination_id,status,payload,payload_raw,payload_sha256,content_type,event_type,correlation_id,traceparent,tracestate,is_replay,source_event_id,max_retries) VALUES($1,$2,'QUEUED',$3,$4,$5,$6,$7,$8,$9,$10,true,$11,(SELECT max_retries FROM events WHERE id=$11))")
        .bind(replay_id).bind(source.0).bind(source.1).bind(source.2).bind(source.3).bind(source.4)
        .bind(source.5).bind(source.6).bind(source.7).bind(source.8).bind(source_id).execute(pool).await?;
    Ok(replay_id)
}

#[allow(clippy::too_many_arguments)]
pub async fn ingest(
    pool: &SqlitePool,
    destination_id: Uuid,
    payload: Value,
    payload_raw: Vec<u8>,
    payload_hash: String,
    key_hash: Option<String>,
    event_type: Option<String>,
    correlation_id: Option<String>,
    content_type: String,
    traceparent: Option<String>,
    tracestate: Option<String>,
    rate_limit: i64,
    max_retries: i32,
) -> Result<IngestOutcome, DomainError> {
    let enabled = sqlx::query_scalar::<_, bool>(
        "SELECT enabled FROM destinations WHERE id=$1 AND deleted_at IS NULL",
    )
    .bind(destination_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| DomainError::not_found("destination not found"))?;
    if !enabled {
        return Err(DomainError::conflict("destination is disabled"));
    }
    if rate_limit > 0 {
        let count = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM events WHERE destination_id=?1 AND is_replay=0 AND created_at>=datetime('now','start of minute')")
            .bind(destination_id).fetch_one(pool).await?;
        if count >= rate_limit {
            return Err(DomainError::too_many_requests(
                "configured accepted-event rate reached",
            ));
        }
    }
    let mut tx = pool.begin().await?;
    if let Some(ref hash) = key_hash {
        let existing = sqlx::query_as::<_, (Uuid, String, String)>("SELECT id,payload_sha256,status FROM events WHERE destination_id=$1 AND idempotency_key_hash=$2 AND is_replay=false")
            .bind(destination_id).bind(hash).fetch_optional(&mut *tx).await?;
        if let Some((id, existing_hash, status)) = existing {
            if existing_hash != payload_hash {
                return Err(DomainError::conflict(
                    "Idempotency-Key was already used with a different payload",
                ));
            }
            tx.commit().await?;
            return Ok(IngestOutcome::IdempotentReplay { id, status });
        }
    }
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO events(id,destination_id,status,event_type,correlation_id,payload,payload_raw,payload_sha256,idempotency_key_hash,content_type,traceparent,tracestate,max_retries) VALUES($1,$2,'QUEUED',$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)")
        .bind(id).bind(destination_id).bind(event_type).bind(correlation_id).bind(payload).bind(payload_raw)
        .bind(payload_hash).bind(key_hash).bind(content_type).bind(traceparent).bind(tracestate).bind(max_retries)
        .execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(IngestOutcome::Created { id })
}

pub async fn summary(pool: &SqlitePool) -> Result<(i64, i64, i64, i64), DomainError> {
    sqlx::query_as::<_, (i64, i64, i64, i64)>("SELECT sum(CASE WHEN status IN('QUEUED','RETRYING','PROCESSING') THEN 1 ELSE 0 END),sum(CASE WHEN status='DELIVERED' THEN 1 ELSE 0 END),sum(CASE WHEN status='EXPIRED' THEN 1 ELSE 0 END),sum(CASE WHEN status='RETRYING' THEN 1 ELSE 0 END) FROM events")
        .fetch_one(pool).await.map_err(DomainError::from)
}

pub async fn cleanup(pool: &SqlitePool, retention_days: i64) -> Result<u64, DomainError> {
    if retention_days <= 0 {
        return Ok(0);
    }
    Ok(sqlx::query("DELETE FROM events WHERE status IN('DELIVERED','EXPIRED') AND updated_at < datetime('now', '-' || ?1 || ' days')")
        .bind(retention_days as i32).execute(pool).await?.rows_affected())
}
