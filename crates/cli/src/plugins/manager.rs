use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::runtime::Runtime;
use log::{info, warn, error, debug};
use libloading::Library;

use crate::plugins::{PluginItem, PluginMetadata, PluginStatus};
use crate::plugins::plugin::{Plugin, PluginFactory};
use crate::plugins::error::PluginError;
use crate::plugins::state::PluginState;
use squirrel_commands::{Command, CommandRegistry};

/// Type for a plugin create function
type PluginCreateFn = unsafe fn() -> Result<Arc<dyn Plugin>, PluginError>;

/// Type for a plugin factory registration function
type PluginFactoryRegisterFn = unsafe fn() -> Arc<dyn PluginFactory>;

/// A manager for Squirrel plugins
pub struct PluginManager {
    /// The installed plugins
    plugins: HashMap<String, PluginItem>,
    /// The loaded plugin instances
    loaded_plugins: HashMap<String, Arc<dyn Plugin>>,
    /// The plugin states
    plugin_states: HashMap<String, PluginState>,
    /// Plugin libraries
    libraries: HashMap<String, Library>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            loaded_plugins: HashMap::new(),
            plugin_states: HashMap::new(),
            libraries: HashMap::new(),
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
    pub fn add_plugin(&mut self, metadata: PluginMetadata, path: PathBuf, status: PluginStatus) -> Result<&PluginItem, PluginError> {
        let name = metadata.name.clone();
        
        if self.plugins.contains_key(&name) {
            return Err(PluginError::plugin_already_exists(&name));
        }
        
        let plugin = PluginItem::new(metadata, path, status);
        self.plugins.insert(name.clone(), plugin);
        self.plugin_states.insert(name.clone(), PluginState::Created);
        
        Ok(self.plugins.get(&name).unwrap())
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
    pub fn load_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        // Check if plugin exists
        let plugin_item = self.get_plugin(name)?;
        
        // Check if plugin is already loaded
        if self.loaded_plugins.contains_key(name) {
            return Ok(());
        }
        
        // Create a runtime for async operations
        let rt = Runtime::new()
            .map_err(|e| PluginError::Unknown(format!("Failed to create runtime: {}", e)))?;
        
        // Get the plugin path
        let plugin_path = plugin_item.path().to_path_buf();
        
        // Load the plugin
        let plugin = self.load_plugin_from_path(name, &plugin_path)?;
        
        // Initialize the plugin
        match rt.block_on(plugin.initialize()) {
            Ok(()) => {
                info!("Plugin {} initialized successfully", name);
                self.loaded_plugins.insert(name.to_string(), plugin);
                
                // Update plugin state
                self.plugin_states.insert(name.to_string(), PluginState::Initialized);
                
                // Update plugin status
                let plugin_item = self.get_plugin_mut(name)?;
                plugin_item.set_status(PluginStatus::Enabled);
                
                Ok(())
            },
            Err(e) => {
                error!("Failed to initialize plugin {}: {}", name, e);
                
                // Update plugin status
                let plugin_item = self.get_plugin_mut(name)?;
                plugin_item.set_status(PluginStatus::Failed(e.to_string()));
                
                Err(e)
            }
        }
    }
    
