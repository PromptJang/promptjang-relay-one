//! MCP stdio server exposing the PromptJang Relay agent mailbox as tools.
//!
//! Transport: newline-delimited JSON-RPC 2.0 on stdin/stdout per the MCP spec.
//! Logs never touch stdout; failures are reported as tool errors or stderr notes.

use promptjang_relay::domain::secrets;
use promptjang_relay::store::mail::{self, IncomingMessage};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

const PROTOCOL_VERSION: &str = "2025-06-18";
const SERVER_NAME: &str = "promptjang-relay-mcp";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
pub struct Server {
    pool: PgPool,
    default_mailbox: Option<String>,
}

fn error_object(code: i64, message: &str) -> Value {
    json!({ "code": code, "message": message })
}

fn mailbox_schema(default_mailbox: &Option<String>) -> Value {
    let description = match default_mailbox {
        Some(name) => format!("Mailbox name (default: {name})"),
        None => "Mailbox name".to_string(),
    };
    json!({ "type": "string", "description": description })
}

pub fn tool_definitions(default_mailbox: &Option<String>) -> Vec<Value> {
    vec![
        json!({
            "name": "mail_push",
            "description": "Push a message into a PromptJang Relay agent mailbox for a consumer to claim later.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mailbox": mailbox_schema(default_mailbox),
                    "payload": { "type": ["object", "array", "string"], "description": "Message body; objects and arrays are stored as JSON" },
                    "idempotency_key": { "type": "string", "description": "Optional key; the same key returns the original message" },
                },
                "required": ["payload"],
            }
        }),
        json!({
            "name": "mail_claim",
            "description": "Claim (pull) pending messages from a PromptJang Relay agent mailbox. Each claimed message includes a claim_token used to acknowledge or requeue it.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mailbox": mailbox_schema(default_mailbox),
                    "limit": { "type": "integer", "minimum": 1, "maximum": 100, "description": "Maximum messages to claim (default 10)" },
                    "lease_seconds": { "type": "integer", "minimum": 30, "maximum": 3600, "description": "Seconds before an unacknowledged claim expires back to the queue (default 300)" },
                },
            }
        }),
        json!({
            "name": "mail_ack",
            "description": "Acknowledge a claimed mailbox message so it is never delivered again.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mailbox": mailbox_schema(default_mailbox),
                    "id": { "type": "string", "description": "Message id from mail_claim" },
                    "claim_token": { "type": "string", "description": "Claim token from mail_claim" },
                },
                "required": ["id", "claim_token"],
            }
        }),
        json!({
            "name": "mail_nack",
            "description": "Requeue a claimed mailbox message immediately, for example after a processing failure.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mailbox": mailbox_schema(default_mailbox),
                    "id": { "type": "string", "description": "Message id from mail_claim" },
                    "claim_token": { "type": "string", "description": "Claim token from mail_claim" },
                },
                "required": ["id", "claim_token"],
            }
        }),
        json!({
            "name": "mail_list",
            "description": "List PromptJang Relay agent mailboxes with unread, claimed, and acknowledged counts.",
            "inputSchema": { "type": "object", "properties": {} },
        }),
    ]
}

fn resolve_mailbox<'a>(
    default_mailbox: &'a Option<String>,
    arguments: &'a Value,
) -> Result<&'a str, String> {
    if let Some(name) = arguments.get("mailbox").and_then(Value::as_str) {
        return Ok(name);
    }
    default_mailbox
        .as_deref()
        .ok_or_else(|| "mailbox is required when RELAY_MAILBOX is not configured".to_string())
}

fn text_content(value: &Value) -> Value {
    let text = serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string());
    json!({ "content": [{ "type": "text", "text": text }], "isError": false })
}

fn error_content(message: &str) -> Value {
    json!({ "content": [{ "type": "text", "text": message }], "isError": true })
}

impl Server {
    pub fn new(pool: PgPool, default_mailbox: Option<String>) -> Self {
        Self {
            pool,
            default_mailbox,
        }
    }

    pub async fn call_tool(&self, name: &str, arguments: &Value) -> Result<Value, String> {
        match name {
            "mail_push" => self.mail_push(arguments).await,
            "mail_claim" => self.mail_claim(arguments).await,
            "mail_ack" => self.mail_complete(arguments, true).await,
            "mail_nack" => self.mail_complete(arguments, false).await,
            "mail_list" => self.mail_list().await,
            other => Err(format!("unknown tool: {other}")),
        }
    }

    fn mailbox(&self, arguments: &Value) -> Result<String, String> {
        resolve_mailbox(&self.default_mailbox, arguments).map(str::to_string)
    }

