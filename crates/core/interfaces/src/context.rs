// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Context interfaces for plugin system integration
//!
//! This module defines shared interfaces for the context system to avoid
//! circular dependencies between the context and plugins crates.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::fmt::Debug;
use std::sync::Arc;

use crate::plugins::Plugin;

/// Context transformation trait
///
/// This trait defines the interface for context transformations.
#[async_trait]
pub trait ContextTransformation: Send + Sync + Debug {
    /// Get the transformation ID
    fn get_id(&self) -> &str;

    /// Get the transformation name
    fn get_name(&self) -> &str;

    /// Get the transformation description
    fn get_description(&self) -> &str;

    /// Transform context data
    async fn transform(
        &self,
        data: Value,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>>;
}

/// Context transformation metadata
#[derive(Clone, Debug)]
pub struct TransformationMetadata {
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
///
/// This trait defines the interface for plugins that provide context transformations.
#[async_trait]
pub trait ContextPlugin: Plugin + Send + Sync {
    /// Get available context transformations
    async fn get_transformations(&self) -> Vec<Arc<dyn ContextTransformation>>;

    /// Get available adapters
    async fn get_adapters(&self) -> Vec<Arc<dyn ContextAdapterPlugin>>;
}

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
///
/// This trait defines the interface for plugins that provide format conversion.
#[async_trait]
pub trait ContextAdapterPlugin: Plugin + Send + Sync + Debug {
    /// Get the adapter metadata
    async fn get_metadata(&self) -> AdapterMetadata;

    /// Convert data from source format to target format
    async fn convert(&self, data: Value)
    -> Result<Value, Box<dyn std::error::Error + Send + Sync>>;
}

/// Context manager trait
///
/// This trait defines the interface for context management.
#[async_trait]
pub trait ContextManager: Send + Sync {
    /// Initialize the context manager
    async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Transform data using the specified transformation ID
    async fn transform_data(
        &self,
        transformation_id: &str,
        data: Value,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>>;

    /// Get all available transformations
    async fn get_transformations(
        &self,
    ) -> Result<Vec<Box<dyn ContextTransformation>>, Box<dyn std::error::Error + Send + Sync>>;

    /// Register a plugin
    async fn register_plugin(
        &self,
        plugin: Box<dyn ContextPlugin>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transformation_metadata() {
        let meta = TransformationMetadata {
            id: "transform-1".to_string(),
            name: "Test Transform".to_string(),
            description: "A test transformation".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: serde_json::json!({"type": "string"}),
        };
        assert_eq!(meta.id, "transform-1");
        assert_eq!(meta.name, "Test Transform");
        assert_eq!(meta.description, "A test transformation");
    }

    #[test]
    fn test_transformation_metadata_clone() {
        let meta = TransformationMetadata {
            id: "t1".to_string(),
            name: "T1".to_string(),
            description: "Desc".to_string(),
            input_schema: serde_json::Value::Null,
            output_schema: serde_json::Value::Null,
        };
        let cloned = meta.clone();
        assert_eq!(cloned.id, meta.id);
    }

    #[test]
    fn test_adapter_metadata() {
        let meta = AdapterMetadata {
            id: "adapter-1".to_string(),
            name: "Test Adapter".to_string(),
            description: "Converts format A to B".to_string(),
            source_format: "json".to_string(),
            target_format: "yaml".to_string(),
        };
        assert_eq!(meta.id, "adapter-1");
        assert_eq!(meta.source_format, "json");
        assert_eq!(meta.target_format, "yaml");
    }

    #[test]
    fn test_adapter_metadata_clone() {
        let meta = AdapterMetadata {
            id: "a1".to_string(),
            name: "A1".to_string(),
            description: "Desc".to_string(),
            source_format: "a".to_string(),
            target_format: "b".to_string(),
        };
        let cloned = meta.clone();
        assert_eq!(cloned.source_format, meta.source_format);
    }
}
