use std::fmt;
use thiserror::Error;
use crate::error::SquirrelError;

/// Session error types
#[derive(Error, Debug)]
pub enum SessionError {
    /// Authentication error
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    /// Token error
    #[error("Token error: {0}")]
    Token(String),
    
    /// Session timeout error
    #[error("Session timeout: {0}")]
    Timeout(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// Persistence error
    #[error("Persistence error: {0}")]
    Persistence(String),
}

impl From<SessionError> for SquirrelError {
    fn from(err: SessionError) -> Self {
        SquirrelError::Session(format!("{}", err))
    }
}

/// Create an authentication error
pub fn auth_error(msg: &str) -> SquirrelError {
    SessionError::Authentication(msg.to_string()).into()
}

/// Create a token error
pub fn token_error(msg: &str) -> SquirrelError {
    SessionError::Token(msg.to_string()).into()
}

/// Create a timeout error
pub fn timeout_error(msg: &str) -> SquirrelError {
    SessionError::Timeout(msg.to_string()).into()
}

/// Create a validation error
pub fn validation_error(msg: &str) -> SquirrelError {
    SessionError::Validation(msg.to_string()).into()
}

/// Create a persistence error
pub fn persistence_error(msg: &str) -> SquirrelError {
    SessionError::Persistence(msg.to_string()).into()
} 