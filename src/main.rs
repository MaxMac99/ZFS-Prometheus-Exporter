mod cli;
mod metrics;
mod zfs;

use anyhow::{Context, Result};
use axum::{routing::get, Router};
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::info;

use cli::Args;
use metrics::handlers::{health_handler, metrics_handler};
use metrics::Metrics;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    info!("Starting ZFS Prometheus Exporter");

    // Initialize metrics
    let metrics = Arc::new(RwLock::new(Metrics::new()));

    // Build router
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
        .with_state(metrics);

    // Start server
    let addr = format!("{}:{}", args.host, args.port)
        .parse::<SocketAddr>()
        .context("Invalid address")?;

    info!("Listening on http://{}", addr);
    info!("Metrics available at http://{}/metrics", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
