//! Context adapter plugin module
//!
//! This module provides functionality for context adapter plugins.

use std::fmt::Debug;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::plugin::Plugin;

/// Adapter metadata
#[derive(Clone, Debug)]
pub struct AdapterMetadata {
    /// Adapter ID
    pub id: String,
    
    /// Adapter name
    pub name: String,
    
    /// Adapter description
    pub description: String,
    
    /// Source format
    pub source_format: String,
    
    /// Target format
    pub target_format: String,
}

/// Context adapter plugin trait
#[async_trait]
pub trait ContextAdapterPlugin: Plugin {
    /// Get available adapters
    fn get_adapters(&self) -> Vec<AdapterMetadata>;
    
    /// Convert data from source format to target format
    async fn convert(&self, adapter_id: &str, data: Value) -> Result<Value>;
    
    /// Validate data format
    fn validate_format(&self, format: &str, data: &Value) -> Result<bool>;
    
    /// Check compatibility between formats
    fn check_compatibility(&self, source_format: &str, target_format: &str) -> bool;
    
    /// Check if the plugin supports an adapter
    fn supports_adapter(&self, adapter_id: &str) -> bool {
        self.get_adapters().iter().any(|a| a.id == adapter_id)
    }
    
    /// Get plugin capabilities
    fn get_capabilities(&self) -> Vec<String> {
        self.metadata().capabilities.clone()
    }
} 