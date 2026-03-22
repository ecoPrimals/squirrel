// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Web plugin registry module
//!
//! This module provides functionality for managing web plugins.

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::registry::PluginRegistry;
use crate::web::{
    HttpMethod, Route, WebComponent, WebEndpoint, WebPlugin, WebRequest, WebResponse,
};

/// Error types for web plugin registry
#[derive(Debug, thiserror::Error)]
pub enum WebPluginError {
    /// Endpoint not found
    #[error("Endpoint not found for path {0} and method {1}")]
    EndpointNotFound(String, String),

    /// Component not found
    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    /// Plugin error
    #[error("Plugin error: {0}")]
    PluginError(#[from] anyhow::Error),
}

/// Web plugin registry
pub struct WebPluginRegistry {
    /// Plugin registry reference
    registry: Arc<dyn PluginRegistry>,
    /// Endpoints by plugin ID
    endpoints: RwLock<HashMap<Uuid, Vec<WebEndpoint>>>,
    /// Components by plugin ID
    components: RwLock<HashMap<Uuid, Vec<WebComponent>>>,
    /// Cached routes for pattern matching
    routes: RwLock<HashMap<String, Route>>,
}

impl WebPluginRegistry {
    /// Create a new web plugin registry
    pub fn new(registry: Arc<dyn PluginRegistry>) -> Self {
        Self {
            registry,
            endpoints: RwLock::new(HashMap::new()),
            components: RwLock::new(HashMap::new()),
            routes: RwLock::new(HashMap::new()),
        }
    }

    /// Load all web plugins
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if the underlying plugin registry cannot be queried.
    pub async fn load_plugins(&self) -> Result<usize> {
        let plugins = self.registry.get_plugins().await?;
        let mut count = 0;

        for plugin in plugins {
            // Check if the plugin implements WebPlugin
            if let Some(web_plugin) = self.try_get_web_plugin(&plugin) {
                // Load endpoints and components
                let endpoints = web_plugin.get_endpoints();
                let components = web_plugin.get_components();

                // Store endpoints and components
                let mut endpoint_map = self.endpoints.write().await;
                let mut component_map = self.components.write().await;
                let mut route_map = self.routes.write().await;

                // Cache route patterns for faster matching
                for endpoint in &endpoints {
                    let pattern = endpoint.path.clone();
                    if !route_map.contains_key(&pattern) {
                        route_map.insert(pattern.clone(), Route::new(&pattern));
                    }
                }

                endpoint_map.insert(plugin.metadata().id, endpoints);
                component_map.insert(plugin.metadata().id, components);

                count += 1;
            }
        }

        Ok(count)
    }

    /// Try to get a plugin as a `WebPlugin`
    fn try_get_web_plugin<'a>(
        &self,
        plugin: &'a Arc<dyn crate::plugin::Plugin>,
    ) -> Option<&'a dyn WebPlugin> {
        // Try ExampleWebPlugin
        if let Some(ex_plugin) = plugin
            .as_any()
            .downcast_ref::<crate::web::ExampleWebPlugin>()
        {
            return Some(ex_plugin);
        }

        // Try other specific implementations
        // (Add additional concrete plugin types here as needed)

