//! Management and control endpoint handlers
//!
//! Handles administrative operations like graceful shutdown.

use std::sync::Arc;
use warp::Reply;

use crate::shutdown::ShutdownManager;

use super::types::ShutdownResponse;

/// Handle shutdown endpoint
pub async fn handle_shutdown(
    shutdown_manager: Arc<ShutdownManager>,
) -> Result<impl Reply, warp::Rejection> {
    // Request graceful shutdown (sends signal, doesn't block)
    if let Err(e) = shutdown_manager.request_shutdown().await {
        tracing::error!(error = ?e, "Failed to request shutdown");
        let response = ShutdownResponse {
            status: "error".to_string(),
        };
        return Ok(warp::reply::json(&response));
    }

    let response = ShutdownResponse {
        status: "shutdown_initiated".to_string(),
    };

    Ok(warp::reply::json(&response))
}

#[cfg(test)]
#[path = "management_tests.rs"]
mod tests;
