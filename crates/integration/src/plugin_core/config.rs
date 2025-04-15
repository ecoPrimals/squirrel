//! Configuration for Plugin-Core integration
//!
//! This module defines configuration structures for the Plugin-Core adapter.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Configuration options for the Plugin-Core adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCoreConfig {
    /// Whether to automatically initialize plugins after registration
    pub auto_initialize_plugins: bool,
    
    /// Whether to require plugins to be registered with the core
    pub require_core_registration: bool,
    
    /// The directory to scan for plugins
    pub plugin_directory: PathBuf,
    
    /// Whether to verify plugin signatures
    pub verify_signatures: bool,
}

impl Default for PluginCoreConfig {
    fn default() -> Self {
        Self {
            auto_initialize_plugins: true,
            require_core_registration: false,
            plugin_directory: PathBuf::from("./plugins"),
            verify_signatures: false,
        }
    }
} 