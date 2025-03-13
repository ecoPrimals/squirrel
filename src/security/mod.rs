//! Security module for Squirrel
//!
//! This module provides security functionality including authentication,
//! encryption, and audit logging.

pub mod auth;
pub mod encryption;
pub mod audit;

// Re-export commonly used types
pub use auth::{Auth, AuthProvider, AuthToken, AuthError};
pub use encryption::{Encryption, EncryptionProvider, Key, EncryptionError};
pub use audit::{AuditLog, AuditEvent, AuditError};

/// Initialize the security system
pub async fn initialize() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize authentication
    auth::initialize().await?;
    
    // Initialize encryption
    encryption::initialize().await?;
    
    // Initialize audit logging
    audit::initialize().await?;
    
    Ok(())
}

/// Shutdown the security system
pub async fn shutdown() -> Result<(), Box<dyn std::error::Error>> {
    // Shutdown in reverse order
    audit::shutdown().await?;
    encryption::shutdown().await?;
    auth::shutdown().await?;
    
    Ok(())
} 