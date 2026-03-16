// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Error conversions and WASM compatibility for the Squirrel Plugin SDK

use super::context::{EnhancedError, ErrorContext};
use super::core::PluginError;
use super::severity::PluginErrorClassification;
use wasm_bindgen::prelude::*;

impl PluginError {
    /// Convert to a JsValue for WASM compatibility
    pub fn to_js_value(&self) -> JsValue {
        let error_obj = js_sys::Object::new();

        // Set error type
        js_sys::Reflect::set(&error_obj, &"type".into(), &self.error_type().into()).unwrap();

        // Set error message
        js_sys::Reflect::set(&error_obj, &"message".into(), &self.to_string().into()).unwrap();

        // Set error code
        js_sys::Reflect::set(&error_obj, &"code".into(), &self.error_code().into()).unwrap();

        // Set error category
        js_sys::Reflect::set(
            &error_obj,
            &"category".into(),
            &self.category().as_str().into(),
        )
        .unwrap();

        // Set error severity
        js_sys::Reflect::set(
            &error_obj,
            &"severity".into(),
            &self.severity().as_str().into(),
        )
        .unwrap();

        // Set recoverable flag
        js_sys::Reflect::set(
            &error_obj,
            &"recoverable".into(),
            &self.is_recoverable().into(),
        )
        .unwrap();

        error_obj.into()
    }

    /// Get a numeric error code
    pub fn error_code(&self) -> u32 {
        match self {
            PluginError::UnknownCommand { .. } => 1001,
            PluginError::MissingParameter { .. } => 1002,
            PluginError::InvalidParameter { .. } => 1003,
            PluginError::PermissionDenied { .. } => 1004,
            PluginError::NetworkError { .. } => 2001,
            PluginError::FileSystemError { .. } => 2002,
            PluginError::McpError { .. } => 2003,
            PluginError::InitializationError { .. } => 3001,
            PluginError::ConfigurationError { .. } => 3002,
            PluginError::SerializationError { .. } => 3003,
            PluginError::TimeoutError { .. } => 4001,
            PluginError::ResourceLimitExceeded { .. } => 4002,
            PluginError::QuotaExceeded { .. } => 4003,
            PluginError::PluginNotFound { .. } => 5001,
            PluginError::PluginAlreadyExists { .. } => 5002,
            PluginError::DependencyError { .. } => 5003,
            PluginError::VersionIncompatible { .. } => 5004,
            PluginError::InvalidVersion { .. } => 5005,
            PluginError::SecurityViolation { .. } => 6001,
            PluginError::InternalError { .. } => 9999,
            PluginError::ExecutionError { .. } => 7001,
            PluginError::InvalidConfiguration { .. } => 8001,
            PluginError::JsError { .. } => 9001,
            PluginError::Unknown { .. } => 9999,
            PluginError::HttpError { .. } => 2010,
            PluginError::JsonError { .. } => 3010,
            PluginError::ValidationError { .. } => 1010,
            PluginError::ConnectionError { .. } => 2011,
            PluginError::AuthenticationError { .. } => 6010,
            PluginError::AuthorizationError { .. } => 6011,
            PluginError::RateLimitError { .. } => 4010,
            PluginError::LifecycleError { .. } => 5010,
            PluginError::CommandExecutionError { .. } => 7010,
            PluginError::EventHandlingError { .. } => 7011,
            PluginError::ContextError { .. } => 8010,
            PluginError::StorageError { .. } => 2020,
            PluginError::CacheError { .. } => 2021,
            PluginError::LockError { .. } => 8020,
            PluginError::CommunicationError { .. } => 2030,
            PluginError::ResourceNotFound { .. } => 5020,
            PluginError::ResourceAlreadyExists { .. } => 5021,
            PluginError::TemporaryFailure { .. } => 4020,
            PluginError::PermanentFailure { .. } => 4021,
            PluginError::ExternalServiceError { .. } => 2040,
            PluginError::NotImplemented { .. } => 9010,
            PluginError::NotSupported { .. } => 9011,
            PluginError::Deprecated { .. } => 9012,
        }
    }

