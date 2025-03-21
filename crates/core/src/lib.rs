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
}

/// Core application struct
#[derive(Debug, Clone)]
pub struct Core {
    /// The version of the application
    version: String,
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