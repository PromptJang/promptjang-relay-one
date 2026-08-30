mod delivery;

use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use crate::store;
use sqlx::SqlitePool;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::Instrument;

pub async fn run(pool: SqlitePool, config: Arc<Config>, shutdown: CancellationToken) {
    let mut workers = JoinSet::new();
    for worker_id in 0..config.worker_concurrency {
        let pool = pool.clone();
        let config = config.clone();
        let shutdown = shutdown.clone();
        workers.spawn(async move { delivery_loop(worker_id, pool, config, shutdown).await });
    }
    loop {
        match delivery::recover_stuck(&pool, &config)
            .instrument(tracing::info_span!("relay.recovery"))
            .await
        {
            Err(error) => tracing::error!(%error, "stuck delivery recovery failed"),
            Ok(recovered) => crate::telemetry::recovered(recovered),
        }
        match store::events::cleanup(&pool, config.retention_days)
            .instrument(tracing::info_span!("relay.retention.cleanup"))
            .await
        {
            Err(error) => tracing::error!(%error, "retention cleanup failed"),
            Ok(cleaned) => crate::telemetry::cleaned(cleaned),
        }
        match store::mail::cleanup(&pool, config.retention_days).await {
            Err(error) => tracing::error!(%error, "mailbox retention cleanup failed"),
            Ok(cleaned) => crate::telemetry::cleaned(cleaned),
        }
        if let Ok((active, _, _, _)) = store::events::summary(&pool).await {
            crate::telemetry::queue_depth(active.max(0) as u64);
        }
        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = tokio::time::sleep(Duration::from_secs(60)) => {}
        }
    }
    while workers.join_next().await.is_some() {}
}

async fn delivery_loop(
    worker_id: usize,
    pool: SqlitePool,
    config: Arc<Config>,
    shutdown: CancellationToken,
) {
    loop {
        if shutdown.is_cancelled() {
            break;
        }
        match delivery::process_one(&pool, &config).await {
            Ok(true) => continue,
            Ok(false) => tokio::select! {
                _ = shutdown.cancelled() => break,
                _ = tokio::time::sleep(Duration::from_millis(500)) => {}
            },
            Err(error) => {
                tracing::error!(%error, worker_id, "delivery worker iteration failed");
                tokio::select! {
                    _ = shutdown.cancelled() => break,
                    _ = tokio::time::sleep(Duration::from_secs(2)) => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn worker_module_exposes_no_panicking_paths() {
        // Arrange
        let source = include_str!("delivery.rs");

        // Act
        let production_panics = source
            .lines()
            .filter(|line| {
                !line.trim_start().starts_with("//")
                    && (line.contains(".unwrap()")
                        || line.contains(".expect(")
                        || line.contains("panic!"))
            })
            .count();

        // Assert
        assert_eq!(production_panics, 0);
    }
}
