// Common plugin traits and structures for monitoring
// This provides a simplified plugin interface to avoid circular dependencies

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::fmt::Debug;
use uuid::Uuid;

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Plugin identifier
    pub id: Uuid,
    /// Plugin capabilities
    pub capabilities: Vec<String>,
}

impl PluginMetadata {
    /// Create new plugin metadata
    pub fn new(name: &str, version: &str, description: &str, author: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            description: description.to_string(),
            author: author.to_string(),
            id: Uuid::new_v4(),
            capabilities: Vec::new(),
        }
    }

    /// Add a capability to this plugin
    pub fn with_capability(mut self, capability: &str) -> Self {
        self.capabilities.push(capability.to_string());
        self
    }
}

/// Base plugin trait
#[async_trait]
pub trait MonitoringPlugin: Send + Sync + Debug {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Initialize the plugin
    async fn initialize(&self) -> anyhow::Result<()>;

    /// Shutdown the plugin
    async fn shutdown(&self) -> anyhow::Result<()>;

    /// Collect metrics from the plugin
    async fn collect_metrics(&self) -> anyhow::Result<Value>;
    
    /// Get monitoring targets provided by this plugin
    fn get_monitoring_targets(&self) -> Vec<String>;
    
    /// Handle an alert
    async fn handle_alert(&self, alert: Value) -> anyhow::Result<()>;
} 