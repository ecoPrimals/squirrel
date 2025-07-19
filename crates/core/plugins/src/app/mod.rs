//! App plugin module
//!
//! This module provides functionality for application plugins.

use std::fmt::Debug;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::plugin::Plugin;

/// App extension point
#[derive(Clone, Debug)]
pub struct AppExtensionPoint {
    /// Extension point ID
    pub id: String,
    
    /// Extension point name
    pub name: String,
    
    /// Extension point description
    pub description: String,
}

/// App plugin trait
#[async_trait]
pub trait AppPlugin: Plugin {
    /// Get extension points
    fn get_extension_points(&self) -> Vec<AppExtensionPoint>;
    
    /// Register extension
    async fn register_extension(&self, extension_point_id: &str, data: Value) -> Result<()>;
    
    /// Unregister extension
    async fn unregister_extension(&self, extension_point_id: &str, extension_id: &str) -> Result<()>;
    
    /// Get extensions for extension point
    async fn get_extensions(&self, extension_point_id: &str) -> Result<Vec<Value>>;
    
    /// Check if plugin supports extension point
    fn supports_extension_point(&self, extension_point_id: &str) -> bool {
        self.get_extension_points().iter().any(|ep| ep.id == extension_point_id)
    }
    
    /// Get plugin capabilities
    fn get_capabilities(&self) -> Vec<String> {
        self.metadata().capabilities.clone()
    }
} 