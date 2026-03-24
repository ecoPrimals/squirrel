// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! `DispatchOutcome<T>` — protocol vs application error separation at RPC dispatch.
//!
//! Absorbed from ecosystem consensus (groundSpring V112, loamSpine v0.9.3,
//! sweetGrass v0.7.19). Separates protocol-level errors (malformed request,
//! method not found — retryable after fix) from application-level errors
//! (business logic rejection — do not retry blindly).
//!
//! # Usage
//!
//! ```
//! use serde_json::{json, Value};
//! use universal_patterns::DispatchOutcome;
//!
//! fn run_query(_params: Value) -> Result<Value, String> {
//!     Ok(json!({"answer": 42}))
//! }
//!
//! fn handle_rpc(method: &str, params: Value) -> DispatchOutcome<Value> {
//!     match method {
//!         "system.health" => DispatchOutcome::Ok(json!({"status": "healthy"})),
//!         "ai.query" => match run_query(params) {
//!             Ok(v) => DispatchOutcome::Ok(v),
//!             Err(e) => DispatchOutcome::ApplicationError {
//!                 code: -1,
//!                 message: e.to_string(),
//!             },
//!         },
//!         _ => DispatchOutcome::ProtocolError {
//!             code: -32601,
//!             message: format!("method not found: {method}"),
//!         },
//!     }
//! }
//!
//! # fn main() {
//! # let _ = handle_rpc("system.health", json!({}));
//! # }
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// Outcome of dispatching a JSON-RPC request.
///
/// Distinguishes protocol-level failures (wire/routing) from application-level
/// failures (business logic). This lets callers make informed retry decisions:
/// - `ProtocolError` → fix the request or route, then retry
/// - `ApplicationError` → do NOT retry blindly; the handler rejected the request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DispatchOutcome<T> {
    /// Successful result from the handler.
    Ok(T),

    /// Protocol-level error: the request could not be dispatched.
    ///
    /// JSON-RPC standard codes: -32700 (parse), -32600 (invalid request),
    /// -32601 (method not found), -32602 (invalid params).
    ProtocolError {
        /// JSON-RPC error code
        code: i32,
        /// Human-readable message
        message: String,
    },

    /// Application-level error: the handler processed the request but rejected it.
    ///
    /// Uses application-specific error codes (positive or custom negative).
    ApplicationError {
        /// Application error code
        code: i32,
        /// Human-readable message
        message: String,
    },
}

impl<T> DispatchOutcome<T> {
    /// Whether this outcome is a success.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    /// Whether this is a protocol-level error (potentially retryable after fix).
    #[must_use]
    pub fn is_protocol_error(&self) -> bool {
        matches!(self, Self::ProtocolError { .. })
    }

    /// Whether this is an application-level error (do not retry blindly).
    #[must_use]
    pub fn is_application_error(&self) -> bool {
        matches!(self, Self::ApplicationError { .. })
    }

    /// Whether any kind of error occurred.
    #[must_use]
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Convert to a standard `Result`, losing the protocol/application distinction.
    pub fn into_result(self) -> Result<T, DispatchError> {
        match self {
            Self::Ok(v) => Ok(v),
            Self::ProtocolError { code, message } => Err(DispatchError::Protocol { code, message }),
            Self::ApplicationError { code, message } => {
                Err(DispatchError::Application { code, message })
            }
        }
    }

    /// Map the success value.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> DispatchOutcome<U> {
        match self {
            Self::Ok(v) => DispatchOutcome::Ok(f(v)),
            Self::ProtocolError { code, message } => {
                DispatchOutcome::ProtocolError { code, message }
            }
            Self::ApplicationError { code, message } => {
                DispatchOutcome::ApplicationError { code, message }
            }
        }
    }

    /// Convenience: create a "method not found" protocol error.
    pub fn method_not_found(method: &str) -> Self {
        Self::ProtocolError {
            code: -32601,
            message: format!("method not found: {method}"),
        }
    }

    /// Convenience: create an "invalid params" protocol error.
    pub fn invalid_params(detail: &str) -> Self {
        Self::ProtocolError {
            code: -32602,
            message: format!("invalid params: {detail}"),
        }
    }
}

