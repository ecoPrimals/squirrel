//! Data module for Squirrel
//!
//! This module provides data management functionality including storage,
//! versioning, and migration capabilities.

pub mod storage;
pub mod versioning;
pub mod migration;

// Re-export commonly used types
pub use storage::{Store, Cache, StorageError, StorageConfig};
pub use versioning::{VersionManager, Version, VersionError, VersionConfig};
pub use migration::{Migration, MigrationManager, MigrationConfig};

/// Initialize the data management system
pub async fn initialize() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize storage
    storage::initialize().await?;
    
    // Initialize versioning
    versioning::initialize().await?;
    
    // Initialize migration
    migration::initialize().await?;
    
    Ok(())
}

/// Shutdown the data management system
pub async fn shutdown() -> Result<(), Box<dyn std::error::Error>> {
    // Shutdown in reverse order
    migration::shutdown().await?;
    versioning::shutdown().await?;
    storage::shutdown().await?;
    
    Ok(())
}

/// Get the current data management configuration
pub fn get_config() -> DataConfig {
    DataConfig {
        storage: storage::get_config(),
        versioning: versioning::get_config(),
        migration: migration::get_config(),
    }
}

/// Configuration for the data management system
#[derive(Debug, Clone)]
pub struct DataConfig {
    pub storage: storage::StorageConfig,
    pub versioning: versioning::VersionConfig,
    pub migration: migration::MigrationConfig,
}

/// Error types for data management operations
#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("Storage error: {0}")]
    Storage(#[from] storage::StorageError),
    
    #[error("Version error: {0}")]
    Version(#[from] versioning::VersionError),
    
    #[error("Migration error: {0}")]
    Migration(#[from] migration::MigrationError),
} 