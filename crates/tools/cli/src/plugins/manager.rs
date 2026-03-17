// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::commands::registry::CommandRegistry;
use squirrel_commands::Command;
use tokio::runtime::Runtime;
use tracing::{debug, error, info, warn};

use super::security::SecurePluginLoader;
use crate::plugins::error::PluginError;
use crate::plugins::plugin::Plugin;
use crate::plugins::plugin::{PluginFactory, PluginItem, PluginMetadata, PluginStatus};
use crate::plugins::state::PluginState;

/// A secure manager for Squirrel plugins
pub struct PluginManager {
    /// The installed plugins
    plugins: HashMap<String, PluginItem>,
    /// The loaded plugin instances
    loaded_plugins: HashMap<String, Arc<dyn Plugin>>,
    /// The plugin states
    plugin_states: HashMap<String, PluginState>,
    /// Secure plugin loader (replaces unsafe dynamic loading)
    secure_loader: SecurePluginLoader,
    /// Plugin factories (maintained for compatibility)
    plugin_factories: HashMap<String, Arc<dyn PluginFactory>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            loaded_plugins: HashMap::new(),
            plugin_states: HashMap::new(),
            secure_loader: SecurePluginLoader::new(),
            plugin_factories: HashMap::new(),
        }
    }

    /// List all installed plugins
    pub fn list_plugins(&self) -> Vec<&PluginItem> {
        self.plugins.values().collect()
    }

    /// Get a plugin by name
    pub fn get_plugin(&self, name: &str) -> Result<&PluginItem, PluginError> {
        self.plugins
            .get(name)
            .ok_or_else(|| PluginError::plugin_not_found(name))
    }

    /// Get a mutable reference to a plugin by name
    pub fn get_plugin_mut(&mut self, name: &str) -> Result<&mut PluginItem, PluginError> {
        self.plugins
            .get_mut(name)
            .ok_or_else(|| PluginError::plugin_not_found(name))
    }

    /// Add a new plugin to the manager
    pub fn add_plugin(
        &mut self,
        metadata: PluginMetadata,
        path: PathBuf,
        status: PluginStatus,
    ) -> Result<&PluginItem, PluginError> {
        let name = metadata.name.clone();

        if self.plugins.contains_key(&name) {
            return Err(PluginError::plugin_already_exists(&name));
        }

        let plugin = PluginItem::new(metadata, path, status);
        self.plugins.insert(name.clone(), plugin);
        self.plugin_states
            .insert(name.clone(), PluginState::Created);

        // Safe: We just inserted the plugin, so it must exist
        self.plugins
            .get(&name)
            .ok_or_else(|| PluginError::plugin_not_found(&name))
    }

    /// Remove a plugin from the manager
    pub fn remove_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        if !self.plugins.contains_key(name) {
            return Err(PluginError::plugin_not_found(name));
        }

        // Clean up loaded plugin if it exists
        if let Some(plugin) = self.loaded_plugins.remove(name) {
            let rt = Runtime::new()
                .map_err(|e| PluginError::Unknown(format!("Failed to create runtime: {}", e)))?;

            // Call cleanup but don't propagate errors
            if let Err(e) = rt.block_on(plugin.cleanup()) {
                warn!("Error cleaning up plugin {}: {}", name, e);
            }
        }

        self.plugins.remove(name);
        self.plugin_states.remove(name);
        Ok(())
    }

    /// Load a plugin by name
    ///
    /// This method loads and initializes a plugin.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the plugin to load
    ///
    /// # Returns
    ///
    /// `Ok(())` if loading succeeds, or an error otherwise
    pub async fn load_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        // Check if plugin exists
        let _plugin_item = self.get_plugin(name)?;

        // Check if plugin is already loaded
        if self.loaded_plugins.contains_key(name) {
            return Ok(());
        }

        // Create a runtime for async operations
        let rt = Runtime::new()
            .map_err(|e| PluginError::Unknown(format!("Failed to create runtime: {}", e)))?;

        let plugin = if let Some(factory) = self.plugin_factories.get(name) {
            // Create plugin from factory
            factory.create()?
        } else {
            // Get the plugin path and load from filesystem
            let plugin_path = _plugin_item.path().to_path_buf();
            self.load_plugin_from_path(name, &plugin_path).await?
        };

        // Initialize the plugin
        match rt.block_on(plugin.initialize()) {
            Ok(()) => {
                info!("Plugin {} initialized successfully", name);
                self.loaded_plugins.insert(name.to_string(), plugin);

                // Update plugin state
                self.plugin_states.insert(
                    name.to_string(),
                    crate::plugins::state::PluginState::Initialized,
                );

                // Update plugin status
                let plugin_item = self.get_plugin_mut(name)?;
                plugin_item.set_status(PluginStatus::Enabled);

                Ok(())
            }
            Err(e) => {
                error!("Failed to initialize plugin {}: {}", name, e);

                // Update plugin status
                let plugin_item = self.get_plugin_mut(name)?;
                plugin_item.set_status(PluginStatus::Failed(e.to_string()));

                Err(PluginError::InitError(e.to_string()))
            }
        }
    }

    /// Securely load a plugin from a path (replaces unsafe implementation)
    ///
    /// # Arguments
    ///
    /// * `name` - The plugin name
    /// * `path` - The plugin directory path
    ///
    /// # Returns
    ///
    /// The loaded plugin instance or an error
    async fn load_plugin_from_path(
        &mut self,
        name: &str,
        path: &Path,
    ) -> Result<Arc<dyn Plugin>, PluginError> {
        // Get plugin metadata
        let metadata = self.get_plugin_metadata(name, path)?;

        // Use secure loader instead of unsafe dynamic loading
        match self.secure_loader.load_plugin_secure(path, &metadata).await {
            Ok(plugin) => {
                info!("🔒 Securely loaded plugin: {}", name);
                Ok(plugin)
            }
            Err(security_error) => {
                error!(
                    "🚨 Plugin security validation failed for {}: {}",
                    name, security_error
                );
                Err(PluginError::SecurityError(format!(
                    "Security validation failed: {}",
                    security_error
                )))
            }
        }
    }

    /// Get plugin metadata from plugin directory
    fn get_plugin_metadata(&self, name: &str, path: &Path) -> Result<PluginMetadata, PluginError> {
        // Look for plugin.toml or other metadata file
        let metadata_file = path.join("plugin.toml");

        if metadata_file.exists() {
            // Parse metadata from TOML file
            let _metadata_content =
                std::fs::read_to_string(&metadata_file).map_err(PluginError::IoError)?;

            // For now, create basic metadata - NOTE(phase2): implement TOML manifest parsing
            Ok(PluginMetadata {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                description: Some(format!("Plugin loaded from {}", path.display())),
                author: Some("Unknown".to_string()),
                homepage: None,
            })
        } else {
            // Create basic metadata if no file exists
            warn!(
                "⚠️ No metadata file found for plugin {}, using defaults",
                name
            );
            Ok(PluginMetadata {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                description: Some(format!("Plugin loaded from {}", path.display())),
                author: Some("Unknown".to_string()),
                homepage: None,
            })
        }
    }

    /// Register all commands from loaded plugins with the command registry
    ///
    /// # Arguments
    ///
    /// * `registry` - The command registry to register commands with
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all commands were registered successfully
    /// * `Err(PluginError)` if any command registration failed
    pub fn register_plugin_commands(
        &self,
        registry: &Arc<CommandRegistry>,
    ) -> Result<(), PluginError> {
        debug!("Registering commands from loaded plugins...");

        if self.loaded_plugins.is_empty() {
            debug!("No plugins loaded, no commands to register");
            return Ok(());
        }

        let mut success_count = 0;
        let mut failures = Vec::new();

        for (plugin_name, plugin) in &self.loaded_plugins {
            debug!("Getting commands from plugin: {}", plugin_name);

            let commands = plugin.commands();
            if commands.is_empty() {
                debug!("Plugin '{}' did not provide any commands", plugin_name);
                continue;
            }

            for command in commands {
                let cmd_name = command.name();
                debug!(
                    "Registering command '{}' from plugin '{}'",
                    cmd_name, plugin_name
                );

                match registry.register(cmd_name, command.clone()) {
                    Ok(_) => {
                        success_count += 1;
                        debug!(
                            "Command '{}' from plugin '{}' registered successfully",
                            cmd_name, plugin_name
                        );
                    }
                    Err(err) => {
                        let error_msg = format!(
                            "Failed to register command '{}' from plugin '{}': {}",
                            cmd_name, plugin_name, err
                        );
                        warn!("{}", error_msg);
                        failures.push(error_msg);
                    }
                }
            }
        }

        info!("Registered {} commands from plugins", success_count);

        if !failures.is_empty() {
            warn!("{} command registration failures", failures.len());
            for failure in &failures {
                warn!("  {}", failure);
            }

            return Err(PluginError::RegisterError(format!(
                "Failed to register {} out of {} commands from plugins",
                failures.len(),
                success_count + failures.len()
            )));
        }

        Ok(())
    }

    /// Start all plugins
    ///
    /// This method transitions all initialized plugins from Initialized to Started state.
    ///
    /// # Returns
    ///
    /// `Ok(())` if starting succeeds, or an error otherwise
    pub fn start_plugins(&mut self) -> Result<(), PluginError> {
        // Remove the runtime creation, as we're already in an async context
        // from the #[tokio::main] in main.rs

        let mut failed_plugins = Vec::new();

        for name in self.loaded_plugins.keys() {
            let plugin_state = self
                .plugin_states
                .get(name)
                .cloned()
                .unwrap_or(PluginState::Created);

            // Only start plugins that are in the Initialized state
            if plugin_state == PluginState::Initialized {
                info!("Starting plugin {}", name);

                // In a real implementation, this would call a start method on the plugin
                // For now, just update the state
                if PluginState::is_valid_transition(plugin_state, PluginState::Started) {
                    self.plugin_states
                        .insert(name.clone(), PluginState::Started);
                } else {
                    warn!(
                        "Invalid state transition for plugin {}: {:?} -> {:?}",
                        name,
                        plugin_state,
                        PluginState::Started
                    );
                    failed_plugins.push(name.clone());
                }
            }
        }

        if failed_plugins.is_empty() {
            Ok(())
        } else {
            Err(PluginError::InitError(format!(
                "Failed to start plugins: {:?}",
                failed_plugins
            )))
        }
    }

    /// Stop all plugins
    ///
    /// This method transitions all started plugins from Started to Stopped state.
    ///
    /// # Returns
    ///
    /// `Ok(())` if stopping succeeds, or an error otherwise
    pub fn stop_plugins(&mut self) -> Result<(), PluginError> {
        // Remove the runtime creation, as we're already in an async context
        // from the #[tokio::main] in main.rs

        let mut failed_plugins = Vec::new();

        for name in self.loaded_plugins.keys() {
            let plugin_state = self
                .plugin_states
                .get(name)
                .cloned()
                .unwrap_or(PluginState::Created);

            // Only stop plugins that are in the Started state
            if plugin_state == PluginState::Started {
                info!("Stopping plugin {}", name);

                // In a real implementation, this would call a stop method on the plugin
                // For now, just update the state
                if PluginState::is_valid_transition(plugin_state, PluginState::Stopped) {
                    self.plugin_states
                        .insert(name.clone(), PluginState::Stopped);
                } else {
                    warn!(
                        "Invalid state transition for plugin {}: {:?} -> {:?}",
                        name,
                        plugin_state,
                        PluginState::Stopped
                    );
                    failed_plugins.push(name.clone());
                }
            }
        }

        if failed_plugins.is_empty() {
            Ok(())
        } else {
            Err(PluginError::InitError(format!(
                "Failed to stop plugins: {:?}",
                failed_plugins
            )))
        }
    }

    /// Unload all plugins and clean up resources
    pub async fn unload_plugins(&mut self) -> Result<(), PluginError> {
        info!("Unloading all plugins");

        // We can't create a runtime here as we're already in an async context
        // Instead, we'll use the current runtime context

        // Get a list of all plugin names
        let plugin_names: Vec<String> = self.loaded_plugins.keys().cloned().collect();

        // Track success/failure counts
        let mut success_count = 0;
        let mut failure_count = 0;

        // Unload each plugin
        for name in plugin_names {
            if let Some(plugin) = self.loaded_plugins.remove(&name) {
                // Call cleanup directly (no need for block_on as we're in async context)
                match plugin.cleanup().await {
                    Ok(()) => {
                        info!("Plugin {} unloaded successfully", name);
                        success_count += 1;

                        // Update plugin state and status
                        if let Ok(plugin_item) = self.get_plugin_mut(&name) {
                            plugin_item.set_status(PluginStatus::Disabled);
                        }
                    }
                    Err(e) => {
                        error!("Failed to unload plugin {}: {}", name, e);
                        failure_count += 1;

                        // Update plugin state and status
                        if let Ok(plugin_item) = self.get_plugin_mut(&name) {
                            plugin_item.set_status(PluginStatus::Failed(e.to_string()));
                        }
                    }
                }

                // Remove the plugin library
                // self.libraries.remove(&name); // This line is no longer needed
            }
        }

        // Update plugin states
        let state_keys: Vec<String> = self.plugin_states.keys().cloned().collect();
        for name in state_keys {
            self.plugin_states.insert(name, PluginState::Created);
        }

        info!(
            "Unloaded {} plugins ({} succeeded, {} failed)",
            success_count + failure_count,
            success_count,
            failure_count
        );

        Ok(())
    }

    /// Create a test plugin instance for testing
    #[expect(dead_code, reason = "public API — used for testing plugin infrastructure")]
    fn create_test_plugin(
        &self,
        name: String,
        path: PathBuf,
    ) -> Result<Arc<dyn Plugin>, PluginError> {
        struct TestPlugin {
            name: String,
            _path: PathBuf,
        }

        #[async_trait::async_trait]
        impl Plugin for TestPlugin {
            fn name(&self) -> &str {
                &self.name
            }

            fn version(&self) -> &str {
                "0.1.0"
            }

            fn description(&self) -> Option<&str> {
                Some("A test plugin")
            }

            async fn initialize(&self) -> Result<(), PluginError> {
                debug!("Test plugin {} initialized", self.name);
                Ok(())
            }

            fn register_commands(&self, _registry: &CommandRegistry) -> Result<(), PluginError> {
                debug!("Test plugin {} registered commands", self.name);
                Ok(())
            }

            fn commands(&self) -> Vec<Arc<dyn Command>> {
                // Return an empty vector for test plugin
                Vec::new()
            }

            async fn execute(&self, args: &[String]) -> Result<String, PluginError> {
                debug!("Test plugin {} executed with args: {:?}", self.name, args);
                Ok(format!("Test plugin {} executed", self.name))
            }

            async fn cleanup(&self) -> Result<(), PluginError> {
                debug!("Test plugin {} cleaned up", self.name);
                Ok(())
            }
        }

        Ok(Arc::new(TestPlugin { name, _path: path }))
    }

    /// Register a plugin factory for creating plugins
    ///
    /// # Arguments
    ///
    /// * `name` - The name to register the plugin factory under
    /// * `factory` - The plugin factory to register
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error if registration fails
    pub fn register_plugin_factory(
        &mut self,
        name: &str,
        factory: Arc<dyn PluginFactory>,
    ) -> Result<(), PluginError> {
        info!("Registering plugin factory: {}", name);

        // Create the plugin instance
        let plugin = factory.create()?;

        // Create metadata from the plugin
        let metadata = PluginMetadata {
            name: plugin.name().to_string(),
            version: plugin.version().to_string(),
            description: plugin.description().map(|s| s.to_string()),
            author: None,   // Factories don't provide author
            homepage: None, // Factories don't provide homepage
        };

        // Create a plugin item
        let plugin_path = PathBuf::from("built-in"); // Built-in plugins don't have a path
        let plugin_item = PluginItem::new(metadata, plugin_path, PluginStatus::Installed);

        // Add to plugins list
        self.plugins.insert(name.to_string(), plugin_item);

        // Store the factory
        self.plugin_factories.insert(name.to_string(), factory);

        // Track plugin state
        self.plugin_states.insert(
            name.to_string(),
            crate::plugins::state::PluginState::Created,
        );

        Ok(())
    }

    /// Start a plugin by name
    ///
    /// This method starts a loaded plugin.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the plugin to start
    ///
    /// # Returns
    ///
    /// `Ok(())` if starting succeeds, or an error otherwise
    pub fn start_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        // Check if plugin exists and is loaded
        let _plugin_item = self.get_plugin(name)?;

        // Check if plugin is loaded
        if !self.loaded_plugins.contains_key(name) {
            return Err(PluginError::LoadError(format!(
                "Plugin {} is not loaded",
                name
            )));
        }

        // Create a runtime for async operations
        let rt = Runtime::new()
            .map_err(|e| PluginError::Unknown(format!("Failed to create runtime: {}", e)))?;

        // Get plugin state
        let current_state = self
            .plugin_states
            .get(name)
            .ok_or_else(|| PluginError::Unknown(format!("Plugin {} has no state", name)))?;

        // Check if plugin is already started
        if *current_state == crate::plugins::state::PluginState::Started {
            return Ok(());
        }

        // Check if plugin can be started
        if *current_state != crate::plugins::state::PluginState::Initialized {
            return Err(PluginError::ValidationError(format!(
                "Plugin {} is in state {:?}, must be in Initialized state to start",
                name, current_state
            )));
        }

        // Get plugin instance
        let plugin = self.loaded_plugins.get(name).ok_or_else(|| {
            PluginError::Unknown(format!("Plugin {} not found in loaded plugins", name))
        })?;

        // Start the plugin
        match rt.block_on(plugin.start()) {
            Ok(()) => {
                info!("Plugin {} started successfully", name);

                // Update plugin state
                self.plugin_states.insert(
                    name.to_string(),
                    crate::plugins::state::PluginState::Started,
                );

                // Update plugin status
                let plugin_item = self.get_plugin_mut(name)?;
                plugin_item.set_status(PluginStatus::Enabled);

                Ok(())
            }
            Err(e) => {
                error!("Failed to start plugin {}: {}", name, e);

                // Update plugin status
                let plugin_item = self.get_plugin_mut(name)?;
                plugin_item.set_status(PluginStatus::Failed(e.to_string()));

                Err(e)
            }
        }
    }

    /// Stop a plugin by name
    ///
    /// This method stops a running plugin.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the plugin to stop
    ///
    /// # Returns
    ///
    /// `Ok(())` if stopping succeeds, or an error otherwise
    pub fn stop_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        // Check if plugin exists
        let _plugin_item = self.get_plugin(name)?;

        // Check if plugin is loaded
        if !self.loaded_plugins.contains_key(name) {
            return Err(PluginError::LoadError(format!(
                "Plugin {} is not loaded",
                name
            )));
        }

        // Create a runtime for async operations
        let rt = Runtime::new()
            .map_err(|e| PluginError::Unknown(format!("Failed to create runtime: {}", e)))?;

        // Get plugin state
        let current_state = self
            .plugin_states
            .get(name)
            .ok_or_else(|| PluginError::Unknown(format!("Plugin {} has no state", name)))?;

        // Check if plugin is already stopped
        if *current_state == crate::plugins::state::PluginState::Stopped {
            return Ok(());
        }

        // Check if plugin can be stopped
        if *current_state != crate::plugins::state::PluginState::Started {
            return Err(PluginError::ValidationError(format!(
                "Plugin {} is in state {:?}, must be in Started state to stop",
                name, current_state
            )));
        }

        // Get plugin instance
        let plugin = self.loaded_plugins.get(name).ok_or_else(|| {
            PluginError::Unknown(format!("Plugin {} not found in loaded plugins", name))
        })?;

        // Stop the plugin
        match rt.block_on(plugin.stop()) {
            Ok(()) => {
                info!("Plugin {} stopped successfully", name);

                // Update plugin state
                self.plugin_states.insert(
                    name.to_string(),
                    crate::plugins::state::PluginState::Stopped,
                );

                // Update plugin status
                let plugin_item = self.get_plugin_mut(name)?;
                plugin_item.set_status(PluginStatus::Disabled);

                Ok(())
            }
            Err(e) => {
                error!("Failed to stop plugin {}: {}", name, e);

                Err(e)
            }
        }
    }

    /// Remove the plugin library reference when unloading
    pub fn unload_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        // Remove from loaded plugins
        if let Some(_plugin) = self.loaded_plugins.remove(name) {
            info!("🔄 Unloading plugin: {}", name);

            // Update plugin state
            self.plugin_states
                .insert(name.to_string(), PluginState::Stopped);

            // Remove from state tracking - no more library cleanup needed
            info!("✅ Plugin {} unloaded successfully", name);
            Ok(())
        } else {
            Err(PluginError::NotFound(format!("Plugin {} not loaded", name)))
        }
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a default plugin manager
pub fn create_plugin_manager() -> PluginManager {
    PluginManager::new()
}

/// Initialize the plugin system
pub fn initialize_plugins() -> Result<(), PluginError> {
    Ok(())
}
