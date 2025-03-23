#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![doc(html_root_url = "https://docs.rs/squirrel-core")]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::module_name_repetitions)]
#![warn(clippy::todo)]
#![cfg_attr(test, allow(clippy::unwrap_used))]
#![cfg_attr(test, allow(clippy::expect_used))]

//! Core functionality for Squirrel
//!
//! This crate provides the fundamental shared components used throughout the Squirrel ecosystem.
//! It has been refactored to be a minimal crate that only contains:
//! 
//! - Shared error types and utilities
//! - Build information
//!
//! All other functionality has been moved to dedicated crates.

/// Error handling types and utilities
pub mod error;

/// Build information
pub mod build_info {
    /// The built info from the build script
    #[allow(clippy::multiple_unsafe_ops_per_block, clippy::wildcard_imports)]
    pub mod built_info {
        include!(concat!(env!("OUT_DIR"), "/built.rs"));
    }

    /// Get the version string with build information
    #[must_use]
    pub fn version() -> String {
        let version = built_info::PKG_VERSION;
        // Just return the package version for now
        version.to_string()
    }
    
    /// Get the build date
    /// 
    /// Returns a string representation of the current time in seconds since UNIX epoch
    /// 
    /// # Panics
    /// 
    /// This function will panic if the current system time is before the UNIX epoch, which
    /// should be extremely rare and would indicate a system clock issue.
    #[must_use]
    pub fn build_date() -> String {
        // Since BUILT_TIME_UTC may not be available, let's use a fallback
        // that will work in any environment
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Safer alternative to expect() that shouldn't ever fail on normal systems
        let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration,
            Err(_) => {
                // This should be impossible unless the system clock is set to before 1970
                // But just in case, return a reasonable default instead of panicking
                std::time::Duration::from_secs(0)
            }
        };
        
        format!("{}", now.as_secs())
    }
}

/// Core application struct
#[derive(Debug, Clone)]
pub struct Core {
    /// The version of the application
    version: String,
}

/// Status information for the Core
#[derive(Debug, Clone)]
pub struct Status {
    /// Current status message
    pub status: String,
    /// Uptime in seconds
    pub uptime: u64,
    /// Memory usage in MB
    pub memory_usage: u64,
    /// Number of active commands
    pub active_commands: u32,
    /// Number of connected clients
    pub connected_clients: u32,
}

impl Core {
    /// Create a new Core instance
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: build_info::version(),
        }
    }

    /// Get the version string
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }
    
    /// Get the current status
    ///
    /// # Errors
    ///
    /// Returns an error if status information cannot be retrieved
    pub fn get_status(&self) -> crate::error::Result<Status> {
        // In a real implementation, these would be actual values
        Ok(Status {
            status: "Running".to_string(),
            uptime: 3600, // Example: 1 hour
            memory_usage: 128, // Example: 128 MB
            active_commands: 5,
            connected_clients: 2,
        })
    }
}

impl Default for Core {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let version = build_info::version();
        assert!(!version.is_empty());
        assert!(version.contains(build_info::built_info::PKG_VERSION));
    }
}