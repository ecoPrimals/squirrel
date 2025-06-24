//! Trait Definitions for Plugin Sandboxing
//!
//! This module contains the core trait definitions for plugin sandbox implementations.

use std::any::Any;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::Result;
use crate::plugin::security::SecurityContext;
use crate::plugin::resource_monitor::{ResourceMonitor, ResourceUsage};

/// Plugin sandbox trait for isolating plugins
#[async_trait::async_trait]
pub trait PluginSandbox: Send + Sync + std::fmt::Debug {
    /// Create a sandbox for a plugin
    async fn create_sandbox(&self, plugin_id: Uuid) -> Result<()>;
    
    /// Destroy a sandbox for a plugin
    async fn destroy_sandbox(&self, plugin_id: Uuid) -> Result<()>;
    
    /// Check if an operation is allowed for a plugin
    async fn check_permission(&self, plugin_id: Uuid, operation: &str) -> Result<()>;
    
    /// Track resource usage for a plugin
    async fn track_resources(&self, plugin_id: Uuid) -> Result<ResourceUsage>;
    
    /// Check if a plugin has access to a path
    async fn check_path_access(&self, plugin_id: Uuid, path: &Path, write: bool) -> Result<()>;
    
    /// Check if a plugin has a capability
    async fn check_capability(&self, plugin_id: Uuid, capability: &str) -> Result<bool>;
    
    /// Apply a feature with platform-specific implementation
    async fn apply_feature(&self, plugin_id: Uuid, feature: &str) -> Result<()>;
    
    /// Set a security context for a plugin
    async fn set_security_context(&self, plugin_id: Uuid, context: SecurityContext) -> Result<()>;
    
    /// Get the sandbox as Any for downcasting
    fn as_any(&self) -> &dyn Any;
    
    /// Get the resource monitor for this sandbox
    fn get_resource_monitor(&self) -> Option<Arc<ResourceMonitor>> {
        None
    }
    
    /// Check if the sandbox implementation is available on the current platform
    fn is_sandbox_available(&self) -> bool {
        true
    }
} 