use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::fmt::Debug;

// Import the Interface version with renamed types
// We're not using these yet, but they'll be needed for migration
use squirrel_interfaces::plugins::{Plugin as IPlugin, PluginMetadata as IPluginMetadata};

/// Legacy Plugin metadata, will be deprecated in favor of IPluginMetadata
#[derive(Clone, Debug, Serialize, Deserialize)]
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
    
    /// Plugin dependencies (IDs of plugins this plugin depends on)
    pub dependencies: Vec<Uuid>,
}

impl PluginMetadata {
    /// Create new plugin metadata
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: version.into(),
            description: description.into(),
            author: author.into(),
            capabilities: Vec::new(),
            dependencies: Vec::new(),
        }
    }
    
    /// Add a capability to the plugin metadata
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }
    
    /// Add a dependency to the plugin
    #[must_use] pub fn with_dependency(mut self, dependency: Uuid) -> Self {
        self.dependencies.push(dependency);
        self
    }
}

/// Status of a plugin in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginStatus {
    /// Plugin is registered but not initialized
    Registered,
    /// Plugin is initialized and active
    Active,
    /// Plugin is disabled (inactive)
    Inactive,
    /// Plugin failed to initialize
    Failed,
}

impl Default for PluginStatus {
    fn default() -> Self {
        Self::Registered
    }
}

/// Legacy Plugin trait, will be deprecated in favor of IPlugin
#[async_trait]
pub trait Plugin: Send + Sync + Debug {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Initialize the plugin
    async fn initialize(&self) -> Result<()>;
    
    /// Shutdown the plugin
    async fn shutdown(&self) -> Result<()>;
    
    /// Plugin feature check
    fn has_feature(&self, feature: &str) -> bool {
        self.metadata().capabilities.contains(&feature.to_string())
    }
}

/// A simplified web plugin endpoint for use with the trait below
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebEndpoint {
    /// Path to the endpoint
    pub path: String,
    
    /// HTTP method
    pub method: String,
    
    /// Required permissions
    pub permissions: Vec<String>,
}

/// Web plugin extension trait
#[async_trait]
pub trait WebPluginExt: Plugin {
    /// Get the endpoints provided by this plugin
    fn get_endpoints(&self) -> Vec<WebEndpoint>;
    
    /// Handle web endpoint request
    async fn handle_web_endpoint(&self, endpoint: &WebEndpoint, data: Option<Value>) -> Result<Value>;
}

// Re-export the CommandsPlugin trait from interfaces for convenience
pub use squirrel_interfaces::plugins::CommandsPlugin; 