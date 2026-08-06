// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Error conversions and WASM compatibility for the Squirrel Plugin SDK
//!
//! Common `From<X> for SDKError` impls (io, serde_json, parse, etc.) live
//! in `universal-error/src/sdk.rs` where orphan rules are satisfied.
//! This module holds the PluginError bridge and WASM-specific helpers.

use super::core::PluginError;
use universal_error::sdk::{
    ClientError, CommunicationError, InfrastructureError, SDKError,
};
use wasm_bindgen::prelude::*;

// ── PluginError → SDKError bridge (local type, orphan-safe) ───────────

impl From<PluginError> for SDKError {
    #[expect(deprecated, reason = "bridge from deprecated PluginError to SDKError")]
    fn from(err: PluginError) -> Self {
        match err {
            PluginError::McpError { message } => CommunicationError::MCP(message).into(),
            PluginError::SerializationError { message } => {
                CommunicationError::Serialization(message).into()
            }
            PluginError::JsonError { message } => {
                CommunicationError::Serialization(message).into()
            }
            PluginError::EventHandlingError {
                event_type,
                message,
            } => CommunicationError::Event(format!("{event_type}: {message}")).into(),
            PluginError::CommandExecutionError { command, message } => {
                CommunicationError::Command(format!("{command}: {message}")).into()
            }
            PluginError::CommunicationError { target, message } => {
                CommunicationError::MCP(format!("{target}: {message}")).into()
            }
            PluginError::NetworkError { operation, message } => {
                ClientError::Connection(format!("{operation}: {message}")).into()
            }
            PluginError::ConnectionError { endpoint, message } => {
                ClientError::Connection(format!("{endpoint}: {message}")).into()
            }
            PluginError::HttpError { status, message } => {
                ClientError::Http(format!("{status}: {message}")).into()
            }
            PluginError::TimeoutError { seconds, .. } => ClientError::Timeout(seconds).into(),
            PluginError::ConfigurationError { message }
            | PluginError::InvalidConfiguration { message } => {
                InfrastructureError::Configuration(message).into()
            }
            PluginError::ValidationError { field, message } => {
                InfrastructureError::Validation(format!("{field}: {message}")).into()
            }
            PluginError::InitializationError { reason } => {
                InfrastructureError::Configuration(reason).into()
            }
            PluginError::LockError { resource, message } => {
                InfrastructureError::Utility(format!("lock({resource}): {message}")).into()
            }
            PluginError::FileSystemError { operation, message } => {
                SDKError::General(format!("fs({operation}): {message}"))
            }
            PluginError::StorageError { operation, message } => {
                SDKError::General(format!("storage({operation}): {message}"))
            }
            PluginError::CacheError { operation, message } => {
                SDKError::General(format!("cache({operation}): {message}"))
            }
            PluginError::ContextError { context, message } => {
                SDKError::General(format!("context({context}): {message}"))
            }
            other => SDKError::General(other.to_string()),
        }
    }
}

// ── WASM conversions ──────────────────────────────────────────────────

/// Convert an SDKError to a JsValue for WASM interop
pub fn sdk_error_to_js_value(err: &SDKError) -> JsValue {
    let obj = js_sys::Object::new();

    let error_type = match err {
        SDKError::Infrastructure(_) => "Infrastructure",
        SDKError::Communication(_) => "Communication",
        SDKError::Client(_) => "Client",
        SDKError::General(_) => "General",
        _ => "Unknown",
    };

    let _ = js_sys::Reflect::set(&obj, &"type".into(), &error_type.into());
    let _ = js_sys::Reflect::set(&obj, &"message".into(), &err.to_string().into());

    obj.into()
}

/// Convert a JsValue to an SDKError
pub fn sdk_error_from_js_value(js: JsValue) -> SDKError {
    SDKError::General(format!("{js:?}"))
}

/// Convert a serde_wasm_bindgen::Error to an SDKError
pub fn sdk_error_from_wasm_serde(err: serde_wasm_bindgen::Error) -> SDKError {
    CommunicationError::Serialization(err.to_string()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[expect(deprecated, reason = "testing bridge from deprecated PluginError")]
    fn test_from_plugin_error_bridge() {
        let err: SDKError = PluginError::McpError {
            message: "protocol".into(),
        }
        .into();
        assert!(matches!(
            err,
            SDKError::Communication(CommunicationError::MCP(_))
        ));
    }

    #[test]
    fn test_from_serde_json_error() {
        let err: SDKError = serde_json::from_str::<serde_json::Value>("invalid json {{{")
            .unwrap_err()
            .into();
        assert!(matches!(
            err,
            SDKError::Communication(CommunicationError::Serialization(_))
        ));
    }

    #[test]
    fn test_from_io_error() {
        let err: SDKError =
            std::io::Error::new(std::io::ErrorKind::NotFound, "file not found").into();
        assert!(matches!(err, SDKError::General(_)));
        assert!(err.to_string().contains("IO:"));
    }

    #[test]
    fn test_from_parse_int_error() {
        let err: SDKError = "not_a_number".parse::<i32>().unwrap_err().into();
        assert!(matches!(
            err,
            SDKError::Infrastructure(InfrastructureError::Validation(_))
        ));
    }

    #[test]
    fn test_from_parse_float_error() {
        let err: SDKError = "not_a_float".parse::<f64>().unwrap_err().into();
        assert!(matches!(
            err,
            SDKError::Infrastructure(InfrastructureError::Validation(_))
        ));
    }

    #[test]
    fn test_from_utf8_errors() {
        let err: SDKError = String::from_utf8(vec![0xff]).unwrap_err().into();
        assert!(matches!(
            err,
            SDKError::Communication(CommunicationError::Serialization(_))
        ));
        let err2: SDKError = std::str::from_utf8(&[0xff, 0xfe]).unwrap_err().into();
        assert!(matches!(
            err2,
            SDKError::Communication(CommunicationError::Serialization(_))
        ));
    }

    #[test]
    fn test_from_system_time_error() {
        let err: SDKError = std::time::SystemTime::UNIX_EPOCH
            .duration_since(std::time::SystemTime::now())
            .unwrap_err()
            .into();
        assert!(matches!(err, SDKError::General(_)));
    }

    #[test]
    fn test_from_recv_error() {
        let (tx, rx) = std::sync::mpsc::channel::<i32>();
        drop(tx);
        let err: SDKError = rx.recv().unwrap_err().into();
        assert!(matches!(
            err,
            SDKError::Communication(CommunicationError::Event(_))
        ));
    }

    #[test]
    fn test_from_send_error() {
        let (tx, rx) = std::sync::mpsc::channel::<i32>();
        drop(rx);
        let err: SDKError = tx.send(1).unwrap_err().into();
        assert!(matches!(
            err,
            SDKError::Communication(CommunicationError::Event(_))
        ));
    }
}
