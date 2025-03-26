//! Error types for the Squirrel core library.

use std::fmt;
use std::error::Error;

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
            Self::AppInitialization(e) => write!(f, "App initialization error: {e}"),
            Self::AppOperation(e) => write!(f, "App operation error: {e}"),
            Self::Generic(msg) => write!(f, "Error: {msg}"),
            Self::IO(e) => write!(f, "IO error: {e}"),
            Self::Security(msg) => write!(f, "Security error: {msg}"),
            Self::MCP(msg) => write!(f, "MCP error: {msg}"),
            Self::Other(msg) => write!(f, "Other error: {msg}"),
            Self::Health(msg) => write!(f, "Health error: {msg}"),
            Self::Metric(msg) => write!(f, "Metric error: {msg}"),
            Self::Dashboard(msg) => write!(f, "Dashboard error: {msg}"),
            Self::Serialization(msg) => write!(f, "Serialization error: {msg}"),
            Self::Network(msg) => write!(f, "Network error: {msg}"),
            Self::Alert(msg) => write!(f, "Alert error: {msg}"),
            Self::Session(msg) => write!(f, "Session error: {msg}"),
            Self::Persistence(e) => write!(f, "Persistence error: {e}"),
            Self::ProtocolVersion(msg) => write!(f, "Protocol version error: {msg}"),
            Self::Context(msg) => write!(f, "Context error: {msg}"),
        }
    }
}

impl From<std::io::Error> for SquirrelError {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<AppInitializationError> for SquirrelError {
    fn from(err: AppInitializationError) -> Self {
        Self::AppInitialization(err)
    }
}

impl From<AppOperationError> for SquirrelError {
    fn from(err: AppOperationError) -> Self {
        Self::AppOperation(err)
    }
}

impl From<String> for SquirrelError {
    fn from(err: String) -> Self {
        Self::Generic(err)
    }
}

impl From<&str> for SquirrelError {
    fn from(err: &str) -> Self {
        Self::Generic(err.to_string())
    }
}

impl From<serde_json::Error> for SquirrelError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<PersistenceError> for SquirrelError {
    fn from(err: PersistenceError) -> Self {
        Self::Persistence(err)
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
            Self::AlreadyInitialized => {
                write!(f, "Application already initialized")
            }
            Self::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {msg}")
            }
            Self::ResourceLoadFailure(msg) => {
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
    /// The application is already started
    AlreadyStarted,
    /// The application is already stopped
    AlreadyStopped,
    /// The application is not started
    NotStarted,
}

impl Error for AppOperationError {}

impl fmt::Display for AppOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInitialized => {
                write!(f, "Application not initialized")
            }
            Self::UnsupportedOperation(msg) => {
                write!(f, "Unsupported operation: {msg}")
            }
            Self::OperationFailure(msg) => {
                write!(f, "Operation failed: {msg}")
            }
            Self::AlreadyStarted => {
                write!(f, "Application is already started")
            }
            Self::AlreadyStopped => {
                write!(f, "Application is already stopped")
            }
            Self::NotStarted => {
                write!(f, "Application is not started")
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

    /// Create a new monitoring error
    /// 
    /// This creates a `Metric` error which is used for general monitoring functionality
    pub fn monitoring(msg: impl Into<String>) -> Self {
        Self::Metric(msg.into())
    }

    /// Check if the error is recoverable
    #[must_use] 
    pub const fn is_recoverable(&self) -> bool {
        match self {
            Self::IO(_) | Self::Generic(_) | Self::AppInitialization(_) => false,
            Self::AppOperation(e) => {
                matches!(e, AppOperationError::AlreadyStarted | AppOperationError::AlreadyStopped)
            },
            _ => true,
        }
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
            Self::Configuration(msg) => write!(f, "Alert configuration error: {msg}"),
            Self::Notification(msg) => write!(f, "Alert notification error: {msg}"),
            Self::Internal(msg) => write!(f, "Alert internal error: {msg}"),
        }
    }
}

impl Error for AlertError {}

impl From<AlertError> for SquirrelError {
    fn from(err: AlertError) -> Self {
        Self::Alert(err.to_string())
    }
}