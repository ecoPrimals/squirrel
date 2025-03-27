//! Plugin loader implementation
//!
//! This module provides functionality for loading plugins from various sources,
//! including built-in plugins, configuration, and dynamic libraries.

use anyhow::{anyhow, Context, Result};
use libloading::{Library, Symbol};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use async_trait::async_trait;
use std::fmt;

use squirrel_interfaces::plugins::{Plugin, PluginMetadata, PluginRegistry};
use crate::plugins::registry::DefaultPluginRegistry;

/// Function signature for plugin creation
type CreatePluginFunc = unsafe fn() -> *mut dyn Plugin;

/// Function signature for plugin metadata retrieval
type GetPluginMetadataFunc = unsafe fn() -> *mut PluginMetadata;

/// Function signature for plugin destruction
type DestroyPluginFunc = unsafe fn(*mut dyn Plugin);

/// Structure to track loaded dynamic libraries
struct LoadedLibrary {
    /// The loaded library instance
    library: Library,
    /// Path to the library file
    path: PathBuf,
    /// Plugin instance
    plugin: Arc<dyn Plugin>,
}

impl Drop for LoadedLibrary {
    fn drop(&mut self) {
        // Call destroy_plugin function from the library
        unsafe {
            if let Ok(destroy_fn) = self.library.get::<Symbol<DestroyPluginFunc>>(b"destroy_plugin") {
                // Get a raw pointer to the plugin
                let plugin_ptr = Arc::as_ptr(&self.plugin) as *mut dyn Plugin;
                // Call the destroy function
                destroy_fn(plugin_ptr);
            } else {
                error!("Failed to find destroy_plugin function in library: {:?}", self.path);
            }
        }
    }
}

/// Create a wrapper around a dynamic plugin that implements `Sized`
/// This allows us to use `register_plugin` with `Arc<dyn Plugin>`
struct DynamicPluginWrapper {
    inner: Arc<dyn Plugin>,
}

impl fmt::Debug for DynamicPluginWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynamicPluginWrapper")
            .field("metadata", &self.inner.metadata())
            .finish()
    }
}

#[async_trait]
impl Plugin for DynamicPluginWrapper {
    fn metadata(&self) -> &PluginMetadata {
        self.inner.metadata()
    }

    async fn initialize(&self) -> Result<()> {
        self.inner.initialize().await
    }

    async fn shutdown(&self) -> Result<()> {
        self.inner.shutdown().await
    }
}

impl DynamicPluginWrapper {
    fn new(plugin: Arc<dyn Plugin>) -> Self {
        Self { inner: plugin }
    }
}

/// Plugin loader for managing plugin loading
pub struct PluginLoader {
    /// Registry to register plugins with
    registry: Arc<DefaultPluginRegistry>,
    /// Loaded dynamic libraries
    loaded_libraries: Mutex<Vec<LoadedLibrary>>,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new(registry: Arc<DefaultPluginRegistry>) -> Self {
        debug!("Creating new PluginLoader");
        Self {
            registry,
            loaded_libraries: Mutex::new(Vec::new()),
        }
    }

    /// Load built-in plugins
    ///
    /// # Returns
    ///
    /// A list of plugin IDs that were loaded
    pub async fn load_builtin_plugins(&self) -> Result<Vec<String>> {
        info!("Loading built-in plugins");
        let mut plugin_ids = Vec::new();

        // Load the commands plugin adapter
        if let Some(commands_plugin) = self.create_commands_plugin_adapter().await? {
            info!("Loading built-in commands plugin: {}", commands_plugin.metadata().name);
            
            // Register the plugin using the registry's register_plugin method with a wrapper
            let wrapped_plugin = Arc::new(DynamicPluginWrapper::new(commands_plugin));
            let id = self.registry.register_plugin(wrapped_plugin).await?;
            info!("Built-in commands plugin registered with ID: {}", id);
            plugin_ids.push(id);
        }

        Ok(plugin_ids)
    }

