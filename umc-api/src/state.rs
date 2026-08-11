use std::sync::Arc;
use tokio::sync::{broadcast, Semaphore};

use crate::config::Config;
use crate::models::ProgressEvent;

/// Shared application state injected via Actix Data.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: sqlx::PgPool,
    /// Limits simultaneous conversions
    pub conversion_semaphore: Arc<Semaphore>,
    /// Broadcast channel for SSE progress events (keyed by job_id string in message)
    pub progress_tx: broadcast::Sender<ProgressEvent>,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        // Ensure upload/output dirs exist
        tokio::fs::create_dir_all(&config.upload_dir).await?;
        tokio::fs::create_dir_all(&config.output_dir).await?;

        let db = sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .connect(&config.database_url)
            .await
            .map_err(|e| anyhow::anyhow!("DB connect failed: {e}"))?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&db)
            .await
            .map_err(|e| anyhow::anyhow!("Migration failed: {e}"))?;

        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_conversions));
        let (progress_tx, _) = broadcast::channel(1024);

        Ok(Self {
            config: Arc::new(config),
            db,
            conversion_semaphore: semaphore,
            progress_tx,
        })
    }
}
