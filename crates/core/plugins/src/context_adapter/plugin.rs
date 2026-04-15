// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use serde_json::Value;

use crate::plugin::{Plugin, PluginMetadata};
use super::{AdapterMetadata, ContextAdapterPlugin};

/// Context Adapter Plugin implementation
#[derive(Debug)]
pub struct ContextAdapterPluginImpl {
    /// Plugin metadata
    metadata: PluginMetadata,
    /// Available adapters
    adapters: Vec<AdapterMetadata>,
}

impl ContextAdapterPluginImpl {
    /// Create a new context adapter plugin instance
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let metadata = PluginMetadata::new(
            name,
            env!("CARGO_PKG_VERSION"),
            description,
            "ecoPrimals Contributors",
        )
        .with_capability("context.adapter")
        .with_capability("context.format");

        Self {
            metadata,
            adapters: Vec::new(),
        }
    }

    /// Add an adapter to the plugin
    pub fn with_adapter(mut self, adapter: AdapterMetadata) -> Self {
        self.adapters.push(adapter);
        self
    }

    /// Create a default context adapter plugin with standard adapters
    pub fn default_context_adapter_plugin() -> Self {
        let base_plugin = Self::new(
            "Context Adapter Plugin",
            "Provides context format adaptation functionality",
        );

        // Add standard adapters
        base_plugin
            .with_adapter(AdapterMetadata {
                id: "json.to.mcp".to_string(),
                name: "JSON to MCP Adapter".to_string(),
                description: "Converts JSON format to MCP format".to_string(),
                source_format: "json".to_string(),
                target_format: "mcp".to_string(),
            })
            .with_adapter(AdapterMetadata {
                id: "mcp.to.json".to_string(),
                name: "MCP to JSON Adapter".to_string(),
                description: "Converts MCP format to JSON format".to_string(),
                source_format: "mcp".to_string(),
                target_format: "json".to_string(),
            })
    }
}

impl Plugin for ContextAdapterPluginImpl {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn initialize(&self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async {
            tracing::info!("Initializing Context Adapter Plugin: {}", self.metadata.name);
            Ok(())
        })
    }

    fn shutdown(&self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async {
            tracing::info!("Shutting down Context Adapter Plugin: {}", self.metadata.name);
            Ok(())
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ContextAdapterPlugin for ContextAdapterPluginImpl {
    fn get_adapters(&self) -> Vec<AdapterMetadata> {
        self.adapters.clone()
    }

    fn convert(
        &self,
        adapter_id: &str,
        data: Value,
    ) -> Pin<Box<dyn Future<Output = Result<Value>> + Send + '_>> {
        let aid = adapter_id.to_string();
        Box::pin(async {
            if !self.supports_adapter(&aid) {
                return Err(anyhow::anyhow!("Adapter not supported: {}", aid));
            }

            let adapter = self
                .adapters
                .iter()
                .find(|a| a.id == aid)
                .ok_or_else(|| anyhow::anyhow!("Adapter not found: {}", aid))?;

            let result = serde_json::json!({
                "converted_data": data,
                "metadata": {
                    "adapter_id": aid,
                    "source_format": adapter.source_format,
                    "target_format": adapter.target_format,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "plugin_id": self.metadata.id.to_string(),
                }
            });

            Ok(result)
        })
    }

    fn validate_format(&self, format: &str, data: &Value) -> Result<bool> {
        match format {
            "json" => Ok(data.is_object() || data.is_array()),
            "mcp" => {
                if let Value::Object(obj) = data {
                    Ok(obj.contains_key("version") && obj.contains_key("message"))
                } else {
                    Ok(false)
                }
            }
            _ => Err(anyhow::anyhow!("Unsupported format: {}", format)),
        }
    }

    fn check_compatibility(&self, source_format: &str, target_format: &str) -> bool {
        self.adapters.iter().any(|a| {
            a.source_format == source_format && a.target_format == target_format
        })
    }
}

/// Create a new context adapter plugin with default configuration
pub fn create_context_adapter_plugin() -> Arc<dyn ContextAdapterPlugin> {
    Arc::new(ContextAdapterPluginImpl::default_context_adapter_plugin())
}

/// Create a new context adapter plugin with custom adapters
pub fn create_custom_context_adapter_plugin(
    name: impl Into<String>,
    description: impl Into<String>,
    adapters: Vec<AdapterMetadata>,
) -> Arc<dyn ContextAdapterPlugin> {
    let mut plugin = ContextAdapterPluginImpl::new(name, description);
    for adapter in adapters {
        plugin = plugin.with_adapter(adapter);
    }
    Arc::new(plugin)
}
