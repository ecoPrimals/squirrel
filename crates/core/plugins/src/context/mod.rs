// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Context plugin module
//!
//! This module provides functionality for context plugins.

mod plugin;

use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
use serde_json::Value;

pub use plugin::{ContextPluginImpl, create_context_plugin, create_custom_context_plugin};

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

/// Context plugin trait (local; `dyn`-safe via boxed futures).
pub trait ContextPlugin: crate::plugin::Plugin {
    /// Get available context transformations
    fn get_transformations(&self) -> Vec<ContextTransformation>;

    /// Transform context data
    fn transform(
        &self,
        transformation_id: &str,
        data: Value,
    ) -> Pin<Box<dyn Future<Output = Result<Value>> + Send + '_>>;

    /// Validate context data against a schema
    fn validate(&self, schema: &Value, data: &Value) -> Result<bool>;

    /// Check if the plugin supports a transformation
    fn supports_transformation(&self, transformation_id: &str) -> bool {
        self.get_transformations()
            .iter()
            .any(|t| t.id == transformation_id)
    }

    /// Get plugin capabilities
    fn get_capabilities(&self) -> Vec<String> {
        self.metadata().capabilities.clone()
    }
}
