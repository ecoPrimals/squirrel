// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Context Visualization System
//!
//! Context-state serialization, event emission, and debug/CLI output for the CMS.
//! Presentation (dashboards, charts, web UI) is petalTongue's domain, accessed
//! via `visualization.render.*` capability IPC.

mod manager;
#[cfg(test)]
mod manager_tests;
mod metrics;
mod renderers;
mod types;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, broadcast};

use crate::error::Result;

pub use manager::VisualizationManager;
pub use metrics::VisualizationMetrics;
pub use renderers::{JsonRenderer, TerminalRenderer};
pub use types::{
    VisualizationConfig, VisualizationRequest, VisualizationResponse, VisualizationType,
};

/// Visualization system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationSystemConfig {
    /// Enable JSON rendering
    pub enable_json: bool,

    /// Enable terminal rendering
    pub enable_terminal: bool,

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
            update_interval: Duration::from_secs(1),
            max_history: 1000,
            enable_metrics: true,
        }
    }
}

/// Visualization system for context management.
///
/// Squirrel owns context-state serialization and event emission.
/// Presentation (dashboards, charts, web UI) is petalTongue's domain,
/// accessed via `visualization.render.*` capability IPC.
#[derive(Debug)]
pub struct VisualizationSystem {
    /// Visualization manager
    manager: Arc<VisualizationManager>,

    /// Metrics collector
    metrics: Arc<Mutex<VisualizationMetrics>>,

    /// Event broadcaster
    event_broadcaster: Arc<broadcast::Sender<VisualizationEvent>>,
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

    /// Metrics updated
    MetricsUpdated,
}

impl VisualizationSystem {
    /// Create a new visualization system
    pub async fn new(config: VisualizationSystemConfig) -> Result<Self> {
        let config = Arc::new(config);
        let (event_tx, _) = broadcast::channel(1000);

        let manager = Arc::new(VisualizationManager::new(config).await?);
        let metrics = Arc::new(Mutex::new(VisualizationMetrics::new()));

        Ok(Self {
            manager,
            metrics,
            event_broadcaster: Arc::new(event_tx),
        })
    }

    /// Start the visualization system
    pub async fn start(&self) -> Result<()> {
        self.manager.start().await?;
        self.start_metrics_collection().await?;
        Ok(())
    }

    /// Stop the visualization system
    pub async fn stop(&self) -> Result<()> {
        self.manager.stop().await?;
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
            tracing::error!("Failed to emit visualization event: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_visualization_system_config_default() {
        let config = VisualizationSystemConfig::default();
        assert!(config.enable_json);
        assert!(config.enable_terminal);
        assert!(config.enable_metrics);
        assert_eq!(config.max_history, 1000);
        assert_eq!(config.update_interval, Duration::from_secs(1));
    }

    #[test]
    fn test_visualization_event_serialization() {
        let event = VisualizationEvent {
            event_type: VisualizationEventType::StateChanged,
            timestamp: Utc::now(),
            data: json!({"key": "value"}),
        };
        let json_str = serde_json::to_string(&event).expect("serialize");
        let _deserialized: VisualizationEvent =
            serde_json::from_str(&json_str).expect("deserialize");
    }

    #[test]
    fn test_visualization_event_type_variants() {
        let _ = VisualizationEventType::StateChanged;
        let _ = VisualizationEventType::RuleApplied;
        let _ = VisualizationEventType::VisualizationCreated;
        let _ = VisualizationEventType::MetricsUpdated;
    }

    #[tokio::test]
    async fn test_visualization_system_new() {
        let config = VisualizationSystemConfig::default();
        let system = VisualizationSystem::new(config).await.expect("create");
        let _metrics = system.get_metrics().await;
        let _rx = system.subscribe_to_events();
    }

    #[test]
    fn visualization_system_config_serde_roundtrip() {
        let cfg = VisualizationSystemConfig::default();
        let json = serde_json::to_string(&cfg).expect("ser");
        let back: VisualizationSystemConfig = serde_json::from_str(&json).expect("de");
        assert_eq!(back.max_history, cfg.max_history);
        assert_eq!(back.enable_metrics, cfg.enable_metrics);
    }

    #[test]
    fn visualization_event_type_all_variants_serialize() {
        let types = [
            VisualizationEventType::StateChanged,
            VisualizationEventType::RuleApplied,
            VisualizationEventType::VisualizationCreated,
            VisualizationEventType::VisualizationUpdated,
            VisualizationEventType::MetricsUpdated,
        ];
        for t in types {
            let json = serde_json::to_string(&t).expect("ser");
            let back: VisualizationEventType = serde_json::from_str(&json).expect("de");
            assert_eq!(std::mem::discriminant(&t), std::mem::discriminant(&back));
        }
    }

    #[test]
    fn visualization_event_full_roundtrip() {
        let ev = VisualizationEvent {
            event_type: VisualizationEventType::MetricsUpdated,
            timestamp: Utc::now(),
            data: json!({"k": 1}),
        };
        let s = serde_json::to_string(&ev).expect("ser");
        let back: VisualizationEvent = serde_json::from_str(&s).expect("de");
        assert!(matches!(
            back.event_type,
            VisualizationEventType::MetricsUpdated
        ));
    }

    #[tokio::test]
    async fn visualization_start_stop() {
        let config = VisualizationSystemConfig {
            enable_metrics: true,
            ..Default::default()
        };
        let system = VisualizationSystem::new(config).await.expect("create");
        system.start().await.expect("start");
        system.stop().await.expect("stop");
    }
}
