use anyhow::{Context, Result};
use base64::Engine;
use directories::ProjectDirs;
use ipnet::IpNet;
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
    pub rate_limit_per_minute: i64,
    pub retention_days: i64,
    pub worker_concurrency: usize,
    pub delivery_timeout_seconds: u64,
    pub retry_delays_seconds: Vec<i64>,
    pub stuck_after_seconds: i64,
    pub response_body_bytes: usize,
    pub allow_private_cidrs: Vec<IpNet>,
    pub allow_insecure_http: bool,
    pub extra_ca_cert_path: Option<String>,
    pub otel_enabled: bool,
}

impl Config {
    pub fn load(data_dir: Option<PathBuf>, port: u16) -> Result<Self> {
        let data_dir = data_dir
            .or_else(|| ProjectDirs::from("net", "PromptJang", "Relay One").map(|p| p.data_local_dir().to_path_buf()))
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
            rate_limit_per_minute: 10_000,
            retention_days: 30,
            worker_concurrency: 4,
            delivery_timeout_seconds: 15,
            retry_delays_seconds: vec![60, 120, 240, 480, 960],
            stuck_after_seconds: 300,
            response_body_bytes: 10_240,
            allow_private_cidrs: vec!["127.0.0.0/8".parse()?, "::1/128".parse()?],
            allow_insecure_http: true,
            extra_ca_cert_path: None,
            otel_enabled: false,
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
        let directory = std::env::temp_dir().join(format!("relay-one-config-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&directory).expect("create temp directory");
        let path = directory.join("master.key");
        let first = load_or_create_key(&path).expect("create key");
        let second = load_or_create_key(&path).expect("reload key");
        assert_eq!(first, second);
        fs::remove_dir_all(directory).expect("remove temp directory");
    }
}
