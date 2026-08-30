use anyhow::{Context, Result, bail};
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum MigrationCommand {
    FromRelay {
        #[arg(long, env = "DATABASE_URL")]
        database_url: String,
        #[arg(long)]
        relay_encryption_key_file: PathBuf,
        #[arg(long)]
        credential_map: PathBuf,
    },
    ToRelay {
        #[arg(long, env = "DATABASE_URL")]
        database_url: String,
        #[arg(long)]
        relay_encryption_key_file: PathBuf,
        #[arg(long)]
        credential_map: PathBuf,
    },
}

pub async fn run(command: MigrationCommand, data_dir: Option<PathBuf>) -> Result<()> {
    let data_dir = data_dir.context("--data-dir is required for an offline migration")?;
    if !data_dir.is_absolute() {
        bail!("--data-dir must be an absolute path");
    }
    match command {
        MigrationCommand::FromRelay { database_url, relay_encryption_key_file, credential_map } => {
            crate::migration::postgres::from_relay(&database_url, &relay_encryption_key_file, &data_dir, &credential_map).await
        }
        MigrationCommand::ToRelay { database_url, relay_encryption_key_file, credential_map } => {
            crate::migration::postgres::to_relay(&data_dir, &database_url, &relay_encryption_key_file, &credential_map).await
        }
    }
}

mod postgres {
    use super::*;

    pub async fn from_relay(_url: &str, _source_key: &std::path::Path, _data_dir: &std::path::Path, _map: &std::path::Path) -> Result<()> {
        bail!("PostgreSQL to Relay One migration is not enabled in this build")
    }

    pub async fn to_relay(_data_dir: &std::path::Path, _url: &str, _target_key: &std::path::Path, _map: &std::path::Path) -> Result<()> {
        bail!("Relay One to PostgreSQL migration is not enabled in this build")
    }
}
