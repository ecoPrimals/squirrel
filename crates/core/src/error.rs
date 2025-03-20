//! Error types for the Squirrel core library.

use std::fmt;
use std::error::Error;
use crate::context_adapter::ContextAdapterError;

/// Result type alias for `SquirrelError`
pub type Result<T> = std::result::Result<T, SquirrelError>;

/// Persistence errors
#[derive(Debug)]
pub enum PersistenceError {
    /// IO error
    IO(String),
    /// Configuration error
    Config(String),
    /// Storage error
    Storage(String),
    /// Format error
    Format(String),
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(msg) => write!(f, "Persistence IO error: {msg}"),
            Self::Config(msg) => write!(f, "Persistence configuration error: {msg}"),
            Self::Storage(msg) => write!(f, "Persistence storage error: {msg}"),
            Self::Format(msg) => write!(f, "Persistence format error: {msg}"),
        }
    }
}

impl Error for PersistenceError {}

/// Create an IO persistence error
#[must_use]
pub fn io_error(msg: &str) -> PersistenceError {
    PersistenceError::IO(msg.to_string())
}

/// Create a configuration persistence error
#[must_use]
pub fn config_error(msg: &str) -> PersistenceError {
    PersistenceError::Config(msg.to_string())
}

/// Create a storage persistence error
#[must_use]
pub fn storage_error(msg: &str) -> PersistenceError {
    PersistenceError::Storage(msg.to_string())
}

/// Create a format persistence error
#[must_use]
pub fn format_error(msg: &str) -> PersistenceError {
    PersistenceError::Format(msg.to_string())
}

/// Main error type for the Squirrel core library.
#[derive(Debug)]
pub enum SquirrelError {
    /// App initialization errors
    AppInitialization(AppInitializationError),
    /// App operation errors
    AppOperation(AppOperationError),
    /// Generic error with message
    Generic(String),
    /// IO errors
    IO(std::io::Error),
    /// Security-related errors
    Security(String),
    /// MCP module errors
    MCP(String),
    /// Other errors
    Other(String),
    /// Health monitoring errors
    Health(String),
    /// Metric collection errors
    Metric(String),
    /// Dashboard errors
    Dashboard(String),
    /// Serialization errors
    Serialization(String),
    /// Network monitoring errors
    Network(String),
    /// Alert errors
    Alert(String),
    /// Session-related errors
    Session(String),
    /// Persistence errors
    Persistence(PersistenceError),
    /// Protocol version errors
    ProtocolVersion(String),
    /// Context-related errors
    Context(String),
}

impl Error for SquirrelError {}

impl fmt::Display for SquirrelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SquirrelError::AppInitialization(e) => write!(f, "App initialization error: {e}"),
            SquirrelError::AppOperation(e) => write!(f, "App operation error: {e}"),
            SquirrelError::Generic(msg) => write!(f, "Error: {msg}"),
            SquirrelError::IO(e) => write!(f, "IO error: {e}"),
            SquirrelError::Security(msg) => write!(f, "Security error: {msg}"),
            SquirrelError::MCP(msg) => write!(f, "MCP error: {msg}"),
            SquirrelError::Other(msg) => write!(f, "Other error: {msg}"),
            SquirrelError::Health(msg) => write!(f, "Health error: {msg}"),
            SquirrelError::Metric(msg) => write!(f, "Metric error: {msg}"),
            SquirrelError::Dashboard(msg) => write!(f, "Dashboard error: {msg}"),
            SquirrelError::Serialization(msg) => write!(f, "Serialization error: {msg}"),
            SquirrelError::Network(msg) => write!(f, "Network error: {msg}"),
            SquirrelError::Alert(msg) => write!(f, "Alert error: {msg}"),
            SquirrelError::Session(msg) => write!(f, "Session error: {msg}"),
            SquirrelError::Persistence(e) => write!(f, "Persistence error: {e}"),
            SquirrelError::ProtocolVersion(msg) => write!(f, "Protocol version error: {msg}"),
            SquirrelError::Context(msg) => write!(f, "Context error: {msg}"),
        }
    }
}