    /// Load a plugin from a path
    ///
    /// This method attempts to load a plugin library from a path.
    /// If dynamic loading is not available, it falls back to the test plugin.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the plugin
    /// * `path` - The path to the plugin directory
    ///
    /// # Returns
    ///
    /// The loaded plugin instance or an error
    fn load_plugin_from_path(&mut self, name: &str, path: &Path) -> Result<Arc<dyn Plugin>, PluginError> {
        // Look for shared library in the plugin path
        let lib_path = self.find_plugin_library(path, name)?;
        
        // Try to load the library
        match unsafe { Library::new(&lib_path) } {
            Ok(lib) => {
                // Try to get the create_plugin symbol
                match unsafe { lib.get::<PluginCreateFn>(b"create_plugin") } {
                    Ok(create_fn) => {
                        // Call the create function
                        let plugin = unsafe { create_fn() }?;
                        
                        // Store the library
                        self.libraries.insert(name.to_string(), lib);
                        
                        Ok(plugin)
                    },
                    Err(e) => {
                        warn!("Could not find create_plugin function in {}: {}", lib_path.display(), e);
                        
                        // Try to get the register_plugin_factory symbol
                        match unsafe { lib.get::<PluginFactoryRegisterFn>(b"register_plugin_factory") } {
                            Ok(register_fn) => {
                                // Call the register function to get the factory
                                let factory = unsafe { register_fn() };
                                
                                // Create a plugin instance from the factory
                                let plugin = factory.create()?;
                                
                                // Store the library
                                self.libraries.insert(name.to_string(), lib);
                                
                                Ok(plugin)
                            },
                            Err(e) => {
                                warn!("Could not find register_plugin_factory function in {}: {}", lib_path.display(), e);
                                
                                // Fall back to test plugin
                                warn!("Falling back to test plugin for {}", name);
                                self.create_test_plugin(name.to_string(), path.to_path_buf())
                            }
                        }
                    }
                }
            },
            Err(e) => {
                warn!("Could not load plugin library from {}: {}", lib_path.display(), e);
                
                // Fall back to test plugin
                warn!("Falling back to test plugin for {}", name);
                self.create_test_plugin(name.to_string(), path.to_path_buf())
            }
        }
    }
    