    async fn mail_push(&self, arguments: &Value) -> Result<Value, String> {
        let name = self.mailbox(arguments)?;
        if let Err(error) = mail::validate_mailbox_name(&name) {
            return Err(error.to_string());
        }
        let payload = arguments
            .get("payload")
            .cloned()
            .ok_or_else(|| "payload is required".to_string())?;
        let (payload_raw, parsed, content_type) = match &payload {
            Value::String(text) => (text.clone().into_bytes(), None, "text/plain".to_string()),
            value => (
                serde_json::to_vec(value).map_err(|error| error.to_string())?,
                Some(value.clone()),
                "application/json".to_string(),
            ),
        };
        let idempotency_key_hash = arguments
            .get("idempotency_key")
            .and_then(Value::as_str)
            .map(secrets::hash_secret);
        let outcome = mail::push(
            &self.pool,
            &name,
            IncomingMessage {
                payload_raw,
                payload: parsed,
                content_type,
                payload_sha256: secrets::hash_bytes(payload.to_string().as_bytes()),
                idempotency_key_hash,
                traceparent: None,
                tracestate: None,
            },
        )
        .await
        .map_err(|error| error.to_string())?;
        match outcome {
            mail::MailPushOutcome::Created { id } => {
                Ok(json!({ "id": id, "mailbox": name, "status": "UNREAD" }))
            }
            mail::MailPushOutcome::IdempotentReplay { id, status } => Ok(json!({
                "id": id, "mailbox": name, "status": status, "idempotent_replay": true,
            })),
        }
    }

    async fn mail_claim(&self, arguments: &Value) -> Result<Value, String> {
        let name = self.mailbox(arguments)?;
        let limit = mail::normalize_claim_limit(arguments.get("limit").and_then(Value::as_i64));
        let lease = mail::normalize_lease(arguments.get("lease_seconds").and_then(Value::as_i64));
        let messages = mail::claim(&self.pool, &name, limit, lease.seconds)
            .await
            .map_err(|error| error.to_string())?;
        let claimed: Vec<Value> = messages
            .iter()
            .map(|message| {
                json!({
                    "id": message.id,
                    "claim_token": message.claim_token,
                    "payload": String::from_utf8_lossy(&message.payload_raw),
                    "payload_json": message.payload,
                    "claim_count": message.claim_count,
                    "created_at": message.created_at,
                })
            })
            .collect();
        Ok(json!({ "mailbox": name, "lease_seconds": lease.seconds, "messages": claimed }))
    }

    async fn mail_complete(&self, arguments: &Value, acknowledge: bool) -> Result<Value, String> {
        let name = self.mailbox(arguments)?;
        let id = arguments
            .get("id")
            .and_then(Value::as_str)
            .and_then(|value| Uuid::parse_str(value).ok())
            .ok_or_else(|| "id must be a message id from mail_claim".to_string())?;
        let claim_token = arguments
            .get("claim_token")
            .and_then(Value::as_str)
            .ok_or_else(|| "claim_token is required".to_string())?;
        let ok = mail::acknowledge(&self.pool, &name, id, claim_token, acknowledge)
            .await
            .map_err(|error| error.to_string())?;
        if !ok {
            return Err(
                "message is not claimed with this token (already completed or lease expired)"
                    .to_string(),
            );
        }
        let status = if acknowledge {
            "ACKNOWLEDGED"
        } else {
            "UNREAD"
        };
        Ok(json!({ "id": id, "mailbox": name, "status": status }))
    }

    async fn mail_list(&self) -> Result<Value, String> {
        let mailboxes = mail::list_mailboxes(&self.pool)
            .await
            .map_err(|error| error.to_string())?;
        Ok(json!({ "mailboxes": mailboxes }))
    }
}

pub async fn handle_request(server: &Server, request: &Value) -> Option<Value> {
    let id = request.get("id")?;
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");
    let result = match method {
        "initialize" => {
            let requested = request
                .pointer("/params/protocolVersion")
                .and_then(Value::as_str)
                .unwrap_or(PROTOCOL_VERSION);
            Ok(json!({
                "protocolVersion": requested,
                "capabilities": { "tools": {} },
                "serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION },
            }))
        }
        "tools/list" => Ok(json!({ "tools": tool_definitions(&server.default_mailbox) })),
        "tools/call" => {
            let name = request
                .pointer("/params/name")
                .and_then(Value::as_str)
                .unwrap_or("");
            let empty = Value::Object(serde_json::Map::new());
            let arguments = request.pointer("/params/arguments").unwrap_or(&empty);
            match server.call_tool(name, arguments).await {
                Ok(value) => Ok(text_content(&value)),
                Err(message) => Ok(error_content(&message)),
            }
        }
        "ping" => Ok(json!({})),
        other => Err(error_object(-32601, &format!("method not found: {other}"))),
    };
    Some(match result {
        Ok(value) => json!({ "jsonrpc": "2.0", "id": id, "result": value }),
        Err(error) => json!({ "jsonrpc": "2.0", "id": id, "error": error }),
    })
}

