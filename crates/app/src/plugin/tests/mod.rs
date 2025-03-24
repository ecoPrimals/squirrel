//! Plugin system tests
//!
//! This module contains tests for the plugin system, including unit tests
//! and integration tests for the various components.

// Integration tests for the complete plugin system
mod integration_test;

// Re-export existing test functions from the mod.rs file
pub use super::tests::*;

// Add any additional test utilities here 