use anyhow::{Context, Result};
use base64::Engine;
use directories::ProjectDirs;
use rand::RngCore;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Config {
    pub data_dir: PathBuf,
    pub database_path: PathBuf,
    pub bind: String,
    pub encryption_key: [u8; 32],
    pub max_payload_bytes: usize,
    pub retention_days: i64,
    pub mailbox_claim_limit: i64,
    pub update_check_enabled: bool,
}

impl Config {
    pub fn load(data_dir: Option<PathBuf>, port: u16) -> Result<Self> {
        let data_dir = data_dir
            .or_else(|| {
                ProjectDirs::from("net", "PromptJang", "Relay One")
                    .map(|p| p.data_local_dir().to_path_buf())
            })
            .context("could not determine the application data directory; use --data-dir")?;
        fs::create_dir_all(&data_dir)
            .with_context(|| format!("create data directory {}", data_dir.display()))?;
        let encryption_key = load_or_create_key(&data_dir.join("master.key"))?;
        Ok(Self {
            database_path: data_dir.join("relay-one.db"),
            data_dir,
            bind: format!("127.0.0.1:{port}"),
            encryption_key,
            max_payload_bytes: 1_048_576,
            retention_days: 30,
            mailbox_claim_limit: 100,
            update_check_enabled: std::env::var("PJ_UPDATE_CHECK_ENABLED")
                .map(|value| !matches!(value.to_ascii_lowercase().as_str(), "0" | "false" | "no"))
                .unwrap_or(true),
        })
    }
}

fn load_or_create_key(path: &Path) -> Result<[u8; 32]> {
    if path.exists() {
        let encoded = fs::read_to_string(path).context("read master.key")?;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded.trim())
            .context("master.key is not valid base64")?;
        return bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("master.key must contain exactly 32 bytes"));
    }
    let mut key = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    let encoded = base64::engine::general_purpose::STANDARD.encode(key);
    fs::write(path, encoded).context("write master.key")?;
    restrict_key_permissions(path)?;
    Ok(key)
}

#[cfg(unix)]
fn restrict_key_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).context("protect master.key")
}

#[cfg(not(unix))]
fn restrict_key_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_is_created_once_and_reused() {
        let directory =
            std::env::temp_dir().join(format!("relay-one-config-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&directory).expect("create temp directory");
        let path = directory.join("master.key");
        let first = load_or_create_key(&path).expect("create key");
        let second = load_or_create_key(&path).expect("reload key");
        assert_eq!(first, second);
        fs::remove_dir_all(directory).expect("remove temp directory");
    }
}
