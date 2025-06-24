//! macOS-specific plugin sandbox implementation
//!
//! This module provides a macOS-specific implementation of the PluginSandbox trait
//! using App Sandbox and resource limits, organized into focused modules.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{debug, error, info, trace, warn};
use std::any::Any;

use crate::error::{Result, SquirrelError, CoreError};
use crate::plugin::security::{SecurityContext, PermissionLevel, ResourceLimits};
use crate::plugin::resource_monitor::{ResourceMonitor, ResourceUsage};
use crate::plugin::sandbox::{PluginSandbox, SandboxError};

// Re-export all modules
pub mod sandbox_profiles;
pub mod security_context;
pub mod resource_limits;
pub mod tcc_integration;
pub mod sip_integration;
pub mod process_management;
pub mod platform_optimization;
pub mod compatibility;

// Re-export key types and functions
pub use sandbox_profiles::*;
pub use security_context::*;
pub use resource_limits::*;
pub use tcc_integration::*;
pub use sip_integration::*;
pub use process_management::*;
pub use platform_optimization::*;
pub use compatibility::*;

/// macOS-specific plugin sandbox implementation
#[derive(Debug)]
pub struct MacOsSandbox {
    /// Process IDs for plugins
    pub(crate) process_ids: Arc<RwLock<HashMap<Uuid, u32>>>,
    /// Security contexts for plugins
    pub(crate) security_contexts: Arc<RwLock<HashMap<Uuid, SecurityContext>>>,
    /// Resource monitor
    pub(crate) resource_monitor: Arc<ResourceMonitor>,
    /// Profile paths for sandboxed applications
    pub(crate) sandbox_profiles: Arc<RwLock<HashMap<Uuid, PathBuf>>>,
}

impl MacOsSandbox {
    /// Create a new macOS sandbox
    pub fn new(resource_monitor: Arc<ResourceMonitor>) -> Result<Self> {
        Ok(Self {
            process_ids: Arc::new(RwLock::new(HashMap::new())),
            security_contexts: Arc::new(RwLock::new(HashMap::new())),
            resource_monitor,
            sandbox_profiles: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Set a security context for a plugin
    pub async fn set_security_context(&self, plugin_id: Uuid, context: SecurityContext) -> Result<()> {
        // Update the stored security context
        let mut contexts = self.security_contexts.write().await;
        contexts.insert(plugin_id, context);
        
        // If the plugin is already sandboxed, we may need to restart it
        let process_ids = self.process_ids.read().await;
        if process_ids.contains_key(&plugin_id) {
            // In macOS, we can't modify sandbox settings for running processes
            // We would need to restart the plugin, which is handled at a higher level
            warn!("Security context updated for plugin {}, but changes will only take effect after restart", plugin_id);
        }
        
        Ok(())
    }
    
    /// Get a security context for a plugin
    pub async fn get_security_context(&self, plugin_id: Uuid) -> Result<SecurityContext> {
        let contexts = self.security_contexts.read().await;
        contexts.get(&plugin_id)
            .cloned()
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id).into())
    }
    
    /// Get the sandbox profile path for a plugin
    pub fn get_sandbox_profile_path(&self, plugin_id: Uuid) -> PathBuf {
        std::env::temp_dir()
            .join("squirrel_sandbox")
            .join(format!("{}.sb", plugin_id))
    }

    /// Check if a path is in a secure namespace
    pub fn is_path_in_secure_namespace(&self, path: &Path) -> bool {
        // Check against known secure locations on macOS
        let secure_paths = [
            "/System/",
            "/usr/lib/",
            "/usr/libexec/",
            "/Library/Application Support/",
        ];
        
        if let Some(path_str) = path.to_str() {
            secure_paths.iter().any(|&secure| path_str.starts_with(secure))
        } else {
            false
        }
    }
}

/// Shared helper functions for macOS sandbox operations

/// Get the current macOS version
pub fn get_macos_version() -> Result<String> {
    use std::process::Command;
    
    let output = Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .map_err(|e| SandboxError::Creation(format!("Failed to get macOS version: {}", e)))?;
    
    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        Ok(version.trim().to_string())
    } else {
        Err(SandboxError::Creation("Failed to determine macOS version".to_string()).into())
    }
}

/// Check if System Integrity Protection is enabled
pub fn check_sip_enabled() -> Result<bool> {
    use std::process::Command;
    
    let output = Command::new("csrutil")
        .arg("status")
        .output()
        .map_err(|e| SandboxError::Creation(format!("Failed to check SIP status: {}", e)))?;
    
    if output.status.success() {
        let status = String::from_utf8_lossy(&output.stdout);
        Ok(status.contains("enabled"))
    } else {
        // If csrutil fails, assume SIP is enabled for security
        Ok(true)
    }
}

/// Check if App Sandbox is available
pub fn check_app_sandbox_available() -> bool {
    // Check for sandbox-exec binary
    std::process::Command::new("which")
        .arg("sandbox-exec")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Check if TCC integration is available
pub fn check_tcc_available() -> bool {
    // Check for tccutil binary and TCC database
    let tccutil_exists = std::process::Command::new("which")
        .arg("tccutil")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);
        
    let tcc_db_exists = std::path::Path::new("/Library/Application Support/com.apple.TCC/TCC.db").exists();
    
    tccutil_exists && tcc_db_exists
}

/// Create a standard temp directory for sandbox operations
pub fn create_sandbox_temp_dir() -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir().join("squirrel_sandbox");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| SandboxError::Creation(format!("Failed to create sandbox temp directory: {}", e)))?;
    Ok(temp_dir)
}

/// Clean up sandbox temporary files for a plugin
pub fn cleanup_sandbox_files(plugin_id: Uuid) -> Result<()> {
    let temp_dir = std::env::temp_dir().join("squirrel_sandbox");
    let profile_path = temp_dir.join(format!("{}.sb", plugin_id));
    
    if profile_path.exists() {
        std::fs::remove_file(&profile_path)
            .map_err(|e| SandboxError::Cleanup(format!("Failed to remove sandbox profile: {}", e)))?;
    }
    
    Ok(())
}

/// Common error handling for macOS-specific operations
pub fn handle_macos_error(error: std::io::Error, context: &str) -> SandboxError {
    match error.kind() {
        std::io::ErrorKind::NotFound => {
            SandboxError::Creation(format!("{}: Required macOS component not found", context))
        },
        std::io::ErrorKind::PermissionDenied => {
            SandboxError::Creation(format!("{}: Permission denied - may require admin privileges", context))
        },
        _ => {
            SandboxError::Creation(format!("{}: {}", context, error))
        }
    }
} 