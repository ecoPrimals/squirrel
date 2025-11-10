//! Integration Error Types
//!
//! This module provides error types for integration components including web
//! integrations, API clients, context adapters, and ecosystem connections,
//! following the MCP error architecture pattern.
//!
//! # Architecture
//!
//! ```text
//! IntegrationError
//!     ├── Web (auth, database, API)
//!     ├── APIClient (HTTP, Anthropic, OpenAI, GitHub)
//!     ├── ContextAdapter (adapter, conversion)
//!     ├── Ecosystem (registry, client)
//!     └── General (catch-all for integration operations)
//! ```
//!
//! # Examples
//!
//! ```
//! use universal_error::integration::{IntegrationError, APIClientError};
//!
//! fn call_external_api(endpoint: &str) -> Result<String, IntegrationError> {
//!     if endpoint.is_empty() {
//!         return Err(APIClientError::Http(
//!             "Endpoint cannot be empty".to_string()
//!         ).into());
//!     }
//!     Ok("response".to_string())
//! }
//! ```

use thiserror::Error;
use super::{ErrorContextTrait, ErrorSeverity};

/// Top-level Integration error type
///
/// This encompasses all integration-related errors with automatic conversions
/// from sub-domain errors via `#[from]` attribute.
#[derive(Error, Debug, Clone)]
pub enum IntegrationError {
    /// Error originating from web integrations
    #[error(transparent)]
    Web(#[from] WebError),
    
    /// Error originating from API clients
    #[error(transparent)]
    APIClient(#[from] APIClientError),
    
    /// Error originating from context adapters
    #[error(transparent)]
    ContextAdapter(#[from] ContextAdapterError),
    
    /// Error originating from ecosystem connections
    #[error(transparent)]
    Ecosystem(#[from] EcosystemError),
    
    /// General integration error
    #[error("Integration error: {0}")]
    General(String),
}

/// Web integration-related errors
///
/// Covers authentication, database, and API operations.
#[derive(Error, Debug, Clone)]
pub enum WebError {
    /// Authentication error
    #[error("Authentication error: {0}")]
    Auth(String),
    
    /// Database error
    #[error("Database error: {0}")]
    Database(String),
    
    /// API error
    #[error("API error: {0}")]
    API(String),
    
    /// Session error
    #[error("Session error: {0}")]
    Session(String),
    
    /// MFA error
    #[error("Multi-factor authentication error: {0}")]
    MFA(String),
}

/// API Client-related errors
///
/// Covers HTTP clients and external API integrations.
#[derive(Error, Debug, Clone)]
pub enum APIClientError {
    /// HTTP client error
    #[error("HTTP client error: {0}")]
    Http(String),
    
    /// Anthropic API error
    #[error("Anthropic API error: {0}")]
    Anthropic(String),
    
    /// OpenAI API error
    #[error("OpenAI API error: {0}")]
    OpenAI(String),
    
    /// GitHub API error
    #[error("GitHub API error: {0}")]
    GitHub(String),
    
    /// API rate limit error
    #[error("API rate limit exceeded for {0}")]
    RateLimitExceeded(String),
    
    /// Invalid API key
    #[error("Invalid API key for {0}")]
    InvalidAPIKey(String),
    
    /// Request timeout
    #[error("Request timeout after {0} seconds")]
    Timeout(u64),
}

/// Context Adapter-related errors
///
/// Covers context adaptation and conversion operations.
#[derive(Error, Debug, Clone)]
pub enum ContextAdapterError {
    /// Adapter error
    #[error("Adapter error: {0}")]
    Adapter(String),
    
    /// Type conversion error
    #[error("Type conversion error: {0}")]
    Conversion(String),
    
    /// Incompatible context
    #[error("Incompatible context: {0}")]
    IncompatibleContext(String),
    
    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Ecosystem-related errors
///
/// Covers ecosystem registry and client operations.
#[derive(Error, Debug, Clone)]
pub enum EcosystemError {
    /// Registry error
    #[error("Registry error: {0}")]
    Registry(String),
    
    /// Client error
    #[error("Client error: {0}")]
    Client(String),
    
    /// Service not found
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    
    /// Registration failed
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),
}

// Implement ErrorContextTrait for Integration errors following MCP pattern
impl ErrorContextTrait for IntegrationError {
    
    fn severity(&self) -> ErrorSeverity {
        match self {
            IntegrationError::Web(WebError::Auth(_)) => ErrorSeverity::High,
            IntegrationError::Web(WebError::Database(_)) => ErrorSeverity::Critical,
            IntegrationError::APIClient(APIClientError::RateLimitExceeded(_)) => ErrorSeverity::Medium,
            IntegrationError::Ecosystem(_) => ErrorSeverity::High,
            _ => ErrorSeverity::Medium,
        }
    }
    
    fn component(&self) -> Option<&str> {
        Some("Integration")
    }
    
    fn is_recoverable(&self) -> bool {
        match self {
            IntegrationError::APIClient(APIClientError::RateLimitExceeded(_)) => true,
            IntegrationError::APIClient(APIClientError::Timeout(_)) => true,
            IntegrationError::ContextAdapter(ContextAdapterError::MissingField(_)) => true,
            _ => false,
        }
    }
}

impl ErrorContextTrait for WebError {
    
    fn severity(&self) -> ErrorSeverity {
        match self {
            WebError::Auth(_) => ErrorSeverity::High,
            WebError::Database(_) => ErrorSeverity::Critical,
            WebError::MFA(_) => ErrorSeverity::High,
            _ => ErrorSeverity::Medium,
        }
    }
    
    fn component(&self) -> Option<&str> {
        Some("Integration.Web")
    }
}

impl ErrorContextTrait for APIClientError {
    
    fn severity(&self) -> ErrorSeverity {
        match self {
            APIClientError::InvalidAPIKey(_) => ErrorSeverity::High,
            APIClientError::RateLimitExceeded(_) => ErrorSeverity::Medium,
            _ => ErrorSeverity::Medium,
        }
    }
    
    fn component(&self) -> Option<&str> {
        Some("Integration.APIClient")
    }
    
    fn is_recoverable(&self) -> bool {
        matches!(
            self,
            APIClientError::RateLimitExceeded(_) | APIClientError::Timeout(_)
        )
    }
}

impl ErrorContextTrait for ContextAdapterError {
    
    fn severity(&self) -> ErrorSeverity {
        match self {
            ContextAdapterError::IncompatibleContext(_) => ErrorSeverity::High,
            _ => ErrorSeverity::Medium,
        }
    }
    
    fn component(&self) -> Option<&str> {
        Some("Integration.ContextAdapter")
    }
    
    fn is_recoverable(&self) -> bool {
        matches!(self, ContextAdapterError::MissingField(_))
    }
}

impl ErrorContextTrait for EcosystemError {
    
    fn severity(&self) -> ErrorSeverity {
        match self {
            EcosystemError::ServiceNotFound(_) => ErrorSeverity::High,
            EcosystemError::RegistrationFailed(_) => ErrorSeverity::High,
            _ => ErrorSeverity::Medium,
        }
    }
    
    fn component(&self) -> Option<&str> {
        Some("Integration.Ecosystem")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_web_error() {
        let err = WebError::Auth("invalid credentials".to_string());
        assert_eq!(err.severity(), ErrorSeverity::High);
    }
    
    #[test]
    fn test_api_client_error() {
        let err = APIClientError::RateLimitExceeded("OpenAI".to_string());
        assert!(err.is_recoverable());
    }
    
    #[test]
    fn test_context_adapter_error() {
        let err = ContextAdapterError::MissingField("user_id".to_string());
        assert!(err.is_recoverable());
    }
    
    #[test]
    fn test_ecosystem_error() {
        let err = EcosystemError::ServiceNotFound("auth-service".to_string());
        assert_eq!(err.severity(), ErrorSeverity::High);
    }
    
    #[test]
    fn test_integration_error_conversion() {
        let web_err = WebError::Database("connection failed".to_string());
        let integ_err: IntegrationError = web_err.into();
        assert!(matches!(integ_err, IntegrationError::Web(_)));
        assert_eq!(integ_err.component(), Some("Integration"));
    }
}

