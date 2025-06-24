//! Error Types for Plugin Sandboxing
//!
//! This module defines error types that can occur during sandbox operations
//! and their conversions to the main application error types.

use thiserror::Error;
use uuid::Uuid;

use crate::error::SquirrelError;

/// Errors that can occur during sandbox operations
#[derive(Debug, Error)]
pub enum SandboxError {
    /// Plugin not found in sandbox
    #[error("Plugin not found in sandbox: {0}")]
    PluginNotFound(Uuid),
    
    /// Error creating sandbox
    #[error("Error creating sandbox: {0}")]
    Creation(String),
    
    /// Error destroying sandbox
    #[error("Error destroying sandbox: {0}")]
    Destruction(String),
    
    /// Permission error
    #[error("Permission error: {0}")]
    Permission(String),
    
    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),
    
    /// Path access denied
    #[error("Path access denied: {0}")]
    PathAccess(String),
    
    /// Capability not allowed
    #[error("Capability not allowed: {0}")]
    Capability(String),
    
    /// Platform-specific error
    #[error("Platform error: {0}")]
    Platform(String),
    
    /// Feature not supported on this platform
    #[error("Feature not supported: {0}")]
    Unsupported(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
    
    // Additional macOS-specific errors
    /// Resource limit exceeded (more specific)
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    /// Resource monitoring error
    #[error("Resource monitoring error: {0}")]
    ResourceMonitoring(String),
    
    /// Process termination error
    #[error("Process termination error: {0}")]
    ProcessTermination(String),
    
    /// TCC integration error
    #[error("TCC integration error: {0}")]
    TccIntegration(String),
    
    /// Process launch error
    #[error("Process launch error: {0}")]
    ProcessLaunch(String),
    
    /// Process sandboxing error
    #[error("Process sandboxing error: {0}")]
    ProcessSandboxing(String),
    
    /// Permission denied error
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Path access denied error
    #[error("Path access denied: {0}")]
    PathAccessDenied(String),
    
    /// Capability denied error
    #[error("Capability denied: {0}")]
    CapabilityDenied(String),
    
    /// Unsupported feature error
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
    
    /// Cleanup error
    #[error("Cleanup error: {0}")]
    Cleanup(String),
}

impl From<SandboxError> for SquirrelError {
    fn from(err: SandboxError) -> Self {
        match err {
            SandboxError::PluginNotFound(id) => Self::generic(format!("Plugin not found in sandbox: {id}")),
            SandboxError::Creation(msg) => Self::generic(format!("Error creating sandbox: {msg}")),
            SandboxError::Destruction(msg) => Self::generic(format!("Error destroying sandbox: {msg}")),
            SandboxError::Permission(msg) => Self::security(format!("Permission error: {msg}")),
            SandboxError::ResourceLimit(msg) => Self::security(format!("Resource limit exceeded: {msg}")),
            SandboxError::PathAccess(msg) => Self::security(format!("Path access denied: {msg}")),
            SandboxError::Capability(msg) => Self::security(format!("Capability not allowed: {msg}")),
            SandboxError::Platform(msg) => Self::generic(format!("Platform error: {msg}")),
            SandboxError::Unsupported(msg) => Self::generic(format!("Feature not supported: {msg}")),
            SandboxError::Internal(msg) => Self::generic(format!("Internal error: {msg}")),
            SandboxError::ResourceLimitExceeded(msg) => Self::security(format!("Resource limit exceeded: {msg}")),
            SandboxError::ResourceMonitoring(msg) => Self::security(format!("Resource monitoring error: {msg}")),
            SandboxError::ProcessTermination(msg) => Self::security(format!("Process termination error: {msg}")),
            SandboxError::TccIntegration(msg) => Self::security(format!("TCC integration error: {msg}")),
            SandboxError::ProcessLaunch(msg) => Self::security(format!("Process launch error: {msg}")),
            SandboxError::ProcessSandboxing(msg) => Self::security(format!("Process sandboxing error: {msg}")),
            SandboxError::PermissionDenied(msg) => Self::security(format!("Permission denied: {msg}")),
            SandboxError::PathAccessDenied(msg) => Self::security(format!("Path access denied: {msg}")),
            SandboxError::CapabilityDenied(msg) => Self::security(format!("Capability denied: {msg}")),
            SandboxError::UnsupportedFeature(msg) => Self::security(format!("Unsupported feature: {msg}")),
            SandboxError::Cleanup(msg) => Self::generic(format!("Cleanup error: {msg}")),
        }
    }
} 