use std::time::Instant;

use anyhow::{Context, Result};
use chrono::Utc;
use reqwest::{Client, redirect::Policy};
use sqlx::{FromRow, SqlitePool};
use tracing::Instrument;
use uuid::Uuid;

use crate::config::Config;
use crate::domain::delivery::signature_header;
use crate::domain::{secrets, validation};
use crate::telemetry;

#[derive(FromRow)]
struct DeliveryJob {
    id: Uuid,
    destination_id: Uuid,
    payload_raw: Vec<u8>,
    content_type: String,
    event_type: Option<String>,
    traceparent: Option<String>,
    tracestate: Option<String>,
    next_attempt_at: chrono::DateTime<Utc>,
    retry_count: i32,
    max_retries: i32,
}

#[derive(FromRow)]
struct Destination {
    url: String,
    signing_secret_ciphertext: Vec<u8>,
    previous_signing_secret_ciphertext: Option<Vec<u8>>,
    enabled: bool,
}

async fn claim(pool: &SqlitePool) -> Result<Option<DeliveryJob>> {
    Ok(sqlx::query_as::<_, DeliveryJob>("UPDATE events SET status='PROCESSING',updated_at=CURRENT_TIMESTAMP WHERE id=(SELECT id FROM events WHERE status IN('QUEUED','RETRYING') AND next_attempt_at<=CURRENT_TIMESTAMP ORDER BY next_attempt_at,created_at LIMIT 1) RETURNING id,destination_id,payload_raw,content_type,event_type,traceparent,tracestate,next_attempt_at,retry_count,max_retries")
        .fetch_optional(pool).await?)
}

struct InFlightGuard;

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        telemetry::in_flight(-1);
    }
}

pub(crate) async fn process_one(pool: &SqlitePool, config: &Config) -> Result<bool> {
    let Some(job) = claim(pool).await? else {
        return Ok(false);
    };
    let claim_span = tracing::info_span!("relay.worker.claim", event_id=%job.id);
    claim_span.in_scope(|| {});
    let span = tracing::info_span!("relay.delivery.attempt", event_id=%job.id, destination_id=%job.destination_id, retry_count=job.retry_count);
    if config.otel_enabled {
        telemetry::set_span_parent(&span, job.traceparent.as_deref(), job.tracestate.as_deref());
    }
    process_job(pool, config, job, span.clone())
        .instrument(span)
        .await
}

async fn process_job(
    pool: &SqlitePool,
    config: &Config,
    job: DeliveryJob,
    span: tracing::Span,
) -> Result<bool> {
    telemetry::in_flight(1);
    let _in_flight = InFlightGuard;
    let queue_delay_ms = (Utc::now() - job.next_attempt_at).num_milliseconds().max(0) as f64;
    let destination = sqlx::query_as::<_, Destination>("SELECT url,signing_secret_ciphertext,previous_signing_secret_ciphertext,enabled FROM destinations WHERE id=$1 AND deleted_at IS NULL")
        .bind(job.destination_id).fetch_optional(pool).await?;
    let Some(destination) = destination else {
        fail(
            pool,
            &job,
            config,
            None,
            "destination was deleted",
            0,
            None,
            queue_delay_ms,
        )
        .await?;
        return Ok(true);
    };
    if !destination.enabled {
        fail(
            pool,
            &job,
            config,
            None,
            "destination is disabled",
            0,
            None,
            queue_delay_ms,
        )
        .await?;
        return Ok(true);
    }
    let (url, host, address) = match validation::resolve_destination(&destination.url, config).await
    {
        Ok(resolved) => resolved,
        Err(error) => {
            fail(
                pool,
                &job,
                config,
                None,
                &error.message,
                0,
                None,
                queue_delay_ms,
            )
            .await?;
            return Ok(true);
        }
    };
    let mut client_builder = Client::builder()
        .redirect(Policy::none())
        .timeout(std::time::Duration::from_secs(
            config.delivery_timeout_seconds,
        ))
        .resolve(&host, address);
    if let Some(path) = &config.extra_ca_cert_path {
        let certificate = std::fs::read(path)?;
        client_builder =
            client_builder.add_root_certificate(reqwest::Certificate::from_pem(&certificate)?);
    }
    let pinned_client = client_builder.build()?;
    let signing_secret = secrets::decrypt_secret(
        &config.encryption_key,
        &destination.signing_secret_ciphertext,
    )?;
    let timestamp = Utc::now().timestamp();
    let previous_signing_secret = destination
        .previous_signing_secret_ciphertext
        .as_deref()
        .map(|previous| secrets::decrypt_secret(&config.encryption_key, previous))
        .transpose()?;
    let signatures = signature_header(
        &signing_secret,
        previous_signing_secret.as_deref(),
        &job.id.to_string(),
        timestamp,
        &job.payload_raw,
    )
    .context("sign payload")?;
    let started = Instant::now();
    let mut request = pinned_client
        .post(url)
        .header("Content-Type", &job.content_type)
        .header(
            "User-Agent",
            format!("promptjang-relay/{}", env!("CARGO_PKG_VERSION")),
        )
        .header("webhook-id", job.id.to_string())
        .header("webhook-timestamp", timestamp)
        .header("webhook-signature", signatures)
        .body(job.payload_raw.clone());
    if let Some(value) = &job.event_type {
        request = request.header("X-PromptJang-Event-Type", value);
    }
    if config.otel_enabled {
        for (name, value) in telemetry::trace_headers_for_span(&span) {
            if !value.is_empty() {
                request = request.header(name, value);
            }
        }
    }
    match request.send().await {
        Ok(response) => {
            let status = response.status();
            let body = truncate(
                response.text().await.unwrap_or_default(),
                config.response_body_bytes,
            );
            let duration = started.elapsed().as_millis() as i64;
            if status.is_success() {
                succeed(
                    pool,
                    &job,
                    i32::from(status.as_u16()),
                    body,
                    duration,
                    queue_delay_ms,
                )
                .await?;
            } else {
                fail(
                    pool,
                    &job,
                    config,
                    Some(i32::from(status.as_u16())),
                    &format!("HTTP {status}"),
                    duration,
                    Some(body),
                    queue_delay_ms,
                )
                .await?;
            }
        }
        Err(error) => {
            fail(
                pool,
                &job,
                config,
                None,
                &error.to_string(),
                started.elapsed().as_millis() as i64,
                None,
                queue_delay_ms,
            )
            .await?
        }
    }
    Ok(true)
}

