// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Context Visualization System
//!
//! This module provides comprehensive visualization capabilities for the Context Management System.
//! It includes visualization of context state, rule impact, metrics, and interactive control interfaces.

mod controllers;
mod interactive;
mod manager;
#[cfg(test)]
mod manager_tests;
mod metrics;
mod renderers;
mod types;
mod web;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};

use crate::error::Result;

// Re-export public types
pub use controllers::ContextController;
pub use interactive::{InteractiveSession, InteractiveVisualization};
pub use manager::VisualizationManager;
pub use metrics::VisualizationMetrics;
pub use renderers::{HtmlRenderer, JsonRenderer, MarkdownRenderer, TerminalRenderer};
pub use types::{
    VisualizationConfig, VisualizationRequest, VisualizationResponse, VisualizationType,
};
pub use web::WebVisualizationServer;

/// Visualization system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationSystemConfig {
    /// Enable JSON rendering
    pub enable_json: bool,

    /// Enable terminal rendering
    pub enable_terminal: bool,

    /// Enable HTML rendering
    pub enable_html: bool,

    /// Enable markdown rendering
    pub enable_markdown: bool,

    /// Enable interactive visualization
    pub enable_interactive: bool,

    /// Enable web interface
    pub enable_web: bool,

    /// Web server port
    pub web_port: u16,

    /// Update interval for real-time visualization
    pub update_interval: Duration,

    /// Maximum history entries to keep
    pub max_history: usize,

    /// Enable metrics collection
    pub enable_metrics: bool,
}

impl Default for VisualizationSystemConfig {
    fn default() -> Self {
        Self {
            enable_json: true,
            enable_terminal: true,
            enable_html: true,
            enable_markdown: true,
            enable_interactive: true,
            enable_web: true,
            web_port: 8080,
            update_interval: Duration::from_secs(1),
            max_history: 1000,
            enable_metrics: true,
        }
    }
}

/// Visualization system for context management
#[derive(Debug)]
pub struct VisualizationSystem {
    /// Configuration
    #[allow(dead_code)] // Config stored for future system reconfiguration
    config: Arc<VisualizationSystemConfig>,

    /// Visualization manager
    manager: Arc<VisualizationManager>,

    /// Context controller
    controller: Arc<ContextController>,

    /// Interactive visualization
    interactive: Arc<InteractiveVisualization>,

    /// Metrics collector
    metrics: Arc<Mutex<VisualizationMetrics>>,

    /// Event broadcaster
    event_broadcaster: Arc<broadcast::Sender<VisualizationEvent>>,

    /// Web server
    web_server: Option<Arc<WebVisualizationServer>>,
}

/// Visualization event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationEvent {
    /// Event type
    pub event_type: VisualizationEventType,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Event data
    pub data: Value,
}

/// Visualization event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualizationEventType {
    /// Context state changed
    StateChanged,

    /// Rule applied
    RuleApplied,

    /// Visualization created
    VisualizationCreated,

    /// Visualization updated
    VisualizationUpdated,

    /// Interactive session started
    InteractiveSessionStarted,

    /// Interactive session ended
    InteractiveSessionEnded,

    /// Metrics updated
    MetricsUpdated,

    /// Web server started
    WebServerStarted,

    /// Web server stopped
    WebServerStopped,
}

impl VisualizationSystem {
    /// Create a new visualization system
    pub async fn new(config: VisualizationSystemConfig) -> Result<Self> {
        let config = Arc::new(config);
        let (event_tx, _) = broadcast::channel(1000);

        // Create visualization manager
        let manager = Arc::new(VisualizationManager::new(config.clone()).await?);

        // Create context controller
        let controller = Arc::new(ContextController::new(config.clone()).await?);

        // Create interactive visualization
        let interactive = Arc::new(InteractiveVisualization::new(config.clone()).await?);

        // Create metrics
        let metrics = Arc::new(Mutex::new(VisualizationMetrics::new()));

        // Create web server if enabled
        let web_server = if config.enable_web {
            Some(Arc::new(
                WebVisualizationServer::new(config.clone(), event_tx.clone()).await?,
            ))
        } else {
            None
        };

        Ok(Self {
            config,
            manager,
            controller,
            interactive,
            metrics,
            event_broadcaster: Arc::new(event_tx),
            web_server,
        })
    }

    /// Start the visualization system
    pub async fn start(&self) -> Result<()> {
        // Start visualization manager
        self.manager.start().await?;

        // Start context controller
        self.controller.start().await?;

        // Start interactive visualization
        self.interactive.start().await?;

        // Start web server if enabled
        if let Some(web_server) = &self.web_server {
            web_server.start().await?;
        }

        // Start metrics collection
        self.start_metrics_collection().await?;

        // Emit start event
        self.emit_event(
            VisualizationEventType::WebServerStarted,
            serde_json::json!({}),
        )
        .await;

        Ok(())
    }

    /// Stop the visualization system
    pub async fn stop(&self) -> Result<()> {
        // Stop web server if enabled
        if let Some(web_server) = &self.web_server {
            web_server.stop().await?;
        }

        // Stop components
        self.interactive.stop().await?;
        self.controller.stop().await?;
        self.manager.stop().await?;

        // Emit stop event
        self.emit_event(
            VisualizationEventType::WebServerStopped,
            serde_json::json!({}),
        )
        .await;

        Ok(())
    }

    /// Create a visualization
    pub async fn create_visualization(
        &self,
        request: VisualizationRequest,
    ) -> Result<VisualizationResponse> {
        let response = self.manager.create_visualization(request).await?;

        // Emit event
        self.emit_event(
            VisualizationEventType::VisualizationCreated,
            serde_json::json!({
                "type": response.visualization_type,
                "format": response.format
            }),
        )
        .await;

        // Update metrics
        self.update_metrics().await;

        Ok(response)
    }

    /// Update a visualization
    pub async fn update_visualization(
        &self,
        visualization_id: &str,
        data: Value,
    ) -> Result<VisualizationResponse> {
        let response = self
            .manager
            .update_visualization(visualization_id, data)
            .await?;

        // Emit event
        self.emit_event(
            VisualizationEventType::VisualizationUpdated,
            serde_json::json!({
                "visualization_id": visualization_id,
                "type": response.visualization_type,
                "format": response.format
            }),
        )
        .await;

        Ok(response)
    }

    /// Start an interactive session
    pub async fn start_interactive_session(&self, config: Value) -> Result<String> {
        let session_id = self.interactive.start_session(config).await?;

        // Emit event
        self.emit_event(
            VisualizationEventType::InteractiveSessionStarted,
            serde_json::json!({
                "session_id": session_id
            }),
        )
        .await;

        Ok(session_id)
    }

    /// Get visualization metrics
    pub async fn get_metrics(&self) -> VisualizationMetrics {
        self.metrics.lock().await.clone()
    }

    /// Subscribe to visualization events
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<VisualizationEvent> {
        self.event_broadcaster.subscribe()
    }

    // Private helper methods
    async fn start_metrics_collection(&self) -> Result<()> {
        let metrics = self.metrics.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                let mut metrics_guard = metrics.lock().await;
                metrics_guard.update_timestamp();
            }
        });

        Ok(())
    }

    async fn update_metrics(&self) {
        let mut metrics = self.metrics.lock().await;
        metrics.increment_visualizations_created();
    }

    async fn emit_event(&self, event_type: VisualizationEventType, data: Value) {
        let event = VisualizationEvent {
            event_type,
            timestamp: Utc::now(),
            data,
        };

        if let Err(e) = self.event_broadcaster.send(event) {
            eprintln!("Failed to emit visualization event: {e}");
        }
    }
}
