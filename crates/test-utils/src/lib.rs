//! Test utilities for Squirrel
//!
//! This crate provides test utilities, mocks, and fixtures for testing Squirrel components.

use thiserror::Error;
use std::path::PathBuf;
use tempfile::TempDir;
use std::fs;
use std::path::Path;

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

impl From<TestError> for squirrel_core::error::SquirrelError {
    fn from(err: TestError) -> Self {
        squirrel_core::error::SquirrelError::Other(err.to_string())
    }
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
pub fn create_test_env() -> Result<TempDir> {
    let temp_dir = tempfile::tempdir()
        .map_err(squirrel_core::error::SquirrelError::IO)?;
    Ok(temp_dir)
}

/// Creates a test file in the given directory
pub fn create_test_file(dir: &TempDir, name: &str, content: &str) -> Result<PathBuf> {
    let file_path = dir.path().join(name);
    std::fs::write(&file_path, content)
        .map_err(squirrel_core::error::SquirrelError::IO)?;
    Ok(file_path)
}

/// Copies a test file from one directory to another
pub fn copy_test_file(src_path: &Path, dest_path: &Path) -> Result<()> {
    fs::create_dir_all(dest_path.parent().ok_or_else(|| TestError::InvalidData("Invalid destination path".to_string()))?)
        .map_err(squirrel_core::error::SquirrelError::IO)?;

    fs::copy(src_path, dest_path)
        .map_err(squirrel_core::error::SquirrelError::IO)?;

    Ok(())
} 