mod app;
mod auth;
mod config;
mod errors;
mod handlers;
mod models;
mod state;
mod worker;

use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Tracing setup
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,umc_api=debug")),
        )
        .with_target(true)
        .compact()
        .init();

    // Load config
    let config = config::Config::from_env()?;
    config.validate()?;

    tracing::info!(
        host = %config.host,
        port = config.port,
        upload_dir = %config.upload_dir,
        "UMC API starting"
    );

    // Build app state (connects DB + runs migrations)
    let state = state::AppState::new(config.clone()).await?;

    tracing::info!("Database connected and migrations applied");

    // Start HTTP server
    app::create_server(
        state,
        config.host.clone(),
        config.port,
        config.cors_origin.clone(),
    )
    .await?;

    Ok(())
}
