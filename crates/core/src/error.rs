//! Error types for the Squirrel project
//!
//! This module defines the main error types and results used throughout the project.

use thiserror::Error;

/// Main error type for the Squirrel project
#[derive(Debug, Error)]
pub enum SquirrelError {
    /// Errors originating from the app module
    #[error("App error: {0}")]
    App(#[from] crate::app::error::CoreError),
    
    /// Other miscellaneous errors that don't fit into specific categories
    #[error("Other error: {0}")]
    Other(String),
}

/// A Result type alias for operations that may return a `SquirrelError`
pub type Result<T> = std::result::Result<T, SquirrelError>; 