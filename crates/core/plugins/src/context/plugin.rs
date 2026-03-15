// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::plugin::{Plugin, PluginMetadata};
use super::{ContextPlugin, ContextTransformation};

/// Context Plugin implementation
#[derive(Debug)]
pub struct ContextPluginImpl {
    /// Plugin metadata
    metadata: PluginMetadata,
    /// Available transformations
    transformations: Vec<ContextTransformation>,
}

impl ContextPluginImpl {
    /// Create a new context plugin instance
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let metadata = PluginMetadata::new(
            name,
            env!("CARGO_PKG_VERSION"),
            description,
            "ecoPrimals Contributors",
        )
        .with_capability("context.transform")
        .with_capability("context.validate");

        Self {
            metadata,
            transformations: Vec::new(),
        }
    }

    /// Add a transformation to the plugin
    pub fn with_transformation(mut self, transformation: ContextTransformation) -> Self {
        self.transformations.push(transformation);
        self
    }

    /// Create a default context plugin with standard transformations
    pub fn default_context_plugin() -> Self {
        let base_plugin = Self::new(
            "Context Plugin",
            "Provides context transformation and validation functionality",
        );

        // Add standard transformations
        base_plugin
            .with_transformation(ContextTransformation {
                id: "context.standard".to_string(),
                name: "Standard Context Transformation".to_string(),
                description: "Transforms context data to standard format".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "data": { "type": "object" }
                    }
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "result": { "type": "object" },
                        "metadata": { "type": "object" }
                    }
                }),
            })
    }
}

#[async_trait]
impl Plugin for ContextPluginImpl {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing Context Plugin: {}", self.metadata.name);
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down Context Plugin: {}", self.metadata.name);
        Ok(())
    }
}

#[async_trait]
impl ContextPlugin for ContextPluginImpl {
    fn get_transformations(&self) -> Vec<ContextTransformation> {
        self.transformations.clone()
    }

    async fn transform(&self, transformation_id: &str, data: Value) -> Result<Value> {
        if !self.supports_transformation(transformation_id) {
            return Err(anyhow::anyhow!("Transformation not supported: {}", transformation_id));
        }

        // For now, we'll just add basic metadata to the result
        let result = serde_json::json!({
            "result": data,
            "metadata": {
                "transformation_id": transformation_id,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "plugin_id": self.metadata.id.to_string(),
            }
        });

        Ok(result)
    }

    fn validate(&self, schema: &Value, data: &Value) -> Result<bool> {
        // Basic schema validation logic
        // In a real implementation, use a proper JSON schema validator
        if let (Value::Object(schema_obj), Value::Object(data_obj)) = (schema, data) {
            // Simple validation for demo purposes
            for (key, schema_value) in schema_obj {
                if schema_value.get("required").is_some() && !data_obj.contains_key(key) {
                    return Ok(false);
                }
            }
            Ok(true)
        } else {
            Err(anyhow::anyhow!("Invalid schema or data format"))
        }
    }
}

/// Create a new context plugin with default configuration
pub fn create_context_plugin() -> Arc<dyn ContextPlugin> {
    Arc::new(ContextPluginImpl::default_context_plugin())
}

/// Create a new context plugin with custom transformations
pub fn create_custom_context_plugin(
    name: impl Into<String>,
    description: impl Into<String>,
    transformations: Vec<ContextTransformation>,
) -> Arc<dyn ContextPlugin> {
    let mut plugin = ContextPluginImpl::new(name, description);
    for transformation in transformations {
        plugin = plugin.with_transformation(transformation);
    }
    Arc::new(plugin)
} 