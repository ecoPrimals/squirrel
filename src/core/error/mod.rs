//! Error types for the Squirrel project
//!
//! This module defines the core error types used throughout the project.
//! It provides a unified error handling approach with custom error types
//! for different components of the system.

use std::error::Error as StdError;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

/// The main error type for the Squirrel project
#[derive(Debug)]
pub enum Error {
    /// Context-related errors
    Context(ContextError),
    /// Command-related errors
    Command(CommandError),
    /// Event-related errors
    Event(EventError),
    /// Metrics-related errors
    Metrics(MetricsError),
    /// IO errors
    Io(std::io::Error),
    /// Serialization errors
    Serialization(serde_json::Error),
    /// Other errors
    Other(Box<dyn std::error::Error + Send + Sync>),
}

/// Context-specific errors
#[derive(Debug)]
pub enum ContextError {
    /// Error when initializing context
    Initialization(String),
    /// Error when shutting down context
    Shutdown(String),
    /// Error when accessing context state
    State(String),
    /// Error when managing context lifecycle
    Lifecycle(String),
}

/// Command-specific errors
#[derive(Debug)]
pub enum CommandError {
    /// Error when executing a command
    Execution(String),
    /// Error when validating a command
    Validation(String),
    /// Error when handling a command
    Handler(String),
}

/// Event-specific errors
#[derive(Debug)]
pub enum EventError {
    /// Error when emitting an event
    Emission(String),
    /// Error when handling an event
    Handling(String),
    /// Error when processing an event
    Processing(String),
}

/// Metrics-specific errors
#[derive(Debug)]
pub enum MetricsError {
    /// Error when collecting metrics
    Collection(String),
    /// Error when reporting metrics
    Reporting(String),
    /// Error when processing metrics
    Processing(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Context(e) => write!(f, "Context error: {}", e),
            Error::Command(e) => write!(f, "Command error: {}", e),
            Error::Event(e) => write!(f, "Event error: {}", e),
            Error::Metrics(e) => write!(f, "Metrics error: {}", e),
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Serialization(e) => write!(f, "Serialization error: {}", e),
            Error::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContextError::Initialization(e) => write!(f, "Initialization error: {}", e),
            ContextError::Shutdown(e) => write!(f, "Shutdown error: {}", e),
            ContextError::State(e) => write!(f, "State error: {}", e),
            ContextError::Lifecycle(e) => write!(f, "Lifecycle error: {}", e),
        }
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::Execution(e) => write!(f, "Execution error: {}", e),
            CommandError::Validation(e) => write!(f, "Validation error: {}", e),
            CommandError::Handler(e) => write!(f, "Handler error: {}", e),
        }
    }
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventError::Emission(e) => write!(f, "Emission error: {}", e),
            EventError::Handling(e) => write!(f, "Handling error: {}", e),
            EventError::Processing(e) => write!(f, "Processing error: {}", e),
        }
    }
}

impl fmt::Display for MetricsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricsError::Collection(e) => write!(f, "Collection error: {}", e),
            MetricsError::Reporting(e) => write!(f, "Reporting error: {}", e),
            MetricsError::Processing(e) => write!(f, "Processing error: {}", e),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Context(_) => None,
            Error::Command(_) => None,
            Error::Event(_) => None,
            Error::Metrics(_) => None,
            Error::Io(e) => Some(e),
            Error::Serialization(e) => Some(e),
            Error::Other(e) => Some(e.as_ref()),
        }
    }
}

impl StdError for ContextError {}
impl StdError for CommandError {}
impl StdError for EventError {}
impl StdError for MetricsError {}

// Implement From for specific error types
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serialization(err)
    }
}

impl From<ContextError> for Error {
    fn from(err: ContextError) -> Self {
        Error::Context(err)
    }
}

impl From<CommandError> for Error {
    fn from(err: CommandError) -> Self {
        Error::Command(err)
    }
}

impl From<EventError> for Error {
    fn from(err: EventError) -> Self {
        Error::Event(err)
    }
}

impl From<MetricsError> for Error {
    fn from(err: MetricsError) -> Self {
        Error::Metrics(err)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Error::Other(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::Other(Box::new(err) as Box<dyn std::error::Error + Send + Sync>)
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::Other(Box::new(err.to_string()) as Box<dyn std::error::Error + Send + Sync>)
    }
} 