//! Integration tests for the Plugin-Core adapter
//!
//! This module contains tests that verify the integration between
//! the Plugin system and Core components using the adapter pattern.

use std::sync::Arc;
use std::path::PathBuf;
use std::fs;
use uuid::Uuid;

use squirrel_plugins::{Plugin, PluginStatus};
use crate::plugin_core::{
    adapter::PluginCoreAdapter,
    config::PluginCoreConfig,
    testing::{
        create_random_test_plugin
    },
};
use crate::error::Result;

/// Creates a temporary plugins directory for testing
fn setup_test_dir() -> PathBuf {
    let test_dir = PathBuf::from("./target/test_plugins");
    let _ = fs::remove_dir_all(&test_dir); // Clean up previous test directory
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");
    test_dir
}

/// Tests the adapter's initialization with default configuration
#[tokio::test]
async fn test_adapter_initialization() {
    // Create a test directory
    let test_dir = setup_test_dir();
    
    // Create adapter with config pointing to our test directory
    let mut adapter = PluginCoreAdapter::with_config(PluginCoreConfig {
        plugin_directory: test_dir,
        auto_initialize_plugins: false, // Disable auto-init to avoid hanging
        ..PluginCoreConfig::default()
    });
    
    // Check initial state
    assert!(!adapter.is_initialized());
    
    // Initialize adapter with test-specific method
    adapter.initialize_for_tests().await.expect("Failed to initialize adapter");
    
    // Check initialized state
    assert!(adapter.is_initialized());
}

/// Tests the adapter with custom configuration
#[tokio::test]
async fn test_adapter_with_custom_config() {
    // Create a test directory
    let test_dir = setup_test_dir();
    
    // Create custom config
    let config = PluginCoreConfig {
        plugin_directory: test_dir.clone(),
        auto_initialize_plugins: false,
        require_core_registration: false,
        verify_signatures: false,
    };
    
    // Create adapter with custom config
    let adapter = PluginCoreAdapter::with_config(config.clone());
    
    // Check config is stored correctly
    assert_eq!(adapter.config.plugin_directory, test_dir);
    assert_eq!(adapter.config.auto_initialize_plugins, false);
}

/// Tests plugin registration through the adapter
#[tokio::test]
async fn test_plugin_registration() -> Result<()> {
    // Create a test directory
    let test_dir = setup_test_dir();
    
    // Create adapter with config pointing to our test directory
    let mut adapter = PluginCoreAdapter::with_config(PluginCoreConfig {
        plugin_directory: test_dir,
        auto_initialize_plugins: false, // Disable auto-init to avoid hanging
        ..PluginCoreConfig::default()
    });
    
    // Initialize adapter with test-specific method
    adapter.initialize_for_tests().await?;
    
    // Create a test plugin
    let plugin = create_random_test_plugin();
    let plugin_id = plugin.metadata().id;
    
    // Register the plugin
    adapter.register_plugin(plugin.clone() as Arc<dyn Plugin>).await?;
    
    // Check if plugin is registered - the status will be Registered since auto-init is disabled
    let plugin_status = adapter.get_plugin_status(plugin_id).await?;
    assert!(matches!(plugin_status, PluginStatus::Registered));
    
    Ok(())
}

/// Tests auto-initialization of plugins
#[tokio::test]
async fn test_auto_initialization() -> Result<()> {
    // Create a test directory
    let test_dir = setup_test_dir();
    
    // Create adapter with auto-initialization enabled
    let mut adapter = PluginCoreAdapter::with_config(PluginCoreConfig {
        plugin_directory: test_dir,
        auto_initialize_plugins: true,
        ..PluginCoreConfig::default()
    });
    
    // Initialize adapter with test-specific method
    adapter.initialize_for_tests().await?;
    
    // Create a test plugin
    let plugin = create_random_test_plugin();
    let plugin_id = plugin.metadata().id;
    
    // Register the plugin (should auto-initialize because auto_initialize_plugins is true)
    adapter.register_plugin(plugin.clone() as Arc<dyn Plugin>).await?;
    
    // Check if plugin is initialized
    let plugin_status = adapter.get_plugin_status(plugin_id).await?;
    assert!(matches!(plugin_status, PluginStatus::Initialized));
    
    Ok(())
}