impl From<std::io::Error> for SquirrelError {
    fn from(err: std::io::Error) -> Self {
        SquirrelError::IO(err)
    }
}

impl From<AppInitializationError> for SquirrelError {
    fn from(err: AppInitializationError) -> Self {
        SquirrelError::AppInitialization(err)
    }
}

impl From<AppOperationError> for SquirrelError {
    fn from(err: AppOperationError) -> Self {
        SquirrelError::AppOperation(err)
    }
}

impl From<String> for SquirrelError {
    fn from(err: String) -> Self {
        SquirrelError::Generic(err)
    }
}

impl From<&str> for SquirrelError {
    fn from(err: &str) -> Self {
        SquirrelError::Generic(err.to_string())
    }
}

impl From<serde_json::Error> for SquirrelError {
    fn from(err: serde_json::Error) -> Self {
        SquirrelError::Serialization(err.to_string())
    }
}

impl From<PersistenceError> for SquirrelError {
    fn from(err: PersistenceError) -> Self {
        SquirrelError::Persistence(err)
    }
}

impl From<crate::mcp::session::manager::persistence::PersistenceError> for SquirrelError {
    fn from(err: crate::mcp::session::manager::persistence::PersistenceError) -> Self {
        SquirrelError::Session(err.to_string())
    }
}

/// Errors that can occur during application initialization.
#[derive(Debug)]
pub enum AppInitializationError {
    /// The application has already been initialized
    AlreadyInitialized,
    /// Invalid configuration
    InvalidConfiguration(String),
    /// Failed to load resources
    ResourceLoadFailure(String),
}

impl Error for AppInitializationError {}

impl fmt::Display for AppInitializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppInitializationError::AlreadyInitialized => {
                write!(f, "Application already initialized")
            }
            AppInitializationError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {msg}")
            }
            AppInitializationError::ResourceLoadFailure(msg) => {
                write!(f, "Failed to load resources: {msg}")
            }
        }
    }
}

/// Errors that can occur during application operations.
#[derive(Debug)]
pub enum AppOperationError {
    /// The application has not been initialized
    NotInitialized,
    /// Operation is not supported
    UnsupportedOperation(String),
    /// Failed to complete operation
    OperationFailure(String),
}

impl Error for AppOperationError {}

impl fmt::Display for AppOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppOperationError::NotInitialized => {
                write!(f, "Application not initialized")
            }
            AppOperationError::UnsupportedOperation(msg) => {
                write!(f, "Unsupported operation: {msg}")
            }
            AppOperationError::OperationFailure(msg) => {
                write!(f, "Operation failed: {msg}")
            }
        }
    }
}

impl SquirrelError {
    /// Create a new security error
    pub fn security(msg: impl Into<String>) -> Self {
        Self::Security(msg.into())
    }

    /// Create a new MCP error
    pub fn mcp(msg: impl Into<String>) -> Self {
        Self::MCP(msg.into())
    }

    /// Create a new generic error
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }

    /// Create a new other error
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }

    /// Create a new health error
    pub fn health(msg: impl Into<String>) -> Self {
        Self::Health(msg.into())
    }

    /// Create a new metric error
    pub fn metric(msg: impl Into<String>) -> Self {
        Self::Metric(msg.into())
    }

    /// Create a new dashboard error
    pub fn dashboard(msg: impl Into<String>) -> Self {
        Self::Dashboard(msg.into())
    }

    /// Create a new serialization error
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    /// Create a new network error
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    /// Create a new alert error
    pub fn alert(msg: impl Into<String>) -> Self {
        Self::Alert(msg.into())
    }

    /// Create a new session error
    pub fn session(msg: impl Into<String>) -> Self {
        Self::Session(msg.into())
    }

    /// Create a new protocol version error
    pub fn protocol_version(msg: impl Into<String>) -> Self {
        Self::ProtocolVersion(msg.into())
    }

    /// Create a new context error
    pub fn context(msg: impl Into<String>) -> Self {
        Self::Context(msg.into())
    }

    /// Check if the error is recoverable
    #[must_use] pub fn is_recoverable(&self) -> bool {
        match self {
            SquirrelError::IO(_) | SquirrelError::Generic(_) | SquirrelError::AppInitialization(_) => false,
            SquirrelError::AppOperation(e) => {
                !matches!(e, AppOperationError::NotInitialized)
            },
            _ => true,
        }
    }
}