async fn succeed(
    pool: &SqlitePool,
    job: &DeliveryJob,
    status: i32,
    body: String,
    duration: i64,
    queue_delay_ms: f64,
) -> Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO delivery_attempts(id,event_id,status_code,response_body,duration_ms) VALUES($1,$2,$3,$4,$5)")
        .bind(Uuid::new_v4()).bind(job.id).bind(status).bind(body).bind(duration).execute(&mut *tx).await?;
    sqlx::query(
        "UPDATE events SET status='DELIVERED',last_error=NULL,updated_at=CURRENT_TIMESTAMP WHERE id=$1",
    )
    .bind(job.id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    telemetry::attempt(duration as f64, queue_delay_ms, "delivered");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn fail(
    pool: &SqlitePool,
    job: &DeliveryJob,
    config: &Config,
    status: Option<i32>,
    error: &str,
    duration: i64,
    body: Option<String>,
    queue_delay_ms: f64,
) -> Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO delivery_attempts(id,event_id,status_code,response_body,duration_ms,error) VALUES($1,$2,$3,$4,$5,$6)")
        .bind(Uuid::new_v4()).bind(job.id).bind(status).bind(body).bind(duration).bind(error).execute(&mut *tx).await?;
    if job.retry_count < job.max_retries {
        let delay = config
            .retry_delays_seconds
            .get(job.retry_count.max(0) as usize)
            .copied()
            .unwrap_or_else(|| *config.retry_delays_seconds.last().unwrap_or(&960));
        sqlx::query("UPDATE events SET status='RETRYING',retry_count=retry_count+1,next_attempt_at=datetime('now','+' || ?2 || ' seconds'),last_error=?3,updated_at=CURRENT_TIMESTAMP WHERE id=?1")
            .bind(job.id).bind(delay as f64).bind(error).execute(&mut *tx).await?;
        telemetry::retry_scheduled();
    } else {
        sqlx::query(
            "UPDATE events SET status='EXPIRED',last_error=$2,updated_at=now() WHERE id=$1",
        )
        .bind(job.id)
        .bind(error)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    telemetry::attempt(
        duration as f64,
        queue_delay_ms,
        if job.retry_count < job.max_retries {
            "retrying"
        } else {
            "expired"
        },
    );
    Ok(())
}

fn truncate(body: String, maximum: usize) -> String {
    if body.len() <= maximum {
        return body;
    }
    let mut boundary = maximum;
    while !body.is_char_boundary(boundary) {
        boundary -= 1;
    }
    format!("{}[truncated]", &body[..boundary])
}

pub(crate) async fn recover_stuck(pool: &SqlitePool, config: &Config) -> Result<u64> {
    let recovered = sqlx::query("UPDATE events SET status='RETRYING',next_attempt_at=CURRENT_TIMESTAMP,last_error='recovered after interrupted delivery',updated_at=CURRENT_TIMESTAMP WHERE status='PROCESSING' AND updated_at < datetime('now','-' || ?1 || ' seconds')")
        .bind(config.stuck_after_seconds as f64).execute(pool).await?.rows_affected();
    Ok(recovered)
}
