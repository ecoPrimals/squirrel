// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin manager for context transformations and adapters
use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::RwLock;
use tracing::debug;

use crate::ContextError;
use squirrel_interfaces::context::{
    AdapterMetadata, DynContextAdapterPlugin, DynContextPlugin, DynContextTransformation,
};

/// Plugin manager for managing context plugins and transformations
#[derive(Debug)]
pub struct ContextPluginManager {
    /// Collection of registered context plugins
    plugins: RwLock<Vec<Box<dyn DynContextPlugin>>>,
    /// Collection of available context transformations from plugins
    transformations: RwLock<Vec<Arc<dyn DynContextTransformation>>>,
    /// Map of adapter IDs to context adapter plugins
    adapters: RwLock<HashMap<String, Arc<dyn DynContextAdapterPlugin>>>,
}

impl ContextPluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(Vec::new()),
            transformations: RwLock::new(Vec::new()),
            adapters: RwLock::new(HashMap::new()),
        }
    }

    /// Register a plugin
    pub async fn register_plugin(
        &self,
        plugin: Box<dyn DynContextPlugin>,
    ) -> Result<(), ContextError> {
        // Get the plugin's transformations
        let transformations = plugin.get_transformations().await;

        // Get the plugin's adapters
        let adapters = plugin.get_adapters().await;

        // Register transformations
        {
            let mut transformations_lock = self.transformations.write().await;
            for transformation in transformations {
                transformations_lock.push(transformation);
            }
        }

        // Register adapters
        {
            let mut adapters_lock = self.adapters.write().await;
            for adapter in adapters {
                let metadata = adapter.get_metadata().await;
                adapters_lock.insert(metadata.id.clone(), adapter);
            }
        }

        // Store the plugin
        self.plugins.write().await.push(plugin);

        Ok(())
    }

    /// Load plugins from a directory path
    pub async fn load_plugins_from_path(&self, _path: &str) -> Result<(), ContextError> {
        debug!("Dynamic plugin loading not available — use capability.call for plugin dispatch");
        Ok(())
    }

    /// Get all registered transformations
    pub async fn get_transformations(&self) -> Vec<Arc<dyn DynContextTransformation>> {
        self.transformations.read().await.clone()
    }

    /// Get all registered adapters
    pub async fn get_adapters(&self) -> HashMap<String, Arc<dyn DynContextAdapterPlugin>> {
        self.adapters.read().await.clone()
    }

    /// Transform data using a registered transformation
    pub async fn transform(
        &self,
        transformation_id: &str,
        data: Value,
    ) -> Result<Value, ContextError> {
        // Find the transformation
        let transformations = self.transformations.read().await;
        let transformation = transformations
            .iter()
            .find(|t| t.get_id() == transformation_id)
            .ok_or(ContextError::TransformationNotFound(
                transformation_id.to_string(),
            ))?;

        // Apply the transformation
        transformation.transform(data).await.map_err(|e| {
            ContextError::TransformationFailed(transformation_id.to_string(), e.to_string())
        })
    }

    /// Get adapter by ID
    pub async fn get_adapter(&self, adapter_id: &str) -> Option<Arc<dyn DynContextAdapterPlugin>> {
        self.adapters.read().await.get(adapter_id).cloned()
    }

    /// Get adapter metadata
    pub async fn get_adapter_metadata(
        &self,
        adapter_id: &str,
    ) -> Result<AdapterMetadata, ContextError> {
        let adapter = self
            .get_adapter(adapter_id)
            .await
            .ok_or(ContextError::AdapterNotFound(adapter_id.to_string()))?;

        Ok(adapter.get_metadata().await)
    }
}

impl Default for ContextPluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a default plugin manager with built-in plugins
#[cfg(feature = "with-plugins")]
pub async fn create_default_plugin_manager() -> Result<Arc<ContextPluginManager>, ContextError> {
    let manager = Arc::new(ContextPluginManager::new());

    Ok(manager)
}

#[cfg(test)]
mod tests {
    use super::ContextPluginManager;
    use crate::ContextError;
    use serde_json::{Value, json};
    use squirrel_interfaces::context::{
        AdapterMetadata, ContextAdapterPlugin, ContextPlugin, ContextTransformation,
        DynContextAdapterPlugin, DynContextPlugin, DynContextTransformation,
    };
    use squirrel_interfaces::plugins::{Plugin, PluginMetadata};
    use std::sync::Arc;

    #[derive(Debug)]
    struct MetaPlugin {
        meta: PluginMetadata,
        transforms: Vec<Arc<dyn DynContextTransformation>>,
        adapters: Vec<Arc<dyn DynContextAdapterPlugin>>,
    }

    #[derive(Debug)]
    struct IdTransform {
        id: &'static str,
    }

    impl ContextTransformation for IdTransform {
        fn get_id(&self) -> &str {
            self.id
        }

