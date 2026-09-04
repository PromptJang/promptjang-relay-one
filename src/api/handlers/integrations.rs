use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use serde_json::json;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::api::AppState;
use crate::api::error::ApiResult;
use crate::domain::DomainError;
use crate::store;

const SERVER_NAME: &str = "promptjang";

#[derive(Clone, Copy, Deserialize)]
pub enum McpClient {
    #[serde(rename = "claude-desktop")]
    ClaudeDesktop,
    #[serde(rename = "claude-code")]
    Claude,
    #[serde(rename = "codex")]
    Codex,
    #[serde(rename = "opencode")]
    Opencode,
}

impl McpClient {
    fn id(self) -> &'static str {
        match self {
            Self::ClaudeDesktop => "claude-desktop",
            Self::Claude => "claude-code",
            Self::Codex => "codex",
            Self::Opencode => "opencode",
        }
    }

    fn command(self) -> &'static str {
        match self {
            Self::ClaudeDesktop => "claude-desktop",
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Opencode => "opencode",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::ClaudeDesktop => "Claude Desktop",
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
            Self::Opencode => "OpenCode",
        }
    }
}

#[derive(Deserialize)]
pub struct InstallInput {
    client: McpClient,
    key_id: Uuid,
}

#[derive(Deserialize)]
pub struct DiagnoseInput {
    client: McpClient,
    key_id: Uuid,
}

pub async fn mcp_status(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    let executable = std::env::current_exe()
        .map_err(|error| DomainError::internal(format!("resolve Relay One executable: {error}")))?;
    let installations = store::integrations::list(&state.pool).await?;
    let clients = [
        McpClient::ClaudeDesktop,
        McpClient::Claude,
        McpClient::Codex,
        McpClient::Opencode,
    ]
    .into_iter()
    .map(|client| {
        let command = match client {
            McpClient::ClaudeDesktop => claude_desktop_config_path(),
            _ => find_command(client.command()),
        };
        let installation = installations
            .iter()
            .find(|installation| installation.client == client.id());
        json!({
            "id": client.id(),
            "label": client.label(),
            "available": command.is_some(),
            "command_path": command,
            "configured": installation.is_some(),
            "key_id": installation.map(|value| value.key_id),
            "configured_at": installation.map(|value| value.configured_at),
            "adapter_verified_at": installation.and_then(|value| value.adapter_verified_at),
            "last_activity_at": installation.and_then(|value| value.last_activity_at),
        })
    })
    .collect::<Vec<_>>();
    Ok(Json(json!({
        "server_name": SERVER_NAME,
        "relay_url": format!("http://{}", state.config.bind),
        "relay_executable": executable,
        "clients": clients,
        "skill_install_command": "npx --yes skills add PromptJang/promptjang-relay-skill --skill promptjang -y"
    })))
}

pub async fn install_mcp(
    State(state): State<AppState>,
    Json(input): Json<InstallInput>,
) -> ApiResult<Json<serde_json::Value>> {
    let key = store::keys::reveal(&state.pool, &state.config.encryption_key, input.key_id).await?;
    let executable = std::env::current_exe()
        .map_err(|error| DomainError::internal(format!("resolve Relay One executable: {error}")))?;
    let relay_url = format!("http://{}", state.config.bind);
    let client = input.client;
    let result = tokio::task::spawn_blocking(move || match client {
        McpClient::ClaudeDesktop => install_claude_desktop(&executable, &relay_url, &key),
        _ => {
            let client_command = find_command(client.command()).ok_or_else(|| {
                DomainError::bad_request(format!(
                    "{} CLI was not found. Install it, restart Relay One, then try again.",
                    client.label()
                ))
            })?;
            install_for_client(client, &client_command, &executable, &relay_url, &key)
        }
    })
    .await
    .map_err(|error| DomainError::internal(format!("MCP installer task failed: {error}")))??;
    store::integrations::configured(&state.pool, input.client.id(), input.key_id).await?;
    Ok(Json(json!({
        "installed": true,
        "client": input.client.id(),
        "server_name": SERVER_NAME,
        "verification": result
    })))
}

