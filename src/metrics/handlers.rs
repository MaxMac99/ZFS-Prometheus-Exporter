use axum::extract::State;
use prometheus_client::encoding::text::encode;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use super::Metrics;

pub async fn metrics_handler(State(metrics): State<Arc<RwLock<Metrics>>>) -> String {
    let metrics = metrics.read().await;

    // Collect fresh metrics
    match metrics.collect().await {
        Ok(_) => info!("Successfully collected all metrics"),
        Err(e) => error!("Failed to collect metrics: {:?}", e),
    }

    // Encode metrics
    let mut buffer = String::new();
    encode(&mut buffer, &metrics.registry).unwrap();
    buffer
}

pub async fn health_handler() -> &'static str {
    "OK"
}