    /// Create the commands plugin adapter
    ///
    /// # Returns
    ///
    /// An optional Arc<dyn Plugin> containing the commands plugin adapter
    async fn create_commands_plugin_adapter(&self) -> Result<Option<Arc<dyn Plugin>>> {
        // Use conditional compilation to only include this code if the commands feature is enabled
        #[cfg(feature = "commands")]
        {
            use squirrel_commands::adapter::plugins::create_commands_plugin_adapter;
            use squirrel_commands::factory::create_command_registry;
            use std::sync::Mutex;

            debug!("Creating CommandsPluginAdapter");
            let registry = Arc::new(Mutex::new(create_command_registry()?));
            let adapter = create_commands_plugin_adapter(registry);

            return Ok(Some(adapter));
        }

        // If the commands feature is not enabled, return None
        #[cfg(not(feature = "commands"))]
        {
            debug!("Commands feature not enabled, skipping CommandsPluginAdapter");
            return Ok(None);
        }
    }

    /// Load plugins from a directory
    ///
    /// # Arguments
    ///
    /// * `dir_path` - Path to directory containing plugin shared libraries
    ///
    /// # Returns
    ///
    /// A list of plugin IDs that were loaded
    pub async fn load_plugins_from_directory<P: AsRef<Path>>(&self, dir_path: P) -> Result<Vec<String>> {
        let dir_path = dir_path.as_ref();
        info!("Loading plugins from directory: {:?}", dir_path);

        if !dir_path.exists() {
            warn!("Plugin directory does not exist: {:?}", dir_path);
            return Ok(Vec::new());
        }

        if !dir_path.is_dir() {
            return Err(anyhow!("Path is not a directory: {:?}", dir_path));
        }

        let mut plugin_ids = Vec::new();

        // Read the directory entries
        let entries = std::fs::read_dir(dir_path)
            .with_context(|| format!("Failed to read plugin directory: {:?}", dir_path))?;

        // Process each entry
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Skip non-library files
            if !Self::is_plugin_library(&path) {
                continue;
            }

            // Try to load the plugin
            match self.load_plugin_from_path(&path).await {
                Ok(plugin_id) => {
                    plugin_ids.push(plugin_id);
                }
                Err(e) => {
                    error!("Failed to load plugin from {:?}: {}", path, e);
                }
            }
        }

