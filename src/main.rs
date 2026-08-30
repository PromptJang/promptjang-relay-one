use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use promptjang_relay_one::{api, config::Config, mcp, migration, store};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

#[derive(Parser)]
#[command(version, about = "A durable local mailbox for CLI agents")]
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
    Serve {
        #[arg(long)]
        no_open: bool,
    },
    Mcp,
    Export {
        #[arg(long)]
        output: PathBuf,
    },
    Import {
        #[arg(long)]
        input: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    if matches!(cli.command, Some(Command::Mcp)) {
        return mcp::run().await;
    }
    let config = Arc::new(Config::load(cli.data_dir, cli.port)?);
    let pool = open_pool(&config).await?;
    match cli.command {
        Some(Command::Export { output }) => migration::export(&pool, &output).await,
        Some(Command::Import { input }) => migration::import(&pool, &input).await,
        command => {
            serve(
                pool,
                config,
                matches!(command, Some(Command::Serve { no_open: true })),
            )
            .await
        }
    }
}

async fn open_pool(config: &Config) -> Result<SqlitePool> {
    let options =
        SqliteConnectOptions::from_str(&format!("sqlite://{}", config.database_path.display()))?
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .busy_timeout(std::time::Duration::from_secs(5));
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(options)
        .await?;
    sqlx::migrate!().run(&pool).await?;
    sqlx::query("PRAGMA wal_autocheckpoint=1000")
        .execute(&pool)
        .await?;
    Ok(pool)
}

async fn serve(pool: SqlitePool, config: Arc<Config>, no_open: bool) -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let app = api::router(api::AppState {
        pool: pool.clone(),
        config: config.clone(),
    })
    .layer(TraceLayer::new_for_http());
    let listener = TcpListener::bind(&config.bind).await?;
    let url = format!("http://{}", config.bind);
    tracing::info!(%url, data_dir=%config.data_dir.display(), "PromptJang Relay One is ready");
    if !no_open {
        let url = url.clone();
        tokio::task::spawn_blocking(move || open::that(url))
            .await
            .context("open browser task")??;
    }
    let maintenance_pool = pool.clone();
    let retention = config.retention_days;
    let maintenance = tokio::spawn(async move {
        loop {
            if let Err(error) = store::mail::cleanup(&maintenance_pool, retention).await {
                tracing::warn!(%error, "mailbox cleanup failed");
            }
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    maintenance.abort();
    store::checkpoint(&config.database_path).await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
