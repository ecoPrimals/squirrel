// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Context plugin module
//!
//! This module provides functionality for context plugins.

mod plugin;

use std::fmt::Debug;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

// Use the interfaces definitions instead of local ones
pub use squirrel_interfaces::context::{ContextPlugin, ContextTransformation};
pub use squirrel_interfaces::plugins::Plugin;

/// Context transformation metadata
#[derive(Clone, Debug)]
pub struct ContextTransformation {
    /// Transformation ID
    pub id: String,
    
    /// Transformation name
    pub name: String,
    
    /// Transformation description
    pub description: String,
    
    /// Input schema
    pub input_schema: Value,
    
    /// Output schema
    pub output_schema: Value,
}

/// Context plugin trait
#[async_trait]
pub trait ContextPlugin: Plugin {
    /// Get available context transformations
    fn get_transformations(&self) -> Vec<ContextTransformation>;
    
    /// Transform context data
    async fn transform(&self, transformation_id: &str, data: Value) -> Result<Value>;
    
    /// Validate context data against a schema
    fn validate(&self, schema: &Value, data: &Value) -> Result<bool>;
    
    /// Check if the plugin supports a transformation
    fn supports_transformation(&self, transformation_id: &str) -> bool {
        self.get_transformations().iter().any(|t| t.id == transformation_id)
    }
    
    /// Get plugin capabilities
    fn get_capabilities(&self) -> Vec<String> {
        self.metadata().capabilities.clone()
    }
} 

// Re-export the plugin implementation and factory functions
pub use plugin::{ContextPluginImpl, create_context_plugin, create_custom_context_plugin}; 