async fn write_line(
    stdout: &mut (impl tokio::io::AsyncWrite + Unpin),
    line: &Value,
) -> std::io::Result<()> {
    use tokio::io::AsyncWriteExt;
    let text = serde_json::to_string(line).unwrap_or_else(|_| "{}".to_string());
    stdout.write_all(text.as_bytes()).await?;
    stdout.write_all(b"\n").await?;
    stdout.flush().await
}

#[tokio::main]
async fn main() {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("DATABASE_URL is required");
        std::process::exit(2);
    };
    let default_mailbox = std::env::var("RELAY_MAILBOX")
        .ok()
        .filter(|name| !name.is_empty());
    let pool = match PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("cannot connect to PostgreSQL: {error}");
            std::process::exit(2);
        }
    };
    let server = Server::new(pool, default_mailbox);
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
    let mut stdout = tokio::io::stdout();
    let mut line = String::new();
    use tokio::io::AsyncBufReadExt;
    loop {
        line.clear();
        match stdin.read_line(&mut line).await {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let request = match serde_json::from_str::<Value>(line.trim()) {
            Ok(value) => value,
            Err(_) => {
                let response = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": error_object(-32700, "parse error"),
                });
                if write_line(&mut stdout, &response).await.is_err() {
                    break;
                }
                continue;
            }
        };
        if let Some(response) = handle_request(&server, &request).await
            && write_line(&mut stdout, &response).await.is_err()
        {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server() -> Server {
        Server::new(
            PgPool::connect_lazy("postgres://test:test@localhost/test").expect("lazy pool"),
            Some("agent-tasks".to_string()),
        )
    }

    #[tokio::test]
    async fn initialize_echoes_the_client_protocol_version() {
        // Arrange
        let request = json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": { "protocolVersion": "2025-03-26" }
        });

        // Act
        let response = handle_request(&server(), &request).await;

        // Assert
        let response = response.expect("initialize expects a response");
        assert_eq!(
            response.pointer("/result/protocolVersion"),
            Some(&json!("2025-03-26"))
        );
        assert_eq!(
            response.pointer("/result/serverInfo/name"),
            Some(&json!(SERVER_NAME))
        );
    }

    #[tokio::test]
    async fn tools_list_exposes_the_five_mailbox_tools() {
        // Arrange
        let request = json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" });

        // Act
        let response = handle_request(&server(), &request).await;

        // Assert
        let response = response.expect("tools/list expects a response");
        let tools = response
            .pointer("/result/tools")
            .and_then(Value::as_array)
            .expect("tools array");
        let names: Vec<_> = tools
            .iter()
            .filter_map(|tool| tool.get("name").and_then(Value::as_str))
            .collect();
        assert_eq!(
            names,
            [
                "mail_push",
                "mail_claim",
                "mail_ack",
                "mail_nack",
                "mail_list"
            ]
        );
    }

    #[tokio::test]
    async fn notifications_produce_no_response() {
        // Arrange
        let notification = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });

        // Act
        let response = handle_request(&server(), &notification).await;

        // Assert
        assert_eq!(response, None);
    }

    #[tokio::test]
    async fn unknown_methods_return_method_not_found() {
        // Arrange
        let request = json!({ "jsonrpc": "2.0", "id": "abc", "method": "resources/list" });

        // Act
        let response = handle_request(&server(), &request).await;

        // Assert
        let response = response.expect("unknown method expects an error response");
        assert_eq!(response.pointer("/error/code"), Some(&json!(-32601)));
    }

    #[tokio::test]
    async fn unknown_tool_calls_return_tool_errors() {
        // Arrange
        let request = json!({
            "jsonrpc": "2.0", "id": 3, "method": "tools/call",
            "params": { "name": "nope", "arguments": {} }
        });

        // Act
        let response = handle_request(&server(), &request).await;

        // Assert
        let response = response.expect("tools/call expects a response");
        assert_eq!(response.pointer("/result/isError"), Some(&json!(true)));
        assert!(
            response
                .pointer("/result/content/0/text")
                .and_then(Value::as_str)
                .expect("error text")
                .contains("unknown tool")
        );
    }
}
