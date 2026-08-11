use std::{error::Error, sync::Arc};

use server::{AppState, app, config::Config, migrate, repository::SqlxUserRepository};
use sqlx::mysql::MySqlPoolOptions;
use tokio::{net::TcpListener, signal};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    init_tracing();

    if let Err(error) = run().await {
        error!(error = %error, "server stopped");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    let config = Config::from_env()?;
    let pool = MySqlPoolOptions::new()
        .connect_with(config.database_options()?)
        .await?;
    migrate(&pool).await?;

    let repository = Arc::new(SqlxUserRepository::new(pool.clone()));
    let state = AppState::new(config.auth_mode, repository);

    let listener = TcpListener::bind(config.app_addr).await?;
    info!(address = %config.app_addr, "listening");
    axum::serve(listener, app(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    pool.close().await;
    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("server=info,tower_http=info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

async fn shutdown_signal() {
    if let Err(error) = signal::ctrl_c().await {
        error!(error = %error, "failed to install shutdown signal handler");
    }
}
