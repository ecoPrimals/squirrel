//! Dashboard plugin types and interfaces
//!
//! This module defines the core types and traits for dashboard plugins.

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;
use squirrel_core::error::Result;
use crate::dashboard::DashboardComponent;

/// Plugin ID type alias
pub type PluginId = Uuid;

/// Dashboard plugin type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashboardPluginType {
    /// Visualization plugin
    Visualization,
    /// Data source plugin
    DataSource,
    /// Widget plugin
    Widget,
    /// Theme plugin
    Theme,
    /// Layout plugin
    Layout,
    /// Other plugin type
    Other,
}

impl ToString for DashboardPluginType {
    fn to_string(&self) -> String {
        match self {
            Self::Visualization => "visualization".to_string(),
            Self::DataSource => "data_source".to_string(),
            Self::Widget => "widget".to_string(),
            Self::Theme => "theme".to_string(),
            Self::Layout => "layout".to_string(),
            Self::Other => "other".to_string(),
        }
    }
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin ID
    pub id: Uuid,
    /// Plugin type
    pub plugin_type: DashboardPluginType,
    /// Update interval in seconds
    pub update_interval: u64,
    /// Custom configuration
    pub custom_config: Value,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            plugin_type: DashboardPluginType::Visualization,
            update_interval: 30,
            custom_config: Value::Null,
        }
    }
}

/// Dashboard plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin ID
    pub id: String,
    /// Plugin name
    pub name: String,
    /// Plugin description
    pub description: String,
    /// Plugin version
    pub version: String,
    /// Plugin author
    pub author: String,
    /// Plugin type
    pub plugin_type: DashboardPluginType,
}

/// Dashboard plugin event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginEvent {
    /// Configuration update event
    ConfigUpdate(Value),
    /// Data update event
    DataUpdate(Value),
    /// Custom event
    Custom(String, Value),
}

/// Plugin context
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// Plugin configuration
    pub config: PluginConfig,
    /// Dashboard component count
    pub component_count: usize,
}

/// Dashboard plugin trait
#[async_trait]
pub trait DashboardPlugin: DashboardComponent + Send + Sync + Debug {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;
    
    /// Get plugin type
    fn plugin_type(&self) -> DashboardPluginType;
    
    /// Initialize the plugin
    async fn initialize(&self) -> Result<()>;
    
    /// Get plugin data
    async fn get_data(&self) -> Result<Value>;
    
    /// Update plugin with new data
    async fn update(&self, data: Value) -> Result<()>;
    
    /// Handle plugin event
    async fn handle_event(&self, event: PluginEvent) -> Result<()>;
    
    /// Shutdown the plugin
    async fn shutdown(&self) -> Result<()>;
}

/// Visualization plugin trait
#[async_trait]
pub trait VisualizationPlugin: DashboardPlugin {
    /// Get visualization configuration
    async fn get_visualization_config(&self) -> Result<Value>;
    
    /// Render visualization
    async fn render(&self, data: Value) -> Result<String>;
}

/// Data source plugin trait
#[async_trait]
pub trait DataSourcePlugin: DashboardPlugin {
    /// Get data source schema
    async fn get_schema(&self) -> Result<Value>;
    
    /// Query data from data source
    async fn query(&self, query: Value) -> Result<Value>;
    
    /// Subscribe to data updates
    async fn subscribe(&self, query: Value) -> Result<()>;
    
    /// Unsubscribe from data updates
    async fn unsubscribe(&self, subscription_id: String) -> Result<()>;
}

/// Plugin registry trait
#[async_trait]
pub trait DashboardPluginRegistry: Send + Sync + Debug {
    /// Register a plugin with the registry
    async fn register_plugin(&self, plugin: Arc<dyn DashboardPlugin>) -> Result<()>;
    
    /// Get all plugins in the registry
    async fn get_plugins(&self) -> Result<Vec<Arc<dyn DashboardPlugin>>>;
    
    /// Get a plugin by ID
    async fn get_plugin(&self, id: &str) -> Result<Option<Arc<dyn DashboardPlugin>>>;
    
    /// Get plugins by type
    async fn get_plugins_by_type(&self, plugin_type: DashboardPluginType) -> Result<Vec<Arc<dyn DashboardPlugin>>>;
    
    /// Remove a plugin from the registry
    async fn remove_plugin(&self, id: &str) -> Result<()>;
} 