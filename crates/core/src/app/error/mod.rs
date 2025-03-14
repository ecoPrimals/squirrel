//! Error types for the core module
//!
//! This module defines the error types used in the core functionality.

use thiserror::Error;

/// Errors that can occur in core operations
#[derive(Debug, Error)]
pub enum CoreError {
    /// An IO error occurred
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// A database error occurred
    #[error("Database error: {0}")]
    Database(String),
    
    /// A configuration error occurred
    #[error("Configuration error: {0}")]
    Config(String),
}

/// A Result type alias for core error handling
pub type Result<T> = std::result::Result<T, CoreError>; 