//! Metrics and monitoring endpoint handlers
//!
//! Provides performance metrics and operational statistics.

use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Reply;

use crate::MetricsCollector;

use super::server::ServerState;
use super::types::MetricsResponse;

/// Handle metrics endpoint
pub async fn handle_metrics(
    state: Arc<RwLock<ServerState>>,
    _metrics_collector: Arc<MetricsCollector>,
) -> Result<impl Reply, warp::Rejection> {
    let state_guard = state.read().await;
    let uptime = Utc::now()
        .signed_duration_since(state_guard.started_at)
        .num_seconds();

    let response = MetricsResponse {
        request_count: state_guard.request_count,
        active_connections: state_guard.active_connections,
        uptime_seconds: uptime as u64,
    };

    Ok(warp::reply::json(&response))
}

#[cfg(test)]
#[path = "metrics_tests.rs"]
mod tests;
