//! Error types for the Squirrel project
//!
//! This module defines the main error types and results used throughout the project.

use std::fmt;
use std::error::Error as StdError;
use std::io;

/// Main error type for the Squirrel project
#[derive(Debug)]
pub enum SquirrelError {
    Io(io::Error),
    Health(String),
    Metric(String),
    Alert(String),
    Dashboard(String),
    Monitoring(String),
    Protocol(String),
    Context(String),
    Command(String),
    Other(String),
}

impl SquirrelError {
    pub fn health<T: ToString + ?Sized>(msg: &T) -> Self {
        SquirrelError::Health(msg.to_string())
    }
    
    pub fn metric<T: ToString + ?Sized>(msg: &T) -> Self {
        SquirrelError::Metric(msg.to_string())
    }
    
    pub fn alert<T: ToString + ?Sized>(msg: &T) -> Self {
        SquirrelError::Alert(msg.to_string())
    }
    
    pub fn dashboard<T: ToString + ?Sized>(msg: &T) -> Self {
        SquirrelError::Dashboard(msg.to_string())
    }
    
    pub fn monitoring<T: ToString + ?Sized>(msg: &T) -> Self {
        SquirrelError::Monitoring(msg.to_string())
    }
    
    pub fn protocol<T: ToString + ?Sized>(msg: &T) -> Self {
        SquirrelError::Protocol(msg.to_string())
    }
    
    pub fn context<T: ToString + ?Sized>(msg: &T) -> Self {
        SquirrelError::Context(msg.to_string())
    }
    
    pub fn command_not_found<T: ToString>(name: &T) -> Self {
        SquirrelError::Command(format!("Command not found: {name}", name = name.to_string()))
    }
}

impl fmt::Display for SquirrelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SquirrelError::Io(err) => write!(f, "IO error: {err}"),
            SquirrelError::Health(msg) => write!(f, "Health error: {msg}"),
            SquirrelError::Metric(msg) => write!(f, "Metric error: {msg}"),
            SquirrelError::Alert(msg) => write!(f, "Alert error: {msg}"),
            SquirrelError::Dashboard(msg) => write!(f, "Dashboard error: {msg}"),
            SquirrelError::Monitoring(msg) => write!(f, "Monitoring error: {msg}"),
            SquirrelError::Protocol(msg) => write!(f, "Protocol error: {msg}"),
            SquirrelError::Context(msg) => write!(f, "Context error: {msg}"),
            SquirrelError::Command(msg) => write!(f, "Command error: {msg}"),
            SquirrelError::Other(msg) => write!(f, "Error: {msg}"),
        }
    }
}

impl StdError for SquirrelError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            SquirrelError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for SquirrelError {
    fn from(err: io::Error) -> Self {
        SquirrelError::Io(err)
    }
}

impl From<String> for SquirrelError {
    fn from(err: String) -> Self {
        SquirrelError::Other(err)
    }
}

impl From<&str> for SquirrelError {
    fn from(err: &str) -> Self {
        SquirrelError::Other(err.to_string())
    }
}

impl From<serde_json::Error> for SquirrelError {
    fn from(err: serde_json::Error) -> Self {
        SquirrelError::Other(err.to_string())
    }
}

impl From<crate::commands::CommandError> for SquirrelError {
    fn from(err: crate::commands::CommandError) -> Self {
        SquirrelError::Command(err.to_string())
    }
}

impl From<crate::monitoring::network::NetworkError> for SquirrelError {
    fn from(err: crate::monitoring::network::NetworkError) -> Self {
        match err {
            crate::monitoring::network::NetworkError::System(msg) => SquirrelError::Monitoring(format!("Network system error: {msg}")),
            crate::monitoring::network::NetworkError::Metrics(msg) => SquirrelError::Metric(format!("Network metrics error: {msg}")),
        }
    }
}

/// A Result type alias for operations that may return a `SquirrelError`
pub type Result<T> = std::result::Result<T, SquirrelError>;