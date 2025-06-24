//! Plugin Sandbox System
//!
//! This module provides comprehensive plugin sandboxing capabilities with support for
//! multiple platforms, resource monitoring, security contexts, and graceful degradation.
//!
//! ## Architecture
//!
//! The sandbox system is organized into focused modules:
//! - `errors`: Error types and conversions
//! - `traits`: Core trait definitions
//! - `basic`: Basic fallback sandbox implementation
//! - `cross_platform`: Cross-platform sandbox with native implementations
//! - `capabilities`: Platform capability detection
//! - `testing`: Comprehensive test suite
//!
//! ## Usage
//!
//! ```rust
//! use std::sync::Arc;
//! use crate::plugin::resource_monitor::ResourceMonitor;
//! use crate::plugin::sandbox::{CrossPlatformSandbox, PluginSandbox};
//!
//! // Create a cross-platform sandbox
//! let resource_monitor = Arc::new(ResourceMonitor::new());
//! let sandbox = CrossPlatformSandbox::new(resource_monitor)?;
//!
//! // Use the sandbox
//! let plugin_id = uuid::Uuid::new_v4();
//! sandbox.create_sandbox(plugin_id).await?;
//! ```

// Re-export platform modules that may already exist
pub mod windows;
pub mod linux;
pub mod macos;
pub mod seccomp;

// Core refactored modules
pub mod errors;
pub mod traits;
pub mod basic;
pub mod cross_platform;
pub mod capabilities;

// Testing module
// #[cfg(test)]
// pub mod testing;

// Re-exports for backward compatibility
pub use errors::SandboxError;
pub use traits::PluginSandbox;
pub use basic::BasicPluginSandbox;
pub use cross_platform::CrossPlatformSandbox;

// Additional convenience re-exports
pub use capabilities::cross_platform::{get_available_capabilities, has_capability};

// Import commonly used types
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::error::Result;
use crate::plugin::resource_monitor::ResourceMonitor;

/// Create a new cross-platform sandbox with automatic platform detection
pub fn create_sandbox(resource_monitor: Arc<ResourceMonitor>) -> Result<CrossPlatformSandbox> {
    CrossPlatformSandbox::new(resource_monitor)
}

/// Create a basic sandbox for testing or fallback scenarios
pub fn create_basic_sandbox(resource_monitor: Arc<ResourceMonitor>) -> BasicPluginSandbox {
    BasicPluginSandbox::new(resource_monitor)
}

/// Get the available sandbox capabilities for the current platform
pub fn get_platform_capabilities() -> HashSet<String> {
    capabilities::cross_platform::get_available_capabilities()
}

/// Check if a specific capability is available on the current platform
pub fn has_platform_capability(capability: &str) -> bool {
    capabilities::cross_platform::has_capability(capability)
}

/// Get platform-specific sandbox information
pub fn get_platform_info() -> HashMap<String, String> {
    let mut info = HashMap::new();
    
    let platform_name = ResourceMonitor::get_platform_name();
    info.insert("platform".to_string(), platform_name.to_string());
    
    // Add capability information
    let capabilities = get_platform_capabilities();
    info.insert("capabilities_count".to_string(), capabilities.len().to_string());
    info.insert("has_native_sandbox".to_string(), 
        capabilities.contains("native_sandbox").to_string());
    
    // Add platform-specific details
                #[cfg(target_os = "windows")]
                {
        info.insert("windows_job_objects".to_string(), 
            capabilities::windows::has_app_container().to_string());
        info.insert("windows_integrity_levels".to_string(), 
            capabilities::windows::has_integrity_levels().to_string());
    }
    
            #[cfg(target_os = "linux")]
            {
        info.insert("linux_cgroups_v2".to_string(), 
            capabilities::linux::has_cgroups_v2().to_string());
        info.insert("linux_seccomp".to_string(), 
            capabilities::linux::has_seccomp().to_string());
    }

#[cfg(target_os = "macos")]
    {
        info.insert("macos_app_sandbox".to_string(), 
            capabilities::macos::has_app_sandbox().to_string());
        info.insert("macos_sip".to_string(), 
            capabilities::macos::has_sip().to_string());
    }
    
    info
} 