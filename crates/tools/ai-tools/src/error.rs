// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Error handling for AI tools
//!
//! **DEPRECATED**: This error module is being replaced by the unified error system.
//! Please migrate to `universal-error` for all new code.
//!
//! Migration guide:
//! ```ignore
//! // Old:
//! use crate::error::{AIError, Result};
//! // New:
//! use universal_error::{Result, tools::AIToolsError};
//! ```
//!
//! For detailed migration instructions, see: `crates/universal-error/README.md`
//!
//! This module defines the common error types that can occur when
//! interacting with AI services.

use std::fmt;

/// AI Tools error type
///
/// **DEPRECATED**: Use `universal_error::tools::AIToolsError` instead.
#[deprecated(
    since = "0.2.0",
    note = "Use `universal_error::tools::AIToolsError` instead"
)]
#[derive(Debug)]
pub enum AIError {
    /// Configuration error
    Configuration(String),

    /// Network/HTTP error
    Network(String),

    /// HTTP client error
    Http(String),

    /// Provider-specific error
    Provider(String),

    /// Model-related error
    Model(String),

    /// Parsing error
    Parse(String),

    /// Rate limiting error
    RateLimit(String),

    /// Streaming error
    Streaming(String),

    /// Runtime error
    Runtime(String),

    /// Invalid response error
    InvalidResponse(String),

    /// Parsing error
    Parsing(String),

    /// API error
    ApiError(String),

    /// Authentication error
    Authentication(String),

    /// Timeout error
    Timeout(String),

    /// Validation error
    Validation(String),

    /// Invalid request error
    InvalidRequest(String),

    /// Network error (legacy alias)
    NetworkError(String),

    /// Parse error (legacy alias)
    ParseError(String),

    /// Unsupported provider error
    UnsupportedProvider(String),

    /// Generic error
    Generic(String),
}

impl fmt::Display for AIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AIError::Configuration(msg) => write!(f, "Configuration error: {msg}"),
            AIError::Network(msg) => write!(f, "Network error: {msg}"),
            AIError::Http(msg) => write!(f, "HTTP error: {msg}"),
            AIError::Provider(msg) => write!(f, "Provider error: {msg}"),
            AIError::Model(msg) => write!(f, "Model error: {msg}"),
            AIError::Parse(msg) => write!(f, "Parse error: {msg}"),
            AIError::RateLimit(msg) => write!(f, "Rate limit error: {msg}"),
            AIError::Streaming(msg) => write!(f, "Streaming error: {msg}"),
            AIError::Runtime(msg) => write!(f, "Runtime error: {msg}"),
            AIError::InvalidResponse(msg) => write!(f, "Invalid response: {msg}"),
            AIError::ApiError(msg) => write!(f, "API error: {msg}"),
            AIError::Authentication(msg) => write!(f, "Authentication error: {msg}"),
            AIError::Timeout(msg) => write!(f, "Timeout error: {msg}"),
            AIError::Validation(msg) => write!(f, "Validation error: {msg}"),
            AIError::InvalidRequest(msg) => write!(f, "Invalid request: {msg}"),
            AIError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            AIError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            AIError::UnsupportedProvider(msg) => write!(f, "Unsupported provider: {msg}"),
            AIError::Generic(msg) => write!(f, "Error: {msg}"),
            AIError::Parsing(msg) => write!(f, "Parsing error: {msg}"),
        }
    }
}

impl std::error::Error for AIError {}

// Removed: reqwest::Error conversion (old HTTP-based providers deleted)
// Use capability_ai::AiClient instead for TRUE ecoBin compliance

impl From<serde_json::Error> for AIError {
    fn from(err: serde_json::Error) -> Self {
        AIError::Parse(err.to_string())
    }
}

impl From<std::io::Error> for AIError {
    fn from(err: std::io::Error) -> Self {
        AIError::Network(err.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for AIError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        AIError::Timeout(err.to_string())
    }
}

// Bridge from new unified error system to deprecated AIError
// This allows gradual migration while maintaining compatibility
impl From<universal_error::tools::AIToolsError> for AIError {
    fn from(err: universal_error::tools::AIToolsError) -> Self {
        use universal_error::tools::AIToolsError;
        match err {
            AIToolsError::Provider(s) => AIError::Provider(s),
            AIToolsError::Router(s) => AIError::Provider(s),
            AIToolsError::Local(s) => AIError::Provider(s),
            AIToolsError::ModelNotFound(s) => AIError::Model(s),
            AIToolsError::RateLimitExceeded(s) => AIError::RateLimit(s),
            AIToolsError::InvalidResponse(s) => AIError::InvalidResponse(s),
            AIToolsError::Network(s) => AIError::NetworkError(s),
            AIToolsError::Api(s) => AIError::ApiError(s),
            AIToolsError::Configuration(s) => AIError::Configuration(s),
            AIToolsError::Parse(s) => AIError::ParseError(s),
            AIToolsError::UnsupportedProvider(s) => AIError::UnsupportedProvider(s),
            AIToolsError::InvalidRequest(s) => AIError::InvalidRequest(s),
            AIToolsError::Authentication(s) => AIError::Authentication(s),
        }
    }
}

// Result type alias
pub type Result<T> = std::result::Result<T, AIError>;

// Error type alias for convenience
pub type Error = AIError;

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;