        // If no concrete type matches, we can't automatically convert
        None
    }

    /// Get all web plugins
    pub async fn get_web_plugins(&self) -> Vec<Arc<dyn WebPlugin>> {
        let plugins = self.registry.get_plugins().await.unwrap_or_default();
        let mut web_plugins: Vec<Arc<dyn WebPlugin>> = Vec::new();

        for plugin in plugins {
            if self.try_get_web_plugin(&plugin).is_some()
                && let Some(_ex_plugin) = plugin
                    .as_any()
                    .downcast_ref::<Box<dyn crate::web::WebPlugin>>()
            {
                // Create a new arc with the same instance
                let example_plugin = Arc::new(crate::web::ExampleWebPlugin::new());
                web_plugins.push(example_plugin as Arc<dyn WebPlugin>);
            }
            // Add other specific types here as needed
        }

        web_plugins
    }

    /// Find the web plugin that handles the specified endpoint
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if no plugin handles the path and method.
    pub async fn find_plugin_for_endpoint(
        &self,
        path: &str,
        method: HttpMethod,
    ) -> Result<Arc<dyn WebPlugin>> {
        // Get all web plugins
        let web_plugins = self.get_web_plugins().await;

        // Find the plugin that supports the endpoint
        for plugin in &web_plugins {
            if self.supports_endpoint(plugin, path, method) {
                return Ok(plugin.clone());
            }
        }

        // Check if any plugin handles this path with a different method
        for plugin in &web_plugins {
            for endpoint in plugin.get_endpoints() {
                if endpoint.path == path {
                    return Err(WebPluginError::EndpointNotFound(
                        path.to_string(),
                        method.to_string(),
                    )
                    .into());
                }
            }
        }

        // No plugin found for this endpoint
        Err(WebPluginError::EndpointNotFound(path.to_string(), method.to_string()).into())
    }

    /// Check if a plugin supports a specific endpoint
    fn supports_endpoint(
        &self,
        plugin: &Arc<dyn WebPlugin>,
        path: &str,
        method: HttpMethod,
    ) -> bool {
        for endpoint in plugin.get_endpoints() {
            if endpoint.path == path && endpoint.method == method {
                return true;
            }
        }
        false
    }

    /// Find the web plugin that handles the specified component
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if no plugin provides the component.
    pub async fn find_plugin_for_component(
        &self,
        component_id: &Uuid,
    ) -> Result<Arc<dyn WebPlugin>> {
        // Get all web plugins
        let web_plugins = self.get_web_plugins().await;

        // Find the plugin that supports the component
        for plugin in &web_plugins {
            if self.supports_component(plugin, component_id) {
                return Ok(plugin.clone());
            }
        }

        // No plugin found for this component
        Err(WebPluginError::ComponentNotFound(component_id.to_string()).into())
    }

    /// Check if a plugin supports a specific component
    fn supports_component(&self, plugin: &Arc<dyn WebPlugin>, component_id: &Uuid) -> bool {
        for component in plugin.get_components() {
            if component.id == *component_id {
                return true;
            }
        }
        false
    }

    /// Find a plugin for a path, with route parameter extraction support
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if no matching route is found.
    pub async fn find_plugin_for_path(
        &self,
        path: &str,
        method: HttpMethod,
    ) -> Result<(Arc<dyn WebPlugin>, Option<HashMap<String, String>>)> {
        // First try exact path match
        if let Ok(plugin) = self.find_plugin_for_endpoint(path, method).await {
            return Ok((plugin, None));
        }

        // If no exact match found, try pattern matching
        let routes = self.routes.read().await;
        let endpoints = self.endpoints.read().await;

        for (plugin_id, plugin_endpoints) in endpoints.iter() {
            for endpoint in plugin_endpoints {
                if endpoint.method != method {
                    continue;
                }

                if let Some(route) = routes.get(&endpoint.path)
                    && route.matches(path)
                    && let Some(params) = route.extract_params(path)
                {
                    // Lookup the plugin and create appropriate instance
                    for plugin in self.get_web_plugins().await {
                        if plugin.metadata().id == *plugin_id {
                            return Ok((plugin, Some(params)));
                        }
                    }
                }
            }
        }

        // No matching route found
        Err(WebPluginError::EndpointNotFound(path.to_string(), method.to_string()).into())
    }

    /// Handle a web request
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if no plugin matches or the plugin handler fails.
    pub async fn handle_request(&self, mut request: WebRequest) -> Result<WebResponse> {
        // Try to find exact matching endpoint
        if let Ok(plugin) = self
            .find_plugin_for_endpoint(&request.path, request.method)
            .await
        {
            return plugin.handle_request(request).await;
        }
        // Continue to pattern matching below

        // If not found, try pattern matching
        let path_result = self
            .find_plugin_for_path(&request.path, request.method)
            .await;

        if let Ok((plugin, params)) = path_result {
            // Add route parameters to request
            if let Some(route_params) = params {
                request.route_params = route_params;
            }

            return plugin.handle_request(request).await;
        }

        // No matching endpoint found
        Err(
            WebPluginError::EndpointNotFound(request.path.clone(), request.method.to_string())
                .into(),
        )
    }

    /// Get component markup
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if the component cannot be resolved or rendering fails.
    pub async fn get_component_markup(&self, component_id: Uuid, props: Value) -> Result<String> {
        let plugin = self.find_plugin_for_component(&component_id).await?;
        plugin.get_component_markup(component_id, props).await
    }

    /// Get all endpoints from all plugins
    pub async fn get_endpoints(&self) -> Vec<WebEndpoint> {
        let endpoints_map = self.endpoints.read().await;
        let mut result = Vec::new();

        for endpoints in endpoints_map.values() {
            result.extend(endpoints.clone());
        }

        result
    }

    /// Get all components from all plugins
    pub async fn get_components(&self) -> Vec<WebComponent> {
        let components_map = self.components.read().await;
        let mut result = Vec::new();

        for components in components_map.values() {
            result.extend(components.clone());
        }

        result
    }
}
