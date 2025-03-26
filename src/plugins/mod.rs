// Plugins Module
//
// This module provides the plugin system for the Squirrel application.
// It includes interfaces, management, builders, discovery, and security for plugins.

pub mod interfaces;
pub mod management;
pub mod builders;
pub mod discovery;
pub mod security;
pub mod examples;
pub mod resource;
pub mod state;
pub mod errors;
pub mod lifecycle;
pub mod dynamic;
pub mod marketplace;
#[cfg(test)]
mod tests;

//! Plugin implementations
//! 
//! This module contains various plugin implementations.

pub mod context_impl;

use std::sync::Arc;

// Re-export the Plugin trait and key components
pub use interfaces::{Plugin, CommandsPlugin, ToolPlugin};
pub use management::{PluginRegistry, PluginManager};
pub use discovery::PluginDiscovery;
pub use security::PluginSecurityManager;
pub use resource::{ResourceMonitor, ResourceMonitorImpl, ResourceLimits, ResourceUsage, ResourceType, ViolationAction};
pub use state::{StateManager, DefaultStateManager, PluginState, StateStorage, FileStateStorage, MemoryStateStorage, MigrationStrategy};
pub use errors::{PluginError, Result};
pub use lifecycle::{PluginLifecycle, PluginLifecycleManager, LifecycleState, LifecycleEvent};
pub use dynamic::{
    DynamicLibraryLoader, 
    VersionCompatibilityChecker,
    create_library_loader,
    PluginMetadata,
    PluginDependency
};
pub use marketplace::{
    RepositoryManager,
    RepositoryProvider,
    HttpRepositoryProvider,
    RepositoryInfo,
    PluginPackageInfo
};

// Plugin factory functions
pub fn create_plugin_manager() -> Arc<PluginManager> {
    use management::PluginRegistryImpl;
    
    let registry = Arc::new(PluginRegistryImpl::new());
    let security_manager = Arc::new(security::PluginSecurityValidator::new());
    
    Arc::new(PluginManager::new(registry, security_manager))
}

pub fn create_plugin_discovery(manager: Arc<PluginManager>) -> Arc<dyn PluginDiscovery> {
    use discovery::FileSystemDiscovery;
    
    let discovery = Arc::new(FileSystemDiscovery::new(manager.clone()));
    
    // Add example plugin loader
    let example_loader = examples::create_example_plugin_loader();
    discovery.add_loader(example_loader).expect("Failed to add example loader");
    
    discovery
}

/// Creates a resource monitor with default settings
pub fn create_resource_monitor() -> Arc<dyn ResourceMonitor> {
    Arc::new(ResourceMonitorImpl::new())
}

/// Creates a state manager with file storage
pub fn create_state_manager(data_dir: impl Into<std::path::PathBuf>) -> Arc<dyn StateManager> {
    let storage = Arc::new(FileStateStorage::new(data_dir));
    Arc::new(DefaultStateManager::new(storage))
}

/// Creates a repository manager for plugin marketplace
pub fn create_repository_manager(
    app_version: &str,
    download_dir: impl Into<std::path::PathBuf>,
) -> Result<Arc<RepositoryManager>> {
    let security_manager = Arc::new(security::PluginSecurityValidator::new());
    let manager = RepositoryManager::new(
        app_version,
        download_dir.into(),
        security_manager,
    )?;
    
    Ok(Arc::new(manager))
}

/// Plugin status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginStatus {
    /// Plugin is available but not loaded
    Available,
    /// Plugin is registered but not initialized
    Registered,
    /// Plugin is initialized but not started
    Initialized,
    /// Plugin is started and running
    Running,
    /// Plugin is stopped
    Stopped,
    /// Plugin has an error
    Error(String),
} 