/// Tests shutting down all plugins
#[tokio::test]
async fn test_shutdown_all_plugins() -> Result<()> {
    // Create a test directory
    let test_dir = setup_test_dir();
    
    // Create adapter with config pointing to our test directory
    let mut adapter = PluginCoreAdapter::with_config(PluginCoreConfig {
        plugin_directory: test_dir,
        auto_initialize_plugins: true, // Enable auto-init for this test
        ..PluginCoreConfig::default()
    });
    
    // Initialize adapter with test-specific method
    adapter.initialize_for_tests().await?;
    
    // Create and register multiple plugins
    let plugins = vec![
        create_random_test_plugin(),
        create_random_test_plugin(),
        create_random_test_plugin(),
    ];
    
    let plugin_ids: Vec<Uuid> = plugins.iter()
        .map(|p| p.metadata.id)
        .collect();
    
    for plugin in plugins {
        adapter.register_plugin(plugin.clone() as Arc<dyn Plugin>).await?;
    }
    
    // Each plugin should be initialized due to auto_initialize_plugins = true
    
    // Verify all plugins are initialized
    for id in &plugin_ids {
        let status = adapter.get_plugin_status(*id).await?;
        assert!(matches!(status, PluginStatus::Initialized));
    }
    
    // Shutdown all plugins
    adapter.shutdown_all_plugins().await?;
    
    // Verify plugins are now in registered state after shutdown
    for id in &plugin_ids {
        let status = adapter.get_plugin_status(*id).await?;
        assert!(matches!(status, PluginStatus::Registered));
    }
    
    Ok(())
}

/// Tests error handling for uninitialized adapter
#[tokio::test]
async fn test_uninitialized_error() {
    // Create adapter without initializing
    let adapter = PluginCoreAdapter::new();
    
    // Attempt operations that require initialization
    let result = adapter.get_all_plugins().await;
    assert!(result.is_err());
    
    let result = adapter.plugin_manager();
    assert!(result.is_err());
    
    let result = adapter.core();
    assert!(result.is_err());
}

/// Tests the adapter's accessor methods
#[tokio::test]
async fn test_accessor_methods() -> Result<()> {
    // Create a test directory
    let test_dir = setup_test_dir();
    
    // Create adapter with config pointing to our test directory
    let mut adapter = PluginCoreAdapter::with_config(PluginCoreConfig {
        plugin_directory: test_dir,
        auto_initialize_plugins: false, // Disable auto-init to avoid hanging
        ..PluginCoreConfig::default()
    });
    
    // Initialize adapter with test-specific method
    adapter.initialize_for_tests().await?;
    
    // Test accessors
    let plugin_manager = adapter.plugin_manager()?;
    assert!(Arc::strong_count(&plugin_manager) > 0);
    
    let core = adapter.core()?;
    assert!(Arc::strong_count(&core) > 0);
    
    Ok(())
}

/// Tests handling plugin lifecycle with different states
#[tokio::test]
async fn test_plugin_lifecycle() -> Result<()> {
    // Create a test directory
    let test_dir = setup_test_dir();
    
    // Create adapter with config pointing to our test directory
    let mut adapter = PluginCoreAdapter::with_config(PluginCoreConfig {
        plugin_directory: test_dir,
        auto_initialize_plugins: true, // Enable auto-init for this test
        ..PluginCoreConfig::default()
    });
    
    // Initialize adapter with test-specific method
    adapter.initialize_for_tests().await?;
    
    // Create a normal plugin
    let normal_plugin = create_random_test_plugin();
    let normal_id = normal_plugin.metadata().id;
    
    // Register the normal plugin - should initialize due to auto_initialize_plugins = true
    adapter.register_plugin(normal_plugin.clone() as Arc<dyn Plugin>).await?;
    
    // Verify the normal plugin is initialized
    let status = adapter.get_plugin_status(normal_id).await?;
    assert!(matches!(status, PluginStatus::Initialized), "Expected normal plugin to be initialized");
    
    // Shutdown all plugins
    adapter.shutdown_all_plugins().await?;
    
    // Verify normal plugin status changed
    let status = adapter.get_plugin_status(normal_id).await?;
    assert!(matches!(status, PluginStatus::Registered), "Expected plugin to be back in Registered state after shutdown");
    
    Ok(())
} 