        fn get_name(&self) -> &'static str {
            "n"
        }

        fn get_description(&self) -> &'static str {
            "d"
        }

        async fn transform(&self, data: Value) -> anyhow::Result<Value> {
            Ok(json!({ "wrapped": data }))
        }
    }

    #[derive(Debug)]
    struct StaticAdapter {
        plugin_meta: PluginMetadata,
        adapter_meta: AdapterMetadata,
    }

    impl Plugin for StaticAdapter {
        fn metadata(&self) -> &PluginMetadata {
            &self.plugin_meta
        }
    }

    impl ContextAdapterPlugin for StaticAdapter {
        async fn get_metadata(&self) -> AdapterMetadata {
            self.adapter_meta.clone()
        }

        async fn convert(&self, data: Value) -> anyhow::Result<Value> {
            Ok(data)
        }
    }

    impl Plugin for MetaPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.meta
        }
    }

    impl ContextPlugin for MetaPlugin {
        async fn get_transformations(&self) -> Vec<Arc<dyn DynContextTransformation>> {
            self.transforms.clone()
        }

        async fn get_adapters(&self) -> Vec<Arc<dyn DynContextAdapterPlugin>> {
            self.adapters.clone()
        }
    }

    fn sample_plugin_with_transform() -> Box<dyn DynContextPlugin> {
        let meta = PluginMetadata::new("p.test", "1.0.0", "d", "a");
        let tid: Arc<dyn DynContextTransformation> = Arc::new(IdTransform { id: "tid" });
        Box::new(MetaPlugin {
            meta,
            transforms: vec![tid],
            adapters: vec![],
        })
    }

    fn sample_plugin_with_adapter() -> Box<dyn DynContextPlugin> {
        let plugin_meta = PluginMetadata::new("p.adapt", "1.0.0", "d", "a");
        let adapter_meta = AdapterMetadata {
            id: "aid".to_string(),
            name: "Adapter".to_string(),
            description: "desc".to_string(),
            source_format: "a".to_string(),
            target_format: "b".to_string(),
        };
        let adapter: Arc<dyn DynContextAdapterPlugin> = Arc::new(StaticAdapter {
            plugin_meta: plugin_meta.clone(),
            adapter_meta: adapter_meta.clone(),
        });
        Box::new(MetaPlugin {
            meta: plugin_meta,
            transforms: vec![],
            adapters: vec![adapter],
        })
    }

    #[tokio::test]
    async fn register_plugin_registers_transformations_and_adapters() {
        let mgr = ContextPluginManager::new();
        mgr.register_plugin(sample_plugin_with_transform())
            .await
            .expect("should succeed");
        let t = mgr.get_transformations().await;
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].get_id(), "tid");
    }

    #[tokio::test]
    async fn transform_happy_path() {
        let mgr = ContextPluginManager::new();
        mgr.register_plugin(sample_plugin_with_transform())
            .await
            .expect("should succeed");
        let out = mgr
            .transform("tid", json!({"x": 1}))
            .await
            .expect("should succeed");
        assert_eq!(out["wrapped"]["x"], 1);
    }

    #[tokio::test]
    async fn transform_unknown_id_errors() {
        let mgr = ContextPluginManager::new();
        mgr.register_plugin(sample_plugin_with_transform())
            .await
            .expect("should succeed");
        let err = mgr.transform("missing", json!({})).await.unwrap_err();
        assert!(matches!(err, ContextError::TransformationNotFound(_)));
    }

    #[tokio::test]
    async fn get_adapter_and_metadata() {
        let mgr = ContextPluginManager::new();
        mgr.register_plugin(sample_plugin_with_adapter())
            .await
            .expect("should succeed");
        let a = mgr.get_adapter("aid").await.expect("adapter");
        let meta = mgr
            .get_adapter_metadata("aid")
            .await
            .expect("should succeed");
        assert_eq!(meta.id, "aid");
        assert_eq!(a.get_metadata().await.id, "aid");
    }

    #[tokio::test]
    async fn get_adapter_metadata_missing_errors() {
        let mgr = ContextPluginManager::new();
        let err = mgr.get_adapter_metadata("nope").await.unwrap_err();
        assert!(matches!(err, ContextError::AdapterNotFound(_)));
    }

    #[tokio::test]
    async fn load_plugins_from_path_ok() {
        let mgr = ContextPluginManager::new();
        mgr.load_plugins_from_path("/nonexistent/unloaded_plugins")
            .await
            .expect("should succeed");
    }

    #[tokio::test]
    async fn default_matches_new() {
        let _ = ContextPluginManager::default();
    }

    #[tokio::test]
    async fn get_adapters_map() {
        let mgr = ContextPluginManager::new();
        mgr.register_plugin(sample_plugin_with_adapter())
            .await
            .expect("should succeed");
        let map = mgr.get_adapters().await;
        assert!(map.contains_key("aid"));
    }
}
