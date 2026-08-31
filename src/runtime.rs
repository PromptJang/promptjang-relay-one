use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::{api, config::Config, store};

pub struct PreparedServer {
    pool: SqlitePool,
    config: Arc<Config>,
    listener: TcpListener,
    url: String,
}

impl PreparedServer {
    pub fn url(&self) -> &str {
        &self.url
    }
}

pub async fn open_pool(config: &Config) -> Result<SqlitePool> {
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

pub async fn prepare(config: Arc<Config>) -> Result<PreparedServer> {
    let pool = open_pool(&config).await?;
    let listener = TcpListener::bind(&config.bind).await?;
    let address = listener.local_addr()?;
    let url = format!("http://{address}");
    tracing::info!(%url, data_dir=%config.data_dir.display(), "PromptJang Relay One is ready");
    Ok(PreparedServer {
        pool,
        config,
        listener,
        url,
    })
}

pub async fn serve<F>(prepared: PreparedServer, shutdown: F) -> Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    let PreparedServer {
        pool,
        config,
        listener,
        ..
    } = prepared;
    let app = api::router(api::AppState {
        pool: pool.clone(),
        config: config.clone(),
    })
    .layer(TraceLayer::new_for_http());
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
        .with_graceful_shutdown(shutdown)
        .await?;
    maintenance.abort();
    pool.close().await;
    store::checkpoint(&config.database_path).await?;
    Ok(())
}

pub async fn serve_cli(config: Arc<Config>, open_browser: bool) -> Result<()> {
    let prepared = prepare(config).await?;
    if open_browser {
        let url = prepared.url().to_string();
        tokio::task::spawn_blocking(move || open::that(url))
            .await
            .context("open browser task")??;
    }
    serve(prepared, async {
        let _ = tokio::signal::ctrl_c().await;
    })
    .await
}

pub fn init_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}
