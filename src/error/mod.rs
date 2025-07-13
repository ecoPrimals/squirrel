pub mod types;

use thiserror::Error;

/// Primary error type for squirrel primal
#[derive(Error, Debug)]
pub enum PrimalError {
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Protocol error: {0}")]
    Protocol(String),
}

impl From<reqwest::Error> for PrimalError {
    fn from(error: reqwest::Error) -> Self {
        PrimalError::Network(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, PrimalError>;
