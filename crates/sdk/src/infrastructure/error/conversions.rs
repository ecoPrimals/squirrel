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
#[allow(clippy::unwrap_used, clippy::expect_used)] // Invariant or startup failure: unwrap/expect after validation
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

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_plugin_error_to_js_value_and_from_jsvalue() {
        use wasm_bindgen::JsValue;

        let e = PluginError::UnknownCommand {
            command: "c".into(),
        };
        let js = e.to_js_value();
        assert!(!format!("{js:?}").is_empty());
        let pe: PluginError = JsValue::from_str("x").into();
        assert!(matches!(pe, PluginError::JsError { .. }));
        let js2: JsValue = PluginError::NetworkError {
            operation: "o".into(),
            message: "m".into(),
        }
        .into();
        assert!(!format!("{js2:?}").is_empty());
    }

    #[test]
    #[expect(clippy::too_many_lines, reason = "Exhaustive variant coverage table")]
    fn test_error_code_all_variants() {
        let cases: Vec<PluginError> = vec![
            PluginError::MissingParameter {
                parameter: "p".into(),
            },
            PluginError::InvalidParameter {
                name: "n".into(),
                reason: "r".into(),
            },
            PluginError::PermissionDenied {
                operation: "o".into(),
                reason: "r".into(),
            },
            PluginError::FileSystemError {
                operation: "o".into(),
                message: "m".into(),
            },
            PluginError::McpError {
                message: "m".into(),
            },
            PluginError::ConfigurationError {
                message: "m".into(),
            },
            PluginError::QuotaExceeded {
                resource: "r".into(),
                message: "m".into(),
            },
            PluginError::PluginNotFound {
                plugin_id: "p".into(),
            },
            PluginError::DependencyError {
                dependency: "d".into(),
                message: "m".into(),
            },
            PluginError::VersionIncompatible {
                required: "1".into(),
                found: "2".into(),
            },
            PluginError::InvalidVersion {
                version: "v".into(),
                reason: "r".into(),
            },
            PluginError::SecurityViolation {
                violation: "v".into(),
            },
            PluginError::ExecutionError {
                context: "c".into(),
                message: "m".into(),
            },
            PluginError::Unknown {
                message: "m".into(),
            },
            PluginError::HttpError {
                status: 400,
                message: "m".into(),
            },
            PluginError::ValidationError {
                field: "f".into(),
                message: "m".into(),
            },
            PluginError::ConnectionError {
                endpoint: "e".into(),
                message: "m".into(),
            },
            PluginError::AuthenticationError {
                message: "m".into(),
            },
            PluginError::AuthorizationError {
                resource: "r".into(),
                message: "m".into(),
            },
            PluginError::RateLimitError {
                resource: "r".into(),
                retry_after: 1,
            },
            PluginError::LifecycleError {
                state: "s".into(),
                target_state: "t".into(),
                message: "m".into(),
            },
            PluginError::CommandExecutionError {
                command: "c".into(),
                message: "m".into(),
            },
            PluginError::EventHandlingError {
                event_type: "e".into(),
                message: "m".into(),
            },
            PluginError::ContextError {
                context: "c".into(),
                message: "m".into(),
            },
            PluginError::StorageError {
                operation: "o".into(),
                message: "m".into(),
            },
            PluginError::CacheError {
                operation: "o".into(),
                message: "m".into(),
            },
            PluginError::ResourceNotFound {
                resource: "r".into(),
            },
            PluginError::ResourceAlreadyExists {
                resource: "r".into(),
            },
            PluginError::PermanentFailure {
                operation: "o".into(),
                message: "m".into(),
            },
            PluginError::ExternalServiceError {
                service: "s".into(),
                message: "m".into(),
            },
            PluginError::NotImplemented {
                feature: "f".into(),
            },
            PluginError::NotSupported {
                feature: "f".into(),
            },
            PluginError::Deprecated {
                feature: "f".into(),
                alternative: "a".into(),
            },
        ];
        for err in cases {
            let _ = err.error_code();
            let _ = err.to_string();
        }
    }

    #[test]
    fn test_with_source_chaining() {
        let inner = PluginError::TimeoutError {
            operation: "a".into(),
            seconds: 1,
        };
        let enhanced = inner.with_context(ErrorContext::new("op1"));
        let outer = PluginError::InternalError {
            message: "outer".into(),
        }
        .with_source(enhanced);
        assert!(outer.source.is_some());
    }

    #[test]
    fn test_from_utf8_errors() {
        let err: PluginError = String::from_utf8(vec![0xff]).unwrap_err().into();
        assert!(matches!(err, PluginError::SerializationError { .. }));
        let bad_utf8 = vec![0xff_u8, 0xfe, 0xfd];
        let err2: PluginError = std::str::from_utf8(&bad_utf8).unwrap_err().into();
        assert!(matches!(err2, PluginError::SerializationError { .. }));
    }

    #[test]
    fn test_from_system_time_error() {
        let err: PluginError = std::time::SystemTime::UNIX_EPOCH
            .duration_since(std::time::SystemTime::now())
            .unwrap_err()
            .into();
        assert!(matches!(err, PluginError::InternalError { .. }));
    }

    #[test]
    fn test_from_try_send_errors() {
        let (tx, rx) = std::sync::mpsc::sync_channel::<i32>(0);
        let err: PluginError = tx.try_send(1).unwrap_err().into();
        assert!(matches!(err, PluginError::ResourceLimitExceeded { .. }));
        drop(rx);
        let err2: PluginError = tx.try_send(1).unwrap_err().into();
        assert!(matches!(err2, PluginError::CommunicationError { .. }));
    }

    #[test]
    fn test_from_boxed_error() {
        let b: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::other("e"));
        let err: PluginError = b.into();
        assert!(matches!(err, PluginError::InternalError { .. }));

        let b2: Box<dyn std::error::Error> = Box::new(std::io::Error::other("e"));
        let err2: PluginError = b2.into();
        assert!(matches!(err2, PluginError::InternalError { .. }));
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_from_serde_wasm_bindgen_error() {
        use wasm_bindgen::JsValue;

        let e: serde_wasm_bindgen::Error =
            serde_wasm_bindgen::from_value::<i32>(JsValue::NULL).unwrap_err();
        let err: PluginError = e.into();
        assert!(matches!(err, PluginError::SerializationError { .. }));
    }

    #[test]
    fn test_from_poison_errors() {
        let m = std::sync::Mutex::new(());
        let _ = std::panic::catch_unwind(|| {
            let _g = m.lock().unwrap();
            std::panic::resume_unwind(Box::new("intentional mutex poison for test"));
        });
        let err: PluginError = m.lock().unwrap_err().into();
        assert!(matches!(err, PluginError::LockError { .. }));

        let rw = std::sync::RwLock::new(());
        let _ = std::panic::catch_unwind(|| {
            let _g = rw.write().unwrap();
            std::panic::resume_unwind(Box::new("intentional rwlock poison for test"));
        });
        let err_r: PluginError = rw.read().unwrap_err().into();
        assert!(matches!(err_r, PluginError::LockError { .. }));
        let err_w: PluginError = rw.write().unwrap_err().into();
        assert!(matches!(err_w, PluginError::LockError { .. }));
    }
}
