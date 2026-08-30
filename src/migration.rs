use anyhow::{Context, Result, bail};
use base64::Engine;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::Path;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Archive {
    format: String,
    exported_at: chrono::DateTime<chrono::Utc>,
    mailboxes: Vec<Mailbox>,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize)]
struct Mailbox {
    id: Uuid,
    name: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize)]
struct Message {
    id: Uuid,
    mailbox_id: Uuid,
    status: String,
    payload_base64: String,
    content_type: String,
    payload: Option<serde_json::Value>,
    payload_sha256: String,
    idempotency_key_hash: Option<String>,
    traceparent: Option<String>,
    tracestate: Option<String>,
    claim_count: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn export(pool: &SqlitePool, output: &Path) -> Result<()> {
    let mailboxes = sqlx::query_as::<_, (Uuid, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT id,name,created_at FROM mailboxes ORDER BY created_at",
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, name, created_at)| Mailbox {
        id,
        name,
        created_at,
    })
    .collect();
    let rows = sqlx::query_as::<_, (Uuid,Uuid,String,Vec<u8>,String,Option<serde_json::Value>,String,Option<String>,Option<String>,Option<String>,i32,chrono::DateTime<chrono::Utc>,chrono::DateTime<chrono::Utc>)>(
        "SELECT id,mailbox_id,status,payload_raw,content_type,payload,payload_sha256,idempotency_key_hash,traceparent,tracestate,claim_count,created_at,updated_at FROM mailbox_messages ORDER BY created_at"
    ).fetch_all(pool).await?;
    let messages = rows
        .into_iter()
        .map(|r| Message {
            id: r.0,
            mailbox_id: r.1,
            status: if r.2 == "CLAIMED" {
                "UNREAD".into()
            } else {
                r.2
            },
            payload_base64: base64::engine::general_purpose::STANDARD.encode(r.3),
            content_type: r.4,
            payload: r.5,
            payload_sha256: r.6,
            idempotency_key_hash: r.7,
            traceparent: r.8,
            tracestate: r.9,
            claim_count: r.10,
            created_at: r.11,
            updated_at: r.12,
        })
        .collect();
    let archive = Archive {
        format: "promptjang-relay-one-mailbox-v1".into(),
        exported_at: chrono::Utc::now(),
        mailboxes,
        messages,
    };
    std::fs::write(output, serde_json::to_vec_pretty(&archive)?)
        .with_context(|| format!("write {}", output.display()))?;
    restrict(output)?;
    Ok(())
}

pub async fn import(pool: &SqlitePool, input: &Path) -> Result<()> {
    let archive: Archive = serde_json::from_slice(
        &std::fs::read(input).with_context(|| format!("read {}", input.display()))?,
    )?;
    if archive.format != "promptjang-relay-one-mailbox-v1" {
        bail!("unsupported mailbox archive format");
    }
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT (SELECT count(*) FROM mailboxes)+(SELECT count(*) FROM mailbox_messages)",
    )
    .fetch_one(pool)
    .await?;
    if existing != 0 {
        bail!("import target must not contain mailbox data");
    }
    let mut tx = pool.begin().await?;
    for mailbox in archive.mailboxes {
        sqlx::query("INSERT INTO mailboxes(id,name,created_at) VALUES(?1,?2,?3)")
            .bind(mailbox.id)
            .bind(mailbox.name)
            .bind(mailbox.created_at)
            .execute(&mut *tx)
            .await?;
    }
    for message in archive.messages {
        let raw = base64::engine::general_purpose::STANDARD.decode(message.payload_base64)?;
        let hash = crate::domain::secrets::hash_bytes(&raw);
        if hash != message.payload_sha256 {
            bail!("payload hash mismatch for message {}", message.id);
        }
        sqlx::query("INSERT INTO mailbox_messages(id,mailbox_id,status,payload_raw,content_type,payload,payload_sha256,idempotency_key_hash,traceparent,tracestate,claim_count,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)")
            .bind(message.id).bind(message.mailbox_id).bind(message.status).bind(raw).bind(message.content_type).bind(message.payload).bind(message.payload_sha256).bind(message.idempotency_key_hash).bind(message.traceparent).bind(message.tracestate).bind(message.claim_count).bind(message.created_at).bind(message.updated_at).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(())
}

#[cfg(unix)]
fn restrict(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(())
}
#[cfg(not(unix))]
fn restrict(_path: &Path) -> Result<()> {
    Ok(())
}
