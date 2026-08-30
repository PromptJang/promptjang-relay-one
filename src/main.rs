use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use promptjang_relay_one::{api, config::Config, mcp, migration, store, updater, worker};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower_http::trace::TraceLayer;

#[derive(Parser)]
#[command(version, about = "Durable delivery for webhooks and agents, in one local app")]
struct Cli {
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,
    #[arg(long, global = true, default_value_t = 8081)]
    port: u16,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Serve { #[arg(long)] no_open: bool },
    Mcp,
    Migrate { #[command(subcommand)] direction: migration::MigrationCommand },
    Update { #[command(subcommand)] action: updater::UpdateCommand },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Mcp) => return mcp::run().await,
        Some(Command::Migrate { direction }) => return migration::run(direction, cli.data_dir).await,
        Some(Command::Update { action }) => return updater::run(action).await,
        command => {
            let no_open = matches!(command, Some(Command::Serve { no_open: true }));
            serve(cli.data_dir, cli.port, no_open).await
        }
    }
}

async fn serve(data_dir: Option<PathBuf>, port: u16, no_open: bool) -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let config = Arc::new(Config::load(data_dir, port)?);
    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", config.database_path.display()))?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Full)
        .busy_timeout(std::time::Duration::from_secs(5));
    let pool = SqlitePoolOptions::new().max_connections(8).connect_with(options).await?;
    sqlx::migrate!().run(&pool).await?;
    sqlx::query("PRAGMA wal_autocheckpoint=1000").execute(&pool).await?;

    let shutdown = CancellationToken::new();
    let worker_task = tokio::spawn(worker::run(pool.clone(), config.clone(), shutdown.clone()));
    let app = api::router(api::AppState { pool, config: config.clone() }).layer(TraceLayer::new_for_http());
    let listener = TcpListener::bind(&config.bind).await?;
    let url = format!("http://{}", config.bind);
    tracing::info!(%url, data_dir=%config.data_dir.display(), "PromptJang Relay One is ready");
    if !no_open {
        let url = url.clone();
        tokio::task::spawn_blocking(move || open::that(url)).await.context("open browser task")??;
    }
    axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await?;
    shutdown.cancel();
    let _ = tokio::time::timeout(std::time::Duration::from_secs(20), worker_task).await;
    store::checkpoint(&config.database_path).await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
