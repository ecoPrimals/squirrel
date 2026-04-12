// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Context interfaces for plugin system integration
//!
//! Core traits use `impl Future<Output = _> + Send` so implementations are `Send`.
//! Object-safe [`DynContextTransformation`], [`DynContextPlugin`], and
//! [`DynContextAdapterPlugin`] support heterogeneous collections.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::fmt::Debug;
use std::sync::Arc;

use crate::plugins::{DynPlugin, Plugin};

/// Context transformation trait
pub trait ContextTransformation: Send + Sync + Debug {
    /// Get the transformation ID
    fn get_id(&self) -> &str;

    /// Get the transformation name
    fn get_name(&self) -> &str;

    /// Get the transformation description
    fn get_description(&self) -> &str;

    /// Transform context data
    fn transform(
        &self,
        data: Value,
    ) -> impl std::future::Future<Output = Result<Value>> + Send + '_;
}

/// Object-safe projection of [`ContextTransformation`] for heterogeneous collections.
#[async_trait]
pub trait DynContextTransformation: Send + Sync + Debug {
    /// Get the transformation ID
    fn get_id(&self) -> &str;

    /// Get the transformation name
    fn get_name(&self) -> &str;

    /// Get the transformation description
    fn get_description(&self) -> &str;

    /// Transform context data
    async fn transform(&self, data: Value) -> Result<Value>;
}

#[async_trait]
impl<T: ContextTransformation + Send + Sync> DynContextTransformation for T {
    fn get_id(&self) -> &str {
        ContextTransformation::get_id(self)
    }

    fn get_name(&self) -> &str {
        ContextTransformation::get_name(self)
    }

    fn get_description(&self) -> &str {
        ContextTransformation::get_description(self)
    }

    async fn transform(&self, data: Value) -> Result<Value> {
        ContextTransformation::transform(self, data).await
    }
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
pub trait ContextPlugin: Plugin + Send + Sync {
    /// Get available context transformations
    fn get_transformations(
        &self,
    ) -> impl std::future::Future<Output = Vec<Arc<dyn DynContextTransformation>>> + Send + '_;

    /// Get available adapters
    fn get_adapters(
        &self,
    ) -> impl std::future::Future<Output = Vec<Arc<dyn DynContextAdapterPlugin>>> + Send + '_;
}

/// Object-safe projection of [`ContextPlugin`] for registries.
#[async_trait]
pub trait DynContextPlugin: DynPlugin {
    /// Get available context transformations
    async fn get_transformations(&self) -> Vec<Arc<dyn DynContextTransformation>>;

    /// Get available adapters
    async fn get_adapters(&self) -> Vec<Arc<dyn DynContextAdapterPlugin>>;
}

#[async_trait]
impl<T: ContextPlugin + Send + Sync> DynContextPlugin for T {
    async fn get_transformations(&self) -> Vec<Arc<dyn DynContextTransformation>> {
        ContextPlugin::get_transformations(self).await
    }

    async fn get_adapters(&self) -> Vec<Arc<dyn DynContextAdapterPlugin>> {
        ContextPlugin::get_adapters(self).await
    }
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
pub trait ContextAdapterPlugin: Plugin + Send + Sync + Debug {
    /// Get the adapter metadata
    fn get_metadata(&self) -> impl std::future::Future<Output = AdapterMetadata> + Send + '_;

    /// Convert data from source format to target format
    fn convert(&self, data: Value) -> impl std::future::Future<Output = Result<Value>> + Send + '_;
}

/// Object-safe projection of [`ContextAdapterPlugin`].
#[async_trait]
pub trait DynContextAdapterPlugin: DynPlugin {
    /// Get the adapter metadata
    async fn get_metadata(&self) -> AdapterMetadata;

    /// Convert data from source format to target format
    async fn convert(&self, data: Value) -> Result<Value>;
}

#[async_trait]
impl<T: ContextAdapterPlugin + Send + Sync> DynContextAdapterPlugin for T {
    async fn get_metadata(&self) -> AdapterMetadata {
        ContextAdapterPlugin::get_metadata(self).await
    }

    async fn convert(&self, data: Value) -> Result<Value> {
        ContextAdapterPlugin::convert(self, data).await
    }
}

/// Context manager trait
#[expect(
    async_fn_in_trait,
    reason = "internal trait — all impls are Send + Sync"
)]
pub trait ContextManager: Send + Sync {
    /// Initialize the context manager
    async fn initialize(&self) -> Result<()>;

    /// Transform data using the specified transformation ID
    async fn transform_data(&self, transformation_id: &str, data: Value) -> Result<Value>;

    /// Get all available transformations
    async fn get_transformations(&self) -> Result<Vec<Box<dyn DynContextTransformation>>>;

    /// Register a plugin
    async fn register_plugin(&self, plugin: Box<dyn DynContextPlugin>) -> Result<()>;
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
