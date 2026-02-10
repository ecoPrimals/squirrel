// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Plugin system for the Squirrel AI Coordination System
//!
//! This crate provides plugin functionality and example plugin implementations.

#![deny(unsafe_code)]
/// Plugin system version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Plugin system utilities
pub mod utils {
    /// Example plugin utility function
    pub fn example_function() {
        println!("Plugin system loaded");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_utils() {
        utils::example_function(); // Should not panic
    }
}
