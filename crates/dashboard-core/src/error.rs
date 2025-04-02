//! Error types for the dashboard system.

use thiserror::Error;

/// Result type for dashboard operations
pub type Result<T> = std::result::Result<T, DashboardError>;

/// Dashboard errors
#[derive(Debug, Error)]
pub enum DashboardError {
    /// Error when collecting metrics
    #[error("Failed to collect metrics: {0}")]
    MetricCollection(String),

    /// Error when collecting system information
    #[error("Failed to collect system info: {0}")]
    SystemInfo(String),

    /// Error with storage operations
    #[error("Storage error: {0}")]
    Storage(String),

    /// Error with configuration
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Error with serialization/deserialization
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Error with network operations
    #[error("Network error: {0}")]
    Network(String),

    /// Error with update handling
    #[error("Update error: {0}")]
    Update(String),

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Generic error
    #[error("{0}")]
    Generic(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(String),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(String),
    
    /// External system error
    #[error("External system error: {0}")]
    External(String),
} 