//! Service Security Configuration
//!
//! Canonical security configuration for MCP service-level security
//! (service composition, workflows, etc.)
//!
//! This consolidates security settings across MCP's service-oriented subsystems.

use serde::{Deserialize, Serialize};
use super::EncryptionConfig;

/// Canonical service security configuration
///
/// Used for service composition, workflows, and other service-oriented
/// security requirements within MCP.
///
/// # Evolution Path
///
/// This config consolidates previously scattered SecurityConfig definitions
/// in service_composition and workflow modules, providing a single source
/// of truth for service-level security.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSecurityConfig {
    /// Authentication required for service access
    pub auth_required: bool,
    
    /// Authorization rules (capability names, role names, etc.)
    pub authorization: Vec<String>,
    
    /// Encryption settings for service communications
    pub encryption: EncryptionConfig,
    
    /// Access control configuration (optional for backward compatibility)
    ///
    /// Workflows use this; service composition doesn't yet.
    /// Evolutionary path: all services will eventually adopt access control.
    #[serde(default)]
    pub access_control: Option<AccessControlConfig>,
}

/// Access control configuration
///
/// Evolutionary design: Compatible with existing workflow usage while
/// providing path to more sophisticated access control modes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlConfig {
    /// Access control enabled
    pub enabled: bool,
    
    /// Access rules (patterns, policies, etc.)
    pub rules: Vec<String>,
    
    /// Role-based access control enabled
    /// (Backward compatibility with workflow types)
    #[serde(default)]
    pub rbac: bool,
}

impl Default for ServiceSecurityConfig {
    fn default() -> Self {
        Self {
            auth_required: false,
            authorization: vec![],
            encryption: EncryptionConfig::default(),
            access_control: None,
        }
    }
}

impl Default for AccessControlConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rules: vec![],
            rbac: false,
        }
    }
}

// Evolutionary helper: Create from workflow-style config with access control
impl ServiceSecurityConfig {
    /// Create with access control enabled
    pub fn with_access_control(
        auth_required: bool,
        authorization: Vec<String>,
        encryption: EncryptionConfig,
        access_control: AccessControlConfig,
    ) -> Self {
        Self {
            auth_required,
            authorization,
            encryption,
            access_control: Some(access_control),
        }
    }
    
    /// Create simple config without access control (for service composition)
    pub fn simple(
        auth_required: bool,
        authorization: Vec<String>,
        encryption: EncryptionConfig,
    ) -> Self {
        Self {
            auth_required,
            authorization,
            encryption,
            access_control: None,
        }
    }
}

