//! Plugin Security module
//!
//! This module provides security functionality for plugins.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSecurityConfig {
    /// Whether the plugin is allowed to access the file system
    pub file_system_access: bool,
    /// Whether the plugin is allowed to make network requests
    pub network_access: bool,
    /// Allowed network hosts
    pub allowed_hosts: Vec<String>,
    /// Plugin permissions
    pub permissions: HashMap<String, bool>,
}

impl Default for PluginSecurityConfig {
    fn default() -> Self {
        Self {
            file_system_access: false,
            network_access: false,
            allowed_hosts: Vec::new(),
            permissions: HashMap::new(),
        }
    }
}

/// Plugin security context
#[derive(Debug, Clone)]
pub struct PluginSecurityContext {
    /// Security configuration
    pub config: PluginSecurityConfig,
    /// Plugin ID
    pub plugin_id: String,
}

impl PluginSecurityContext {
    /// Create a new security context
    pub fn new(plugin_id: String, config: PluginSecurityConfig) -> Self {
        Self { plugin_id, config }
    }

    /// Check if a permission is allowed
    pub fn is_permission_allowed(&self, permission: &str) -> bool {
        self.config
            .permissions
            .get(permission)
            .copied()
            .unwrap_or(false)
    }

    /// Check if file system access is allowed
    pub fn is_file_system_access_allowed(&self) -> bool {
        self.config.file_system_access
    }

    /// Check if network access is allowed
    pub fn is_network_access_allowed(&self) -> bool {
        self.config.network_access
    }

    /// Check if a host is allowed
    pub fn is_host_allowed(&self, host: &str) -> bool {
        if !self.config.network_access {
            return false;
        }

        if self.config.allowed_hosts.is_empty() {
            return true; // No restrictions
        }

        self.config.allowed_hosts.contains(&host.to_string())
    }
}

/// Security manager for plugins
pub struct SecurityManager {
    /// Security validator
    validator: PluginSecurityValidator,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> Self {
        Self {
            validator: PluginSecurityValidator::new(),
        }
    }

    /// Add a plugin security context
    pub fn add_context(&mut self, plugin_id: String, context: PluginSecurityContext) {
        self.validator.add_context(plugin_id, context);
    }

    /// Validate a plugin operation
    pub fn validate_operation(&self, plugin_id: &str, operation: &str) -> bool {
        self.validator.validate_operation(plugin_id, operation)
    }

    /// Validate file system access
    pub fn validate_file_system_access(&self, plugin_id: &str) -> bool {
        self.validator.validate_file_system_access(plugin_id)
    }

    /// Validate network access
    pub fn validate_network_access(&self, plugin_id: &str, host: &str) -> bool {
        self.validator.validate_network_access(plugin_id, host)
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Security validator for plugins
pub struct PluginSecurityValidator {
    /// Security contexts by plugin ID
    contexts: HashMap<String, PluginSecurityContext>,
}

impl PluginSecurityValidator {
    /// Create a new security validator
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
        }
    }

    /// Add a plugin security context
    pub fn add_context(&mut self, plugin_id: String, context: PluginSecurityContext) {
        self.contexts.insert(plugin_id, context);
    }

    /// Validate a plugin operation
    pub fn validate_operation(&self, plugin_id: &str, operation: &str) -> bool {
        if let Some(context) = self.contexts.get(plugin_id) {
            context.is_permission_allowed(operation)
        } else {
            false
        }
    }

    /// Validate file system access
    pub fn validate_file_system_access(&self, plugin_id: &str) -> bool {
        if let Some(context) = self.contexts.get(plugin_id) {
            context.is_file_system_access_allowed()
        } else {
            false
        }
    }

    /// Validate network access
    pub fn validate_network_access(&self, plugin_id: &str, host: &str) -> bool {
        if let Some(context) = self.contexts.get(plugin_id) {
            context.is_host_allowed(host)
        } else {
            false
        }
    }
}

impl Default for PluginSecurityValidator {
    fn default() -> Self {
        Self::new()
    }
}