pub async fn diagnose_mcp(
    State(state): State<AppState>,
    Json(input): Json<DiagnoseInput>,
) -> ApiResult<Json<serde_json::Value>> {
    let key = store::keys::reveal(&state.pool, &state.config.encryption_key, input.key_id).await?;
    let executable = std::env::current_exe()
        .map_err(|error| DomainError::internal(format!("resolve Relay One executable: {error}")))?;
    verify_mcp_adapter(&executable, &format!("http://{}", state.config.bind), &key).await?;
    if !store::integrations::adapter_verified(&state.pool, input.client.id(), input.key_id).await? {
        return Err(DomainError::conflict(
            "install MCP for this client and API key before running diagnostics",
        )
        .into());
    }
    Ok(Json(json!({
        "client": input.client.id(),
        "adapter_verified": true,
        "message": "The MCP process started, completed a handshake, listed all mailbox tools, and authenticated with Relay One."
    })))
}

async fn verify_mcp_adapter(
    executable: &Path,
    relay_url: &str,
    key: &str,
) -> Result<(), DomainError> {
    let mut child = tokio::process::Command::new(executable)
        .arg("mcp")
        .env("PJ_ONE_URL", relay_url)
        .env("PJ_ONE_API_KEY", key)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|error| DomainError::internal(format!("start MCP adapter: {error}")))?;
    let requests = concat!(
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\n",
        "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
        "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"mail_list\",\"arguments\":{}}}\n"
    );
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| DomainError::internal("MCP adapter stdin was unavailable"))?;
    stdin
        .write_all(requests.as_bytes())
        .await
        .map_err(|error| DomainError::internal(format!("write MCP diagnostic: {error}")))?;
    drop(stdin);
    let output = tokio::time::timeout(Duration::from_secs(5), child.wait_with_output())
        .await
        .map_err(|_| DomainError::internal("MCP adapter diagnostic timed out"))?
        .map_err(|error| DomainError::internal(format!("wait for MCP adapter: {error}")))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).replace(key, "[redacted]");
        return Err(DomainError::internal(format!(
            "MCP adapter exited unsuccessfully: {}",
            detail.trim()
        )));
    }
    let responses = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(serde_json::from_str::<serde_json::Value>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| DomainError::internal(format!("parse MCP diagnostic: {error}")))?;
    if responses.len() != 3
        || responses
            .iter()
            .any(|response| response.get("error").is_some())
    {
        return Err(DomainError::internal(
            "MCP adapter did not complete the expected handshake",
        ));
    }
    let tool_names = responses[1]["result"]["tools"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|tool| tool["name"].as_str())
        .collect::<Vec<_>>();
    if tool_names
        != [
            "mail_push",
            "mail_claim",
            "mail_ack",
            "mail_nack",
            "mail_list",
        ]
    {
        return Err(DomainError::internal(
            "MCP adapter returned an unexpected tool contract",
        ));
    }
    Ok(())
}