    #[test]
    fn test_display_all_variants() {
        let cases = vec![
            (
                AIError::Configuration("bad".into()),
                "Configuration error: bad",
            ),
            (AIError::Network("timeout".into()), "Network error: timeout"),
            (AIError::Http("500".into()), "HTTP error: 500"),
            (AIError::Provider("openai".into()), "Provider error: openai"),
            (AIError::Model("not found".into()), "Model error: not found"),
            (AIError::Parse("json".into()), "Parse error: json"),
            (AIError::RateLimit("429".into()), "Rate limit error: 429"),
            (
                AIError::Streaming("broken pipe".into()),
                "Streaming error: broken pipe",
            ),
            (AIError::Runtime("panic".into()), "Runtime error: panic"),
            (
                AIError::InvalidResponse("empty".into()),
                "Invalid response: empty",
            ),
            (AIError::Parsing("syntax".into()), "Parsing error: syntax"),
            (AIError::ApiError("401".into()), "API error: 401"),
            (
                AIError::Authentication("bad key".into()),
                "Authentication error: bad key",
            ),
            (AIError::Timeout("30s".into()), "Timeout error: 30s"),
            (
                AIError::Validation("missing field".into()),
                "Validation error: missing field",
            ),
            (
                AIError::InvalidRequest("no body".into()),
                "Invalid request: no body",
            ),
            (AIError::NetworkError("dns".into()), "Network error: dns"),
            (AIError::ParseError("xml".into()), "Parse error: xml"),
            (
                AIError::UnsupportedProvider("foo".into()),
                "Unsupported provider: foo",
            ),
            (AIError::Generic("misc".into()), "Error: misc"),
        ];
        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn test_error_trait() {
        let error: Box<dyn std::error::Error> = Box::new(AIError::Network("test".into()));
        assert!(error.to_string().contains("Network error"));
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let ai_err: AIError = json_err.into();
        if let AIError::Parse(msg) = ai_err {
            assert!(!msg.is_empty());
        } else {
            panic!("Expected Parse variant");
        }
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let ai_err: AIError = io_err.into();
        if let AIError::Network(msg) = ai_err {
            assert!(msg.contains("file not found"));
        } else {
            panic!("Expected Network variant");
        }
    }

    #[test]
    fn test_from_unified_error() {
        use universal_error::tools::AIToolsError;

        let cases: Vec<(AIToolsError, fn(&AIError) -> bool)> = vec![
            (AIToolsError::Provider("p".into()), |e| {
                matches!(e, AIError::Provider(_))
            }),
            (AIToolsError::Router("r".into()), |e| {
                matches!(e, AIError::Provider(_))
            }),
            (AIToolsError::Local("l".into()), |e| {
                matches!(e, AIError::Provider(_))
            }),
            (AIToolsError::ModelNotFound("m".into()), |e| {
                matches!(e, AIError::Model(_))
            }),
            (AIToolsError::RateLimitExceeded("rl".into()), |e| {
                matches!(e, AIError::RateLimit(_))
            }),
            (AIToolsError::InvalidResponse("ir".into()), |e| {
                matches!(e, AIError::InvalidResponse(_))
            }),
            (AIToolsError::Network("n".into()), |e| {
                matches!(e, AIError::NetworkError(_))
            }),
            (AIToolsError::Api("a".into()), |e| {
                matches!(e, AIError::ApiError(_))
            }),
            (AIToolsError::Configuration("c".into()), |e| {
                matches!(e, AIError::Configuration(_))
            }),
            (AIToolsError::Parse("p".into()), |e| {
                matches!(e, AIError::ParseError(_))
            }),
            (AIToolsError::UnsupportedProvider("u".into()), |e| {
                matches!(e, AIError::UnsupportedProvider(_))
            }),
            (AIToolsError::InvalidRequest("i".into()), |e| {
                matches!(e, AIError::InvalidRequest(_))
            }),
            (AIToolsError::Authentication("a".into()), |e| {
                matches!(e, AIError::Authentication(_))
            }),
        ];

        for (unified_err, check) in cases {
            let ai_err: AIError = unified_err.into();
            assert!(check(&ai_err), "Failed for variant: {:?}", ai_err);
        }
    }

    #[test]
    fn test_result_type_alias() {
        let ok_result: Result<i32> = Ok(42);
        assert_eq!(ok_result.unwrap(), 42);

        let err_result: Result<i32> = Err(AIError::Timeout("expired".into()));
        assert!(err_result.is_err());
    }

    #[test]
    fn test_debug_format() {
        let error = AIError::Configuration("test".into());
        let debug = format!("{:?}", error);
        assert!(debug.contains("Configuration"));
    }
}
