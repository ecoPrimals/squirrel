//! Error handling types and utilities for Squirrel

use thiserror::Error;

/// A type alias for a Result with a `SquirrelError`
pub type Result<T> = std::result::Result<T, SquirrelError>;

/// Persistence-related error types
#[derive(Error, Debug)]
pub enum PersistenceError {
    /// Error related to storage operations
    #[error("Storage error: {0}")]
    Storage(String),

    /// Error related to data serialization/deserialization
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Error related to invalid data
    #[error("Invalid data error: {0}")]
    InvalidData(String),

    /// Error related to data not found
    #[error("Data not found: {0}")]
    NotFound(String),
}

/// The main error type for Squirrel
#[derive(Error, Debug)]
pub enum SquirrelError {
    /// A generic error
    #[error("Generic error: {0}")]
    Generic(String),

    /// An error related to MCP
    #[error("MCP error: {0}")]
    MCP(String),

    /// An error related to context
    #[error("Context error: {0}")]
    Context(String),

    /// An error related to plugins
    #[error("Plugin error: {0}")]
    Plugin(String),

    /// An error related to I/O
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    /// An error related to serialization/deserialization
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// An error related to networking
    #[error("Network error: {0}")]
    Network(String),

    /// An error related to authentication
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// An error related to authorization
    #[error("Authorization error: {0}")]
    Authorization(String),

    /// An error related to validation
    #[error("Validation error: {0}")]
    Validation(String),

    /// An error related to configuration
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// An error from an external service
    #[error("External service error: {0}")]
    ExternalService(String),

    /// An error related to timing out
    #[error("Timeout error: {0}")]
    Timeout(String),

    /// An error related to persistence
    #[error("Persistence error: {0}")]
    Persistence(PersistenceError),

    /// An error related to session management
    #[error("Session error: {0}")]
    Session(String),

    /// An error with additional context
    #[error("{context}: {source}")]
    WithContext {
        /// The source error
        source: Box<SquirrelError>,
        /// Additional context for the error
        context: String,
    },
}

impl SquirrelError {
    /// Create a new generic error
    pub fn generic<S: Into<String>>(msg: S) -> Self {
        SquirrelError::Generic(msg.into())
    }

    /// Create a new MCP error
    pub fn mcp<S: Into<String>>(msg: S) -> Self {
        SquirrelError::MCP(msg.into())
    }

    /// Create a new context error
    pub fn context<S: Into<String>>(msg: S) -> Self {
        SquirrelError::Context(msg.into())
    }

    /// Create a new plugin error
    pub fn plugin<S: Into<String>>(msg: S) -> Self {
        SquirrelError::Plugin(msg.into())
    }

    /// Create a new serialization error
    pub fn serialization<S: Into<String>>(msg: S) -> Self {
        SquirrelError::Serialization(msg.into())
    }

    /// Create a new network error
    pub fn network<S: Into<String>>(msg: S) -> Self {
        SquirrelError::Network(msg.into())
    }

    /// Create a new authentication error
    pub fn authentication<S: Into<String>>(msg: S) -> Self {
        SquirrelError::Authentication(msg.into())
    }

    /// Create a new authorization error
    pub fn authorization<S: Into<String>>(msg: S) -> Self {
        SquirrelError::Authorization(msg.into())
    }

    /// Create a new validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        SquirrelError::Validation(msg.into())
    }

    /// Create a new configuration error
    pub fn configuration<S: Into<String>>(msg: S) -> Self {
        SquirrelError::Configuration(msg.into())
    }

    /// Create a new external service error
    pub fn external_service<S: Into<String>>(msg: S) -> Self {
        SquirrelError::ExternalService(msg.into())
    }

    /// Create a new timeout error
    pub fn timeout<S: Into<String>>(msg: S) -> Self {
        SquirrelError::Timeout(msg.into())
    }

    /// Create a new persistence error
    #[must_use] 
    pub fn persistence(err: PersistenceError) -> Self {
        SquirrelError::Persistence(err)
    }

    /// Create a new persistence storage error
    pub fn persistence_storage<S: Into<String>>(msg: S) -> Self {
        SquirrelError::Persistence(PersistenceError::Storage(msg.into()))
    }

    /// Create a new session error
    pub fn session<S: Into<String>>(msg: S) -> Self {
        SquirrelError::Session(msg.into())
    }

    /// Add context to an error
    pub fn with_context<S: Into<String>>(self, context: S) -> Self {
        SquirrelError::WithContext {
            source: Box::new(self),
            context: context.into(),
        }
    }

    /// Check if the error is recoverable
    #[must_use] 
    pub fn is_recoverable(&self) -> bool {
        match self {
            SquirrelError::Network(_)
            | SquirrelError::Timeout(_)
            | SquirrelError::ExternalService(_) => true,
            SquirrelError::WithContext { source, .. } => source.is_recoverable(),
            _ => false,
        }
    }
}

impl From<anyhow::Error> for SquirrelError {
    fn from(err: anyhow::Error) -> Self {
        SquirrelError::Generic(err.to_string())
    }
}

impl From<serde_json::Error> for SquirrelError {
    fn from(err: serde_json::Error) -> Self {
        SquirrelError::Serialization(err.to_string())
    }
}

impl From<&str> for SquirrelError {
    fn from(s: &str) -> Self {
        SquirrelError::Generic(s.to_string())
    }
}

impl From<String> for SquirrelError {
    fn from(s: String) -> Self {
        SquirrelError::Generic(s)
    }
}

/// A trait for errors that can be converted to a `SquirrelError`
pub trait IntoSquirrelError {
    /// Convert the error to a `SquirrelError`
    fn into_squirrel_error(self) -> SquirrelError;

    /// Convert the error to a `SquirrelError` with context
    fn into_squirrel_error_with_context<C: Into<String>>(self, context: C) -> SquirrelError;
}

impl<E: std::error::Error + Send + Sync + 'static> IntoSquirrelError for E {
    fn into_squirrel_error(self) -> SquirrelError {
        SquirrelError::Generic(self.to_string())
    }

    fn into_squirrel_error_with_context<C: Into<String>>(self, context: C) -> SquirrelError {
        SquirrelError::WithContext {
            source: Box::new(SquirrelError::Generic(self.to_string())),
            context: context.into(),
        }
    }
}
