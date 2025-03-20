//! Test utilities for Squirrel
//!
//! This crate provides test utilities, mocks, and fixtures for testing Squirrel components.

use std::sync::Arc;
use thiserror::Error;
use std::path::PathBuf;
use std::error::Error;
use tokio::sync::RwLock;
use tempfile::TempDir;

/// Integration test utilities
pub mod integration_tests;
pub use integration_tests::IntegrationTestContext;

/// Test errors
#[derive(Debug, Error)]
pub enum TestError {
    /// Invalid test data
    #[error("Invalid test data: {0}")]
    InvalidData(String),
    
    /// Error during test initialization
    #[error("Test initialization error: {0}")]
    InitError(String),
    
    /// Error during test execution
    #[error("Test execution error: {0}")]
    ExecError(String),
}

/// Mock implementation for context adapter
pub mod mock_context;

/// Mock implementation for protocol adapter
pub mod mock_protocol;

/// Mock implementation for security
pub mod mock_security;

/// Test data generator
pub mod test_data;

/// Re-export common types from the core crate
pub use squirrel_core::error::Result;

/// Creates a temporary test environment
pub fn create_test_env() -> Result<TempDir>;

/// Creates a test file in the given directory
pub fn create_test_file(dir: &TempDir, name: &str, content: &str) -> Result<PathBuf>; 