//! Core application functionality
//! 
//! This module provides the main Core struct that represents the application's core functionality.

use crate::VERSION;

/// Core application struct
#[derive(Debug)]
pub struct Core {
    /// Application version
    version: String,
}

impl Core {
    /// Creates a new Core instance
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: VERSION.to_string(),
        }
    }

    /// Returns the application version
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }
}

impl Default for Core {
    fn default() -> Self {
        Self::new()
    }
} 