fn install_for_client(
    client: McpClient,
    client_command: &Path,
    relay_executable: &Path,
    relay_url: &str,
    key: &str,
) -> Result<String, DomainError> {
    let relay_executable = relay_executable.to_string_lossy().to_string();
    let url_env = format!("PJ_ONE_URL={relay_url}");
    let key_env = format!("PJ_ONE_API_KEY={key}");
    let client_env = format!("PJ_ONE_CLIENT={}", client.id());

    let (args, verification): (Vec<String>, String) = match client {
        McpClient::ClaudeDesktop => unreachable!("Claude Desktop uses its JSON configuration"),
        McpClient::Claude => (
            vec![
                "mcp".into(),
                "add".into(),
                "--scope".into(),
                "user".into(),
                "--transport".into(),
                "stdio".into(),
                SERVER_NAME.into(),
                "--env".into(),
                url_env,
                "--env".into(),
                key_env,
                "--env".into(),
                client_env,
                "--".into(),
                relay_executable,
                "mcp".into(),
            ],
            "Run `claude mcp get promptjang`, then restart Claude Code if it was already open."
                .into(),
        ),
        McpClient::Codex => (
            vec![
                "mcp".into(),
                "add".into(),
                SERVER_NAME.into(),
                "--env".into(),
                url_env,
                "--env".into(),
                key_env,
                "--env".into(),
                client_env,
                "--".into(),
                relay_executable,
                "mcp".into(),
            ],
            "Restart Codex, then ask it to list PromptJang mailboxes.".into(),
        ),
        McpClient::Opencode => (
            vec![
                "mcp".into(),
                "add".into(),
                SERVER_NAME.into(),
                "--env".into(),
                url_env,
                "--env".into(),
                key_env,
                "--env".into(),
                client_env,
                "--".into(),
                relay_executable,
                "mcp".into(),
            ],
            "Run `opencode mcp list`, then restart OpenCode if it was already open.".into(),
        ),
    };

    if matches!(client, McpClient::Claude | McpClient::Codex) {
        let remove_args = match client {
            McpClient::Claude => vec!["mcp", "remove", SERVER_NAME],
            McpClient::Codex => vec!["mcp", "remove", SERVER_NAME],
            _ => unreachable!(),
        };
        let _ = Command::new(client_command).args(remove_args).output();
    }

    let output = Command::new(client_command)
        .args(&args)
        .output()
        .map_err(|error| DomainError::internal(format!("start {} CLI: {error}", client.label())))?;
    if output.status.success() {
        return Ok(verification);
    }
    let stderr = String::from_utf8_lossy(&output.stderr).replace(key, "[redacted]");
    let stdout = String::from_utf8_lossy(&output.stdout).replace(key, "[redacted]");
    let detail = [stderr.trim(), stdout.trim()]
        .into_iter()
        .find(|value| !value.is_empty())
        .unwrap_or("client CLI returned an error");
    Err(DomainError::bad_request(format!(
        "{} setup failed: {detail}",
        client.label()
    )))
}

fn install_claude_desktop(
    relay_executable: &Path,
    relay_url: &str,
    key: &str,
) -> Result<String, DomainError> {
    let path = claude_desktop_config_path().ok_or_else(|| {
        DomainError::bad_request("Claude Desktop configuration directory was not found")
    })?;
    install_claude_desktop_at(&path, relay_executable, relay_url, key)?;
    Ok("Quit and reopen Claude Desktop, then open MCP settings and confirm PromptJang is connected.".into())
}

fn install_claude_desktop_at(
    path: &Path,
    relay_executable: &Path,
    relay_url: &str,
    key: &str,
) -> Result<(), DomainError> {
    let mut root = if path.exists() {
        let contents = std::fs::read_to_string(path).map_err(|error| {
            DomainError::internal(format!("read Claude Desktop config: {error}"))
        })?;
        serde_json::from_str::<serde_json::Value>(&contents).map_err(|error| {
            DomainError::bad_request(format!("Claude Desktop config is not valid JSON: {error}"))
        })?
    } else {
        json!({})
    };
    let object = root.as_object_mut().ok_or_else(|| {
        DomainError::bad_request("Claude Desktop config must contain a JSON object")
    })?;
    let servers = object
        .entry("mcpServers")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| DomainError::bad_request("Claude Desktop mcpServers must be an object"))?;
    servers.insert(
        SERVER_NAME.into(),
        json!({
            "command": relay_executable.to_string_lossy(),
            "args": ["mcp"],
            "env": {"PJ_ONE_URL":relay_url,"PJ_ONE_API_KEY":key,"PJ_ONE_CLIENT":"claude-desktop"}
        }),
    );
    let parent = path
        .parent()
        .ok_or_else(|| DomainError::internal("invalid Claude config path"))?;
    std::fs::create_dir_all(parent).map_err(|error| {
        DomainError::internal(format!("create Claude config directory: {error}"))
    })?;
    let temporary = path.with_extension("json.tmp");
    let encoded = serde_json::to_vec_pretty(&root)
        .map_err(|error| DomainError::internal(format!("encode Claude Desktop config: {error}")))?;
    std::fs::write(&temporary, encoded)
        .map_err(|error| DomainError::internal(format!("write Claude Desktop config: {error}")))?;
    protect_config_file(&temporary)?;
    std::fs::rename(&temporary, path).map_err(|error| {
        DomainError::internal(format!("replace Claude Desktop config: {error}"))
    })?;
    Ok(())
}

