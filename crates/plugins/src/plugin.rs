use crate::web::WebPluginEndpoint;
use async_trait::async_trait;
use serde_json::Value;
use anyhow::Result;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin ID
    pub id: Uuid,
    
    /// Plugin name
    pub name: String,
    
    /// Plugin version
    pub version: String,
    
    /// Plugin description
    pub description: String,
    
    /// Plugin author
    pub author: String,
    
    /// Plugin capabilities
    pub capabilities: Vec<String>,
}

impl PluginMetadata {
    /// Create a new plugin metadata with a random ID
    pub fn new(name: impl Into<String>, version: impl Into<String>, description: impl Into<String>, author: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: version.into(),
            description: description.into(),
            author: author.into(),
            capabilities: Vec::new(),
        }
    }
    
    /// Add a capability to this plugin
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }
    
    /// Add multiple capabilities to this plugin
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities.extend(capabilities);
        self
    }
}

/// Plugin trait
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get the metadata for this plugin
    fn metadata(&self) -> &PluginMetadata;
    
    /// Initialize the plugin
    async fn initialize(&self) -> Result<()>;
    
    /// Shutdown the plugin
    async fn shutdown(&self) -> Result<()>;
    
    /// Check if the plugin is active
    fn is_active(&self) -> bool;
}

/// Plugin trait extensions for web plugins
#[async_trait]
pub trait WebPluginExt: Plugin {
    /// Handle a web endpoint
    async fn handle_web_endpoint(&self, endpoint: &WebPluginEndpoint, data: Value) -> Result<Value> {
        Err(anyhow::anyhow!("Web endpoint handling not implemented"))
    }
}

/// Implement WebPluginExt for all Plugins
impl<T: Plugin + ?Sized> WebPluginExt for T {} 