impl From<crate::monitoring::network::NetworkError> for SquirrelError {
    fn from(err: crate::monitoring::network::NetworkError) -> Self {
        SquirrelError::Network(err.to_string())
    }
}

impl From<crate::monitoring::alerts::notify::NotificationError> for SquirrelError {
    fn from(err: crate::monitoring::alerts::notify::NotificationError) -> Self {
        SquirrelError::Alert(format!("Notification error: {err}"))
    }
}

#[derive(Debug)]
/// Error types related to the alert system
pub enum AlertError {
    /// Configuration-related errors in the alert system
    Configuration(String),
    /// Errors related to sending notifications
    Notification(String),
    /// Internal errors within the alert system
    Internal(String),
}

impl fmt::Display for AlertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlertError::Configuration(msg) => write!(f, "Alert configuration error: {msg}"),
            AlertError::Notification(msg) => write!(f, "Alert notification error: {msg}"),
            AlertError::Internal(msg) => write!(f, "Alert internal error: {msg}"),
        }
    }
}

impl Error for AlertError {}

impl From<AlertError> for SquirrelError {
    fn from(err: AlertError) -> Self {
        SquirrelError::Alert(err.to_string())
    }
}

impl From<crate::context::ContextError> for SquirrelError {
    fn from(err: crate::context::ContextError) -> Self {
        match err {
            crate::context::ContextError::InvalidState(msg) => Self::Context(format!("Invalid context state: {msg}")),
            crate::context::ContextError::SnapshotNotFound(msg) => Self::Context(format!("Snapshot not found: {msg}")),
            crate::context::ContextError::NoValidSnapshot(msg) => Self::Context(format!("No valid snapshot: {msg}")),
            crate::context::ContextError::PersistenceError(msg) => Self::Context(format!("Persistence error: {msg}")),
            crate::context::ContextError::SyncError(msg) => Self::Context(format!("Synchronization error: {msg}")),
        }
    }
}

impl From<crate::mcp::context_adapter::MCPContextAdapterError> for SquirrelError {
    fn from(err: crate::mcp::context_adapter::MCPContextAdapterError) -> Self {
        match err {
            crate::mcp::context_adapter::MCPContextAdapterError::NotInitialized => 
                Self::Context("MCP context adapter not initialized".to_string()),
            crate::mcp::context_adapter::MCPContextAdapterError::AlreadyInitialized => 
                Self::Context("MCP context adapter already initialized".to_string()),
            crate::mcp::context_adapter::MCPContextAdapterError::OperationFailed(msg) => 
                Self::Context(format!("MCP context adapter operation failed: {msg}")),
        }
    }
}

impl From<ContextAdapterError> for SquirrelError {
    fn from(err: ContextAdapterError) -> Self {
        match err {
            ContextAdapterError::NotInitialized => SquirrelError::Context("Context adapter not initialized".to_string()),
            ContextAdapterError::AlreadyInitialized => SquirrelError::Context("Context adapter already initialized".to_string()),
            ContextAdapterError::OperationFailed(msg) => SquirrelError::Context(msg),
            ContextAdapterError::ContextError(e) => SquirrelError::Context(format!("Context error: {e}")),
        }
    }
}