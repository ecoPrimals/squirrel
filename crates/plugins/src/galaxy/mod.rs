//! Galaxy plugin module
//!
//! This module provides functionality for galaxy integration plugins.

use std::fmt::Debug;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::plugin::Plugin;

/// Galaxy plugin trait
#[async_trait]
pub trait GalaxyPlugin: Plugin {
    /// Connect to galaxy service
    async fn connect(&self, connection_info: Value) -> Result<()>;
    
    /// Send data to galaxy
    async fn send_data(&self, data: Value) -> Result<Value>;
    
    /// Receive data from galaxy
    async fn receive_data(&self) -> Result<Value>;
    
    /// Get supported integration types
    fn get_integration_types(&self) -> Vec<String>;
    
    /// Check if plugin supports an integration type
    fn supports_integration_type(&self, integration_type: &str) -> bool {
        self.get_integration_types().contains(&integration_type.to_string())
    }
    
    /// Get plugin capabilities
    fn get_capabilities(&self) -> Vec<String> {
        self.metadata().capabilities.clone()
    }
} 