        info!("Loaded {} plugins from directory", plugin_ids.len());
        Ok(plugin_ids)
    }

    /// Load a plugin from a path
    pub async fn load_plugin_from_path<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let path = path.as_ref();
        info!("Loading plugin from path: {}", path.display());

        // Load the library
        // Safety: We're loading a dynamic library that is expected to conform to our plugin interface.
        // This is inherently unsafe as we're executing external code.
        let library = unsafe { Library::new(path) }.context("Failed to load library")?;
        info!("Library loaded successfully");

        // Get plugin metadata
        let metadata: extern "C" fn() -> *const PluginMetadata = unsafe {
            std::mem::transmute(
                library
                    .get::<*const ()>(b"get_plugin_metadata\0")
                    .context("Failed to get plugin metadata function")?,
            )
        };

        // Get plugin creation function
        let create_plugin: extern "C" fn() -> *mut dyn Plugin = unsafe {
            std::mem::transmute(
                library
                    .get::<*const ()>(b"create_plugin\0")
                    .context("Failed to get plugin creation function")?,
            )
        };

        // Get the plugin metadata
        let metadata_ptr = metadata();
        if metadata_ptr.is_null() {
            return Err(anyhow::anyhow!("Plugin metadata is null"));
        }

        let metadata = unsafe { &*metadata_ptr };
        info!(
            "Plugin metadata: {} ({}) by {}",
            metadata.name, metadata.version, metadata.author
        );

        // Create the plugin
        let plugin_ptr = create_plugin();
        if plugin_ptr.is_null() {
            return Err(anyhow::anyhow!("Plugin creation failed"));
        }

        // Convert the raw pointer to an Arc<dyn Plugin>
        let plugin: Arc<dyn Plugin> = unsafe { Arc::from_raw(plugin_ptr as *const dyn Plugin) };

        // Register the plugin with a wrapper
        let wrapped_plugin = Arc::new(DynamicPluginWrapper::new(plugin.clone()));
        let plugin_id = self.registry.register_plugin(wrapped_plugin).await?;

        // Store the loaded library
        let mut libraries = self.loaded_libraries.lock().map_err(|e| {
            anyhow!("Failed to acquire loaded_libraries lock: {}", e)
        })?;
        libraries.push(LoadedLibrary {
            library,
            path: path.to_path_buf(),
            plugin: plugin.clone(),
        });

        info!("Plugin registered with ID: {}", plugin_id);
        Ok(plugin_id)
    }

    /// Check if a path is a plugin library file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to check
    ///
    /// # Returns
    ///
    /// True if the path is a plugin library file, false otherwise
    fn is_plugin_library(path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        let extension = path.extension().and_then(OsStr::to_str);

        #[cfg(target_os = "windows")]
        let is_plugin_ext = extension == Some("dll");

        #[cfg(target_os = "linux")]
        let is_plugin_ext = extension == Some("so");

        #[cfg(target_os = "macos")]
        let is_plugin_ext = extension == Some("dylib");

        is_plugin_ext
    }

    /// Initialize all loaded plugins
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn initialize_all_plugins(&self) -> Result<()> {
        info!("Initializing all plugins");
        let plugins = self.registry.list_plugins().await;

        for plugin in plugins {
            let plugin_id = plugin.metadata().id.clone();
            debug!("Initializing plugin: {}", plugin_id);

            if let Err(e) = plugin.initialize().await {
                error!("Failed to initialize plugin {}: {}", plugin_id, e);
            } else {
                debug!("Plugin {} initialized successfully", plugin_id);
            }
        }

        info!("All plugins initialized");
        Ok(())
    }

    /// Shutdown all loaded plugins
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn shutdown_all_plugins(&self) -> Result<()> {
        info!("Shutting down all plugins");
        let plugins = self.registry.list_plugins().await;

        for plugin in plugins {
            let plugin_id = plugin.metadata().id.clone();
            debug!("Shutting down plugin: {}", plugin_id);

            if let Err(e) = plugin.shutdown().await {
                error!("Failed to shutdown plugin {}: {}", plugin_id, e);
            } else {
                debug!("Plugin {} shut down successfully", plugin_id);
            }
        }

        info!("All plugins shut down");
        Ok(())
    }
}

/// Create a new plugin loader
///
/// # Arguments
///
/// * `registry` - The plugin registry to use
///
/// # Returns
///
/// A new plugin loader instance
pub fn create_plugin_loader(registry: Arc<DefaultPluginRegistry>) -> Arc<PluginLoader> {
    Arc::new(PluginLoader::new(registry))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::registry::create_plugin_registry;
    use squirrel_interfaces::plugins::PluginMetadata;
    use std::io::Write;

    #[derive(Debug)]
    struct TestPlugin {
        metadata: PluginMetadata,
    }

    impl TestPlugin {
        fn new(id: &str) -> Self {
            let metadata = PluginMetadata::new(
                id,
                "1.0.0",
                "Test plugin for unit tests",
                "DataScienceBioLab",
            );

            Self { metadata }
        }
    }

    #[async_trait::async_trait]
    impl Plugin for TestPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        async fn initialize(&self) -> Result<()> {
            Ok(())
        }

        async fn shutdown(&self) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_plugin_loader_creation() {
        let registry = create_plugin_registry();
        let loader = PluginLoader::new(registry);
        assert!(loader.loaded_libraries.lock().unwrap().is_empty());
    }

    // More tests would be added to test dynamic loading, but that requires
    // creating actual dynamic libraries during the test which is complex.
    // Instead, focus on testing the registry integration.
} 