    /// Create an enhanced error with context
    pub fn with_context(self, context: ErrorContext) -> EnhancedError {
        EnhancedError {
            recoverable: self.is_recoverable(),
            recovery_suggestions: self.recovery_suggestions(),
            category: self.category(),
            severity: self.severity(),
            error: self,
            context,
            source: None,
        }
    }

    /// Create an enhanced error with source error chain
    pub fn with_source(self, source: EnhancedError) -> EnhancedError {
        let mut enhanced = self.with_context(ErrorContext::new("error_chaining"));
        enhanced.source = Some(Box::new(source));
        enhanced
    }
}

impl From<PluginError> for JsValue {
    fn from(error: PluginError) -> Self {
        error.to_js_value()
    }
}

impl From<serde_json::Error> for PluginError {
    fn from(error: serde_json::Error) -> Self {
        PluginError::JsonError {
            message: error.to_string(),
        }
    }
}

impl From<serde_wasm_bindgen::Error> for PluginError {
    fn from(error: serde_wasm_bindgen::Error) -> Self {
        PluginError::SerializationError {
            message: error.to_string(),
        }
    }
}

impl From<wasm_bindgen::JsValue> for PluginError {
    fn from(js_value: wasm_bindgen::JsValue) -> Self {
        PluginError::JsError {
            message: format!("{:?}", js_value),
        }
    }
}

impl From<std::io::Error> for PluginError {
    fn from(error: std::io::Error) -> Self {
        PluginError::FileSystemError {
            operation: "io_operation".to_string(),
            message: error.to_string(),
        }
    }
}

impl From<std::num::ParseIntError> for PluginError {
    fn from(error: std::num::ParseIntError) -> Self {
        PluginError::InvalidParameter {
            name: "numeric_value".to_string(),
            reason: format!("invalid integer: {}", error),
        }
    }
}

impl From<std::num::ParseFloatError> for PluginError {
    fn from(error: std::num::ParseFloatError) -> Self {
        PluginError::InvalidParameter {
            name: "numeric_value".to_string(),
            reason: format!("invalid float: {}", error),
        }
    }
}

impl From<std::string::FromUtf8Error> for PluginError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        PluginError::SerializationError {
            message: format!("invalid UTF-8: {}", error),
        }
    }
}

impl From<std::str::Utf8Error> for PluginError {
    fn from(error: std::str::Utf8Error) -> Self {
        PluginError::SerializationError {
            message: format!("invalid UTF-8: {}", error),
        }
    }
}

impl From<std::time::SystemTimeError> for PluginError {
    fn from(error: std::time::SystemTimeError) -> Self {
        PluginError::InternalError {
            message: format!("system time error: {}", error),
        }
    }
}

impl From<std::sync::mpsc::RecvError> for PluginError {
    fn from(error: std::sync::mpsc::RecvError) -> Self {
        PluginError::CommunicationError {
            target: "channel".to_string(),
            message: format!("receive error: {}", error),
        }
    }
}

impl<T> From<std::sync::mpsc::SendError<T>> for PluginError {
    fn from(error: std::sync::mpsc::SendError<T>) -> Self {
        PluginError::CommunicationError {
            target: "channel".to_string(),
            message: format!("send error: {}", error),
        }
    }
}

impl From<std::sync::mpsc::TryRecvError> for PluginError {
    fn from(error: std::sync::mpsc::TryRecvError) -> Self {
        match error {
            std::sync::mpsc::TryRecvError::Empty => PluginError::TemporaryFailure {
                operation: "channel_receive".to_string(),
                message: "channel is empty".to_string(),
            },
            std::sync::mpsc::TryRecvError::Disconnected => PluginError::CommunicationError {
                target: "channel".to_string(),
                message: "channel disconnected".to_string(),
            },
        }
    }
}

impl<T> From<std::sync::mpsc::TrySendError<T>> for PluginError {
    fn from(error: std::sync::mpsc::TrySendError<T>) -> Self {
        match error {
            std::sync::mpsc::TrySendError::Full(_) => PluginError::ResourceLimitExceeded {
                resource: "channel_buffer".to_string(),
                limit: "channel is full".to_string(),
            },
            std::sync::mpsc::TrySendError::Disconnected(_) => PluginError::CommunicationError {
                target: "channel".to_string(),
                message: "channel disconnected".to_string(),
            },
        }
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, ()>>> for PluginError {
    fn from(error: std::sync::PoisonError<std::sync::MutexGuard<'_, ()>>) -> Self {
        PluginError::LockError {
            resource: "mutex".to_string(),
            message: format!("mutex poisoned: {}", error),
        }
    }
}

impl From<std::sync::PoisonError<std::sync::RwLockReadGuard<'_, ()>>> for PluginError {
    fn from(error: std::sync::PoisonError<std::sync::RwLockReadGuard<'_, ()>>) -> Self {
        PluginError::LockError {
            resource: "rwlock".to_string(),
            message: format!("rwlock poisoned: {}", error),
        }
    }
}

impl From<std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, ()>>> for PluginError {
    fn from(error: std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, ()>>) -> Self {
        PluginError::LockError {
            resource: "rwlock".to_string(),
            message: format!("rwlock poisoned: {}", error),
        }
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for PluginError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        PluginError::InternalError {
            message: error.to_string(),
        }
    }
}

impl From<Box<dyn std::error::Error>> for PluginError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        PluginError::InternalError {
            message: error.to_string(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::infrastructure::error::context::ErrorContext;

    #[test]
    fn test_plugin_error_error_code() {
        assert_eq!(
            PluginError::UnknownCommand {
                command: "x".into()
            }
            .error_code(),
            1001
        );
        assert_eq!(
            PluginError::NetworkError {
                operation: "x".into(),
                message: "y".into()
            }
            .error_code(),
            2001
        );
        assert_eq!(
            PluginError::InitializationError { reason: "x".into() }.error_code(),
            3001
        );
    }

    #[test]
    fn test_plugin_error_with_context() {
        let err = PluginError::TimeoutError {
            operation: "test".into(),
            seconds: 5,
        };
        let ctx = ErrorContext::new("test_op");
        let enhanced = err.with_context(ctx);
        assert_eq!(enhanced.context.operation, "test_op");
    }

    #[test]
    fn test_from_serde_json_error() {
        let invalid = "invalid json {{{";
        let err: PluginError = serde_json::from_str::<serde_json::Value>(invalid)
            .unwrap_err()
            .into();
        assert!(matches!(err, PluginError::JsonError { .. }));
    }

    #[test]
    fn test_from_io_error() {
        let err: PluginError =
            std::io::Error::new(std::io::ErrorKind::NotFound, "file not found").into();
        assert!(matches!(err, PluginError::FileSystemError { .. }));
    }

    #[test]
    fn test_from_parse_int_error() {
        let err: PluginError = "not_a_number".parse::<i32>().unwrap_err().into();
        assert!(matches!(err, PluginError::InvalidParameter { .. }));
    }

    #[test]
    fn test_from_parse_float_error() {
        let err: PluginError = "not_a_float".parse::<f64>().unwrap_err().into();
        assert!(matches!(err, PluginError::InvalidParameter { .. }));
    }

    #[test]
    fn test_from_try_recv_error_empty() {
        let (_tx, rx) = std::sync::mpsc::channel::<i32>();
        let err: PluginError = rx.try_recv().unwrap_err().into();
        assert!(matches!(err, PluginError::TemporaryFailure { .. }));
    }

    #[test]
    fn test_from_try_recv_error_disconnected() {
        let (tx, rx) = std::sync::mpsc::channel::<i32>();
        drop(tx);
        let err: PluginError = rx.try_recv().unwrap_err().into();
        assert!(matches!(err, PluginError::CommunicationError { .. }));
    }

    #[test]
    fn test_from_recv_error() {
        let (tx, rx) = std::sync::mpsc::channel::<i32>();
        drop(tx);
        let err: PluginError = rx.recv().unwrap_err().into();
        assert!(matches!(err, PluginError::CommunicationError { .. }));
    }

    #[test]
    fn test_from_send_error() {
        let (tx, rx) = std::sync::mpsc::channel::<i32>();
        drop(rx);
        let err: PluginError = tx.send(1).unwrap_err().into();
        assert!(matches!(err, PluginError::CommunicationError { .. }));
    }
}
