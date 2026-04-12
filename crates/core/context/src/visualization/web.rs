// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Web Visualization Server
//!
//! **Status:** Not yet implemented. A real server would require petalTongue (or equivalent)
//! capability discovery and HTTP/WebSocket wiring from the ecosystem; those contracts are not
//! available in this crate yet. [`WebVisualizationServer::start`] therefore logs and returns
//! `Ok(())` so callers can opt in without failing the context runtime.

use super::{VisualizationEvent, VisualizationSystemConfig};
use crate::error::Result;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::debug;

/// Placeholder for a future web UI (no listener is opened until petalTongue integration exists).
#[derive(Debug)]
pub struct WebVisualizationServer {
    /// Configuration
    #[expect(dead_code, reason = "planned feature not yet wired")]
    config: Arc<VisualizationSystemConfig>,

    /// Event broadcaster
    #[expect(dead_code, reason = "planned feature not yet wired")]
    event_broadcaster: broadcast::Sender<VisualizationEvent>,
}

impl WebVisualizationServer {
    /// Create a new web visualization server
    pub async fn new(
        config: Arc<VisualizationSystemConfig>,
        event_broadcaster: broadcast::Sender<VisualizationEvent>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            event_broadcaster,
        })
    }

    /// Start the web server
    pub async fn start(&self) -> Result<()> {
        debug!(
            "Web visualization server not yet wired — awaiting petalTongue capability discovery"
        );
        Ok(())
    }

    /// Stop the web server
    pub async fn stop(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_web_visualization_server_new() {
        let config = Arc::new(VisualizationSystemConfig::default());
        let (tx, _rx) = broadcast::channel(16);
        let server = WebVisualizationServer::new(config, tx).await;
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_web_visualization_server_start_stop() {
        let config = Arc::new(VisualizationSystemConfig::default());
        let (tx, _rx) = broadcast::channel(16);
        let server = WebVisualizationServer::new(config, tx)
            .await
            .expect("should succeed");
        assert!(server.start().await.is_ok());
        assert!(server.stop().await.is_ok());
    }
}