    /// Find the plugin library file in the plugin directory
    ///
    /// # Arguments
    ///
    /// * `plugin_dir` - The plugin directory
    /// * `name` - The plugin name
    ///
    /// # Returns
    ///
    /// The path to the plugin library or an error
    fn find_plugin_library(&self, plugin_dir: &Path, name: &str) -> Result<PathBuf, PluginError> {
        // Check if plugin directory exists
        if !plugin_dir.exists() || !plugin_dir.is_dir() {
            return Err(PluginError::NotFound(format!("Plugin directory not found: {}", plugin_dir.display())));
        }
        
        // Look for the library file with platform-specific extension
        let lib_name = format!("lib{}", name.replace('-', "_"));
        
        #[cfg(target_os = "linux")]
        let extensions = vec![".so"];
        
        #[cfg(target_os = "macos")]
        let extensions = vec![".dylib", ".so"];
        
        #[cfg(target_os = "windows")]
        let extensions = vec![".dll"];
        
        // First, look for the library file in the lib subdirectory
        let lib_dir = plugin_dir.join("lib");
        if lib_dir.exists() && lib_dir.is_dir() {
            // Check each extension
            for ext in &extensions {
                let lib_path = lib_dir.join(format!("{}{}", lib_name, ext));
                if lib_path.exists() {
                    return Ok(lib_path);
                }
            }
        }
        
        // Then, look for the library file in the plugin directory
        for ext in &extensions {
            let lib_path = plugin_dir.join(format!("{}{}", lib_name, ext));
            if lib_path.exists() {
                return Ok(lib_path);
            }
        }
        
        // Finally, look for a file with the plugin name and any of the extensions
        for ext in &extensions {
            let lib_path = plugin_dir.join(format!("{}{}", name.replace('-', "_"), ext));
            if lib_path.exists() {
                return Ok(lib_path);
            }
        }
        
        Err(PluginError::NotFound(format!("No library file found for plugin {} in {}", name, plugin_dir.display())))
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
    pub fn register_plugin_commands(&self, registry: &Arc<CommandRegistry>) -> Result<(), PluginError> {
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
                debug!("Registering command '{}' from plugin '{}'", cmd_name, plugin_name);
                
                match registry.register(cmd_name, command.clone()) {
                    Ok(_) => {
                        success_count += 1;
                        debug!("Command '{}' from plugin '{}' registered successfully", cmd_name, plugin_name);
                    },
                    Err(err) => {
                        let error_msg = format!("Failed to register command '{}' from plugin '{}': {}", 
                            cmd_name, plugin_name, err);
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
    
    /// Start all loaded plugins
    ///
    /// This method transitions all loaded plugins from Initialized to Started state.
    ///
    /// # Returns
    ///
    /// `Ok(())` if starting succeeds, or an error otherwise
    pub fn start_plugins(&mut self) -> Result<(), PluginError> {
        let _rt = Runtime::new()
            .map_err(|e| PluginError::Unknown(format!("Failed to create runtime: {}", e)))?;
        
        let mut failed_plugins = Vec::new();
        
        for name in self.loaded_plugins.keys() {
            let plugin_state = self.plugin_states.get(name).cloned().unwrap_or(PluginState::Created);
            
            // Only start plugins that are in the Initialized state
            if plugin_state == PluginState::Initialized {
                info!("Starting plugin {}", name);
                
                // In a real implementation, this would call a start method on the plugin
                // For now, just update the state
                if PluginState::is_valid_transition(plugin_state, PluginState::Started) {
                    self.plugin_states.insert(name.clone(), PluginState::Started);
                } else {
                    warn!("Invalid state transition for plugin {}: {:?} -> {:?}", 
                         name, plugin_state, PluginState::Started);
                    failed_plugins.push(name.clone());
                }
            }
        }
        
        if failed_plugins.is_empty() {
            Ok(())
        } else {
            Err(PluginError::InitError(format!("Failed to start plugins: {:?}", failed_plugins)))
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
        let _rt = Runtime::new()
            .map_err(|e| PluginError::Unknown(format!("Failed to create runtime: {}", e)))?;
        
        let mut failed_plugins = Vec::new();
        
        for name in self.loaded_plugins.keys() {
            let plugin_state = self.plugin_states.get(name).cloned().unwrap_or(PluginState::Created);
            
            // Only stop plugins that are in the Started state
            if plugin_state == PluginState::Started {
                info!("Stopping plugin {}", name);
                
                // In a real implementation, this would call a stop method on the plugin
                // For now, just update the state
                if PluginState::is_valid_transition(plugin_state, PluginState::Stopped) {
                    self.plugin_states.insert(name.clone(), PluginState::Stopped);
                } else {
                    warn!("Invalid state transition for plugin {}: {:?} -> {:?}", 
                         name, plugin_state, PluginState::Stopped);
                    failed_plugins.push(name.clone());
                }
            }
        }
        
        if failed_plugins.is_empty() {
            Ok(())
        } else {
            Err(PluginError::InitError(format!("Failed to stop plugins: {:?}", failed_plugins)))
        }
    }
    
    /// Unload all plugins and clean up resources
    pub fn unload_plugins(&mut self) -> Result<(), PluginError> {
        info!("Unloading all plugins");
        
        // Create a runtime for async operations
        let rt = Runtime::new()
            .map_err(|e| PluginError::Unknown(format!("Failed to create runtime: {}", e)))?;
        
        // Get a list of all plugin names
        let plugin_names: Vec<String> = self.loaded_plugins.keys().cloned().collect();
        
        // Track success/failure counts
        let mut success_count = 0;
        let mut failure_count = 0;
        
        // Unload each plugin
        for name in plugin_names {
            if let Some(plugin) = self.loaded_plugins.remove(&name) {
                // Call cleanup
                match rt.block_on(plugin.cleanup()) {
                    Ok(()) => {
                        info!("Plugin {} unloaded successfully", name);
                        success_count += 1;
                        
                        // Update plugin state and status
                        if let Ok(plugin_item) = self.get_plugin_mut(&name) {
                            plugin_item.set_status(PluginStatus::Disabled);
                        }
                    },
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
                self.libraries.remove(&name);
            }
        }
        
        // Update plugin states
        let state_keys: Vec<String> = self.plugin_states.keys().cloned().collect();
        for name in state_keys {
            self.plugin_states.insert(name, PluginState::Created);
        }
        
        info!("Unloaded {} plugins ({} succeeded, {} failed)", 
             success_count + failure_count, success_count, failure_count);
        
        Ok(())
    }
    
    /// Create a test plugin instance for testing
    fn create_test_plugin(&self, name: String, path: PathBuf) -> Result<Arc<dyn Plugin>, PluginError> {
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
            
            fn register_commands(&self, _registry: &mut CommandRegistry) -> Result<(), PluginError> {
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