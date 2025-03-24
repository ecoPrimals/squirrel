/*!
 * Default Galaxy Plugin Implementation
 * 
 * This module provides the default implementation of the Galaxy plugin interface.
 */

use std::fmt::Debug;
use std::any::Any;
use std::sync::Arc;
use async_trait::async_trait;

use crate::adapter::GalaxyAdapter;
use crate::error::Error;
use crate::plugin::GalaxyPlugin;

/// Default implementation of the Galaxy plugin
#[derive(Debug)]
pub struct DefaultGalaxyPlugin {
    /// Plugin name
    name: String,
    /// Plugin version
    version: String,
    /// Plugin description
    description: String,
    /// Plugin capabilities
    capabilities: Vec<String>,
    /// Adapter reference
    adapter: Option<Arc<GalaxyAdapter>>,
}

impl DefaultGalaxyPlugin {
    /// Create a new default plugin
    pub fn new(name: &str, version: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            description: description.to_string(),
            capabilities: vec![],
            adapter: None,
        }
    }
    
    /// Add a capability to this plugin
    pub fn with_capability(mut self, capability: &str) -> Self {
        self.capabilities.push(capability.to_string());
        self
    }
    
    /// Get the adapter reference
    pub fn adapter(&self) -> Result<Arc<GalaxyAdapter>, Error> {
        match &self.adapter {
            Some(adapter) => Ok(Arc::clone(adapter)),
            None => Err(Error::InvalidState("Plugin not initialized with adapter".to_string())),
        }
    }
}

#[async_trait]
impl GalaxyPlugin for DefaultGalaxyPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    async fn initialize(&self, adapter: Arc<GalaxyAdapter>) -> Result<(), Error> {
        // Store the adapter in our internal field
        let this = self as *const Self as *mut Self;
        unsafe {
            (*this).adapter = Some(Arc::clone(&adapter));
        }
        
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<(), Error> {
        // Clear the adapter reference
        let this = self as *const Self as *mut Self;
        unsafe {
            (*this).adapter = None;
        }
        
        Ok(())
    }
    
    fn provides_capability(&self, capability: &str) -> bool {
        self.capabilities.contains(&capability.to_string())
    }
    
    fn capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
} 