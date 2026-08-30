use anyhow::{Context, Result};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub async fn run() -> Result<()> {
    let base = std::env::var("PJ_ONE_URL").unwrap_or_else(|_| "http://127.0.0.1:8081".into());
    let key = std::env::var("PJ_ONE_API_KEY").context("PJ_ONE_API_KEY is required for MCP mode")?;
    let default_mailbox = std::env::var("PJ_ONE_MAILBOX").ok();
    let client = reqwest::Client::new();
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    let mut output = tokio::io::stdout();
    while let Some(line) = lines.next_line().await? {
        let request: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(error) => {
                write(&mut output, json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":error.to_string()}})).await?;
                continue;
            }
        };
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request.get("method").and_then(Value::as_str).unwrap_or_default();
        if method.starts_with("notifications/") { continue; }
        let result = match method {
            "initialize" => Ok(json!({"protocolVersion":"2025-06-18","capabilities":{"tools":{}},"serverInfo":{"name":"promptjang-relay-one","version":env!("CARGO_PKG_VERSION")}})),
            "tools/list" => Ok(json!({"tools": tool_definitions()})),
            "tools/call" => call_tool(&client, &base, &key, default_mailbox.as_deref(), request.get("params").unwrap_or(&Value::Null)).await,
            _ => Err(format!("unknown method: {method}")),
        };
        let response = match result {
            Ok(value) => json!({"jsonrpc":"2.0","id":id,"result":value}),
            Err(message) => json!({"jsonrpc":"2.0","id":id,"error":{"code":-32000,"message":message}}),
        };
        write(&mut output, response).await?;
    }
    Ok(())
}

async fn write(output: &mut tokio::io::Stdout, value: Value) -> Result<()> {
    output.write_all(serde_json::to_string(&value)?.as_bytes()).await?;
    output.write_all(b"\n").await?;
    output.flush().await?;
    Ok(())
}

fn tool_definitions() -> Vec<Value> {
    [
        ("mail_push", "Push a durable message", json!({"mailbox":{"type":"string"},"payload":{},"idempotency_key":{"type":"string"}}), vec!["payload"]),
        ("mail_claim", "Claim pending messages", json!({"mailbox":{"type":"string"},"limit":{"type":"integer"},"lease_seconds":{"type":"integer"}}), vec![]),
        ("mail_ack", "Acknowledge a claimed message", json!({"mailbox":{"type":"string"},"id":{"type":"string"},"claim_token":{"type":"string"}}), vec!["id","claim_token"]),
        ("mail_nack", "Return a claimed message", json!({"mailbox":{"type":"string"},"id":{"type":"string"},"claim_token":{"type":"string"}}), vec!["id","claim_token"]),
        ("mail_list", "List mailboxes", json!({}), vec![]),
    ].into_iter().map(|(name,description,properties,required)| json!({"name":name,"description":description,"inputSchema":{"type":"object","properties":properties,"required":required}})).collect()
}

async fn call_tool(client: &reqwest::Client, base: &str, key: &str, default_mailbox: Option<&str>, params: &Value) -> Result<Value, String> {
    let name = params.get("name").and_then(Value::as_str).ok_or("tool name is required")?;
    let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
    let mailbox = || args.get("mailbox").and_then(Value::as_str).or(default_mailbox).ok_or("mailbox is required");
    let auth = |request: reqwest::RequestBuilder| request.bearer_auth(key);
    let response = match name {
        "mail_list" => client.get(format!("{base}/api/v1/mail")).send().await,
        "mail_push" => {
            let mut request = auth(client.post(format!("{base}/v1/mail/{}/messages", mailbox()?))).json(args.get("payload").ok_or("payload is required")?);
            if let Some(value) = args.get("idempotency_key").and_then(Value::as_str) { request = request.header("Idempotency-Key", value); }
            request.send().await
        }
        "mail_claim" => auth(client.post(format!("{base}/v1/mail/{}/claim", mailbox()?))).json(&json!({"limit":args.get("limit"),"lease_seconds":args.get("lease_seconds")})).send().await,
        "mail_ack" | "mail_nack" => {
            let id = args.get("id").and_then(Value::as_str).ok_or("id is required")?;
            let token = args.get("claim_token").and_then(Value::as_str).ok_or("claim_token is required")?;
            let action = if name == "mail_ack" { "ack" } else { "nack" };
            auth(client.post(format!("{base}/v1/mail/{}/messages/{id}/{action}", mailbox()?))).json(&json!({"claim_token":token})).send().await
        }
        _ => return Err(format!("unknown tool: {name}")),
    }.map_err(|error| error.to_string())?;
    let status = response.status();
    let value = response.json::<Value>().await.map_err(|error| error.to_string())?;
    Ok(json!({"content":[{"type":"text","text":serde_json::to_string(&value).unwrap_or_default()}],"isError":!status.is_success()}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exposes_exact_mailbox_tools() {
        let names: Vec<_> = tool_definitions().iter().filter_map(|v| v["name"].as_str()).collect();
        assert_eq!(names, ["mail_push", "mail_claim", "mail_ack", "mail_nack", "mail_list"]);
    }
}