/// Error extracted from a `DispatchOutcome` via `into_result()`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DispatchError {
    /// Protocol-level dispatch error.
    #[error("protocol error {code}: {message}")]
    Protocol {
        /// JSON-RPC error code
        code: i32,
        /// Human-readable message
        message: String,
    },
    /// Application-level dispatch error.
    #[error("application error {code}: {message}")]
    Application {
        /// Application error code
        code: i32,
        /// Human-readable message
        message: String,
    },
}

impl DispatchError {
    /// Whether this is a protocol error.
    #[must_use]
    pub fn is_protocol(&self) -> bool {
        matches!(self, Self::Protocol { .. })
    }

    /// Whether this is an application error.
    #[must_use]
    pub fn is_application(&self) -> bool {
        matches!(self, Self::Application { .. })
    }
}

impl<T: fmt::Display> fmt::Display for DispatchOutcome<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ok(v) => write!(f, "Ok({v})"),
            Self::ProtocolError { code, message } => {
                write!(f, "ProtocolError({code}: {message})")
            }
            Self::ApplicationError { code, message } => {
                write!(f, "ApplicationError({code}: {message})")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_variant() {
        let outcome: DispatchOutcome<i32> = DispatchOutcome::Ok(42);
        assert!(outcome.is_ok());
        assert!(!outcome.is_err());
        assert!(!outcome.is_protocol_error());
        assert!(!outcome.is_application_error());
    }

    #[test]
    fn protocol_error_variant() {
        let outcome: DispatchOutcome<i32> = DispatchOutcome::method_not_found("foo.bar");
        assert!(!outcome.is_ok());
        assert!(outcome.is_err());
        assert!(outcome.is_protocol_error());
        assert!(!outcome.is_application_error());
    }

    #[test]
    fn application_error_variant() {
        let outcome: DispatchOutcome<i32> = DispatchOutcome::ApplicationError {
            code: -1,
            message: "quota exceeded".into(),
        };
        assert!(outcome.is_application_error());
        assert!(!outcome.is_protocol_error());
    }

    #[test]
    fn into_result_ok() {
        let outcome = DispatchOutcome::Ok(42);
        assert_eq!(outcome.into_result(), Ok(42));
    }

    #[test]
    fn into_result_protocol_error() {
        let outcome: DispatchOutcome<i32> = DispatchOutcome::method_not_found("x");
        let err = outcome.into_result().unwrap_err();
        assert!(err.is_protocol());
        assert!(!err.is_application());
    }

    #[test]
    fn into_result_application_error() {
        let outcome: DispatchOutcome<i32> = DispatchOutcome::ApplicationError {
            code: -1,
            message: "nope".into(),
        };
        let err = outcome.into_result().unwrap_err();
        assert!(err.is_application());
    }

    #[test]
    fn map_transforms_value() {
        let outcome = DispatchOutcome::Ok(21);
        let mapped = outcome.map(|v| v * 2);
        assert_eq!(mapped, DispatchOutcome::Ok(42));
    }

    #[test]
    fn map_preserves_errors() {
        let outcome: DispatchOutcome<i32> = DispatchOutcome::method_not_found("x");
        let mapped = outcome.map(|v| v * 2);
        assert!(mapped.is_protocol_error());
    }

    #[test]
    fn invalid_params_convenience() {
        let outcome: DispatchOutcome<i32> = DispatchOutcome::invalid_params("missing 'query'");
        if let DispatchOutcome::ProtocolError { code, message } = &outcome {
            assert_eq!(*code, -32602);
            assert!(message.contains("missing 'query'"));
        } else {
            unreachable!("expected ProtocolError");
        }
    }

    #[test]
    fn serde_roundtrip() {
        let outcome = DispatchOutcome::Ok(serde_json::json!({"status": "ok"}));
        let json = serde_json::to_string(&outcome).expect("should succeed");
        let deser: DispatchOutcome<serde_json::Value> =
            serde_json::from_str(&json).expect("should succeed");
        assert_eq!(outcome, deser);
    }

    #[test]
    fn display_formatting() {
        let ok = DispatchOutcome::Ok(42);
        assert_eq!(ok.to_string(), "Ok(42)");

        let proto: DispatchOutcome<i32> = DispatchOutcome::method_not_found("x");
        assert!(proto.to_string().contains("ProtocolError"));

        let app: DispatchOutcome<i32> = DispatchOutcome::ApplicationError {
            code: -1,
            message: "bad".into(),
        };
        assert!(app.to_string().contains("ApplicationError"));
    }
}