#[cfg(unix)]
fn protect_config_file(path: &Path) -> Result<(), DomainError> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .map_err(|error| DomainError::internal(format!("protect MCP client config: {error}")))
}

#[cfg(not(unix))]
fn protect_config_file(_path: &Path) -> Result<(), DomainError> {
    Ok(())
}

fn claude_desktop_config_path() -> Option<PathBuf> {
    let base = directories::BaseDirs::new()?;
    #[cfg(target_os = "macos")]
    return Some(
        base.home_dir()
            .join("Library/Application Support/Claude/claude_desktop_config.json"),
    );
    #[cfg(target_os = "windows")]
    return Some(base.config_dir().join("Claude/claude_desktop_config.json"));
    #[cfg(all(unix, not(target_os = "macos")))]
    return Some(
        base.home_dir()
            .join(".config/Claude/claude_desktop_config.json"),
    );
}

fn find_command(name: &str) -> Option<PathBuf> {
    let direct = PathBuf::from(name);
    if direct.components().count() > 1 && direct.is_file() {
        return Some(direct);
    }
    if let Some(path) = std::env::var_os("PATH") {
        for directory in std::env::split_paths(&path) {
            if let Some(found) = executable_in(&directory, name) {
                return Some(found);
            }
        }
    }
    if let Some(home) = directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf()) {
        for directory in [
            home.join(".local/bin"),
            home.join(".opencode/bin"),
            home.join(".cargo/bin"),
        ] {
            if let Some(found) = executable_in(&directory, name) {
                return Some(found);
            }
        }
    }
    None
}

fn executable_in(directory: &Path, name: &str) -> Option<PathBuf> {
    let candidate = directory.join(name);
    if candidate.is_file() {
        return Some(candidate);
    }
    #[cfg(windows)]
    {
        let candidate = directory.join(format!("{name}.exe"));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_an_executable_in_an_explicit_directory() {
        let directory = std::env::temp_dir().join(format!("relay-one-cli-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&directory).expect("create temp directory");
        let executable = directory.join("agent-cli");
        std::fs::write(&executable, b"test").expect("write fake executable");
        assert_eq!(executable_in(&directory, "agent-cli"), Some(executable));
        std::fs::remove_dir_all(directory).expect("remove temp directory");
    }

    #[test]
    fn client_ids_match_the_public_api_contract() {
        assert_eq!(McpClient::ClaudeDesktop.id(), "claude-desktop");
        assert_eq!(McpClient::Claude.id(), "claude-code");
        assert_eq!(McpClient::Codex.id(), "codex");
        assert_eq!(McpClient::Opencode.id(), "opencode");
    }

    #[test]
    fn claude_desktop_setup_replaces_only_promptjang_and_has_no_default_mailbox() {
        let directory = std::env::temp_dir().join(format!("relay-one-claude-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&directory).expect("create temp directory");
        let path = directory.join("claude_desktop_config.json");
        std::fs::write(&path, r#"{"theme":"dark","mcpServers":{"other":{"command":"other"},"promptjang":{"command":"broken"}}}"#).expect("write source config");

        install_claude_desktop_at(
            &path,
            Path::new("/Applications/PromptJang Relay One"),
            "http://127.0.0.1:8081",
            "pj_one_test",
        )
        .expect("install Claude Desktop config");

        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).expect("read config"))
                .expect("parse config");
        assert_eq!(value["theme"], "dark");
        assert_eq!(value["mcpServers"]["other"]["command"], "other");
        assert_eq!(
            value["mcpServers"]["promptjang"]["command"],
            "/Applications/PromptJang Relay One"
        );
        assert!(
            value["mcpServers"]["promptjang"]["env"]
                .get("PJ_ONE_MAILBOX")
                .is_none()
        );
        assert_eq!(
            value["mcpServers"]["promptjang"]["env"]["PJ_ONE_CLIENT"],
            "claude-desktop"
        );
        std::fs::remove_dir_all(directory).expect("remove temp directory");
    }
}
