// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Modernized Universal Adapter Types with ``Arc<str>`` Optimization
//!
//! This module provides modernized universal adapter types that use ``Arc<str>``
//! for dramatic performance improvements in request/response handling.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;

/// String interning for common universal adapter values (modern idiomatic Rust: std::sync::LazyLock)
static UNIVERSAL_STRINGS: LazyLock<HashMap<&'static str, Arc<str>>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    // Common operations
    map.insert("health_check", Arc::from("health_check"));
    map.insert("get_capabilities", Arc::from("get_capabilities"));
    map.insert("process_request", Arc::from("process_request"));
    map.insert("get_status", Arc::from("get_status"));
    map.insert("register_service", Arc::from("register_service"));
    map.insert("discover_services", Arc::from("discover_services"));

    // Common service names
    map.insert("ai_coordinator", Arc::from("ai_coordinator"));
    map.insert("context_manager", Arc::from("context_manager"));
    map.insert("plugin_manager", Arc::from("plugin_manager"));
    map.insert("security_service", Arc::from("security_service"));
    map.insert("metrics_collector", Arc::from("metrics_collector"));

    // Common status values
    map.insert("success", Arc::from("success"));
    map.insert("error", Arc::from("error"));
    map.insert("pending", Arc::from("pending"));
    map.insert("partial", Arc::from("partial"));
    map.insert("timeout", Arc::from("timeout"));

    // Common metadata keys
    map.insert("version", Arc::from("version"));
    map.insert("timestamp", Arc::from("timestamp"));
    map.insert("processing_time", Arc::from("processing_time"));
    map.insert("request_id", Arc::from("request_id"));
    map.insert("user_id", Arc::from("user_id"));
    map.insert("session_id", Arc::from("session_id"));

    map
});

/// Get ``Arc<str>`` for universal adapter string with zero allocation for common values
pub fn intern_universal_string(s: &str) -> Arc<str> {
    UNIVERSAL_STRINGS
        .get(s)
        .cloned()
        .unwrap_or_else(|| Arc::from(s))
}

/// Universal request format with ``Arc<str>`` optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalRequest {
    /// Request ID as `Arc<str>` for efficient sharing
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub request_id: Arc<str>,

    /// Operation name with string interning
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub operation: Arc<str>,

    /// Parameters with `Arc<str>` keys and Arc<serde_json::Value> values
    #[serde(
        serialize_with = "serialize_parameters",
        deserialize_with = "deserialize_parameters"
    )]
    pub parameters: HashMap<Arc<str>, Arc<serde_json::Value>>,

    /// Context with `Arc<str>` keys and Arc<serde_json::Value> values
    #[serde(
        serialize_with = "serialize_parameters",
        deserialize_with = "deserialize_parameters"
    )]
    pub context: HashMap<Arc<str>, Arc<serde_json::Value>>,

    /// Requester as `Arc<str>`
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub requester: Arc<str>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl UniversalRequest {
    /// Create new UniversalRequest with string interning optimization
    pub fn new(request_id: &str, operation: &str, requester: &str) -> Self {
        Self {
            request_id: Arc::from(request_id),
            operation: intern_universal_string(operation),
            parameters: HashMap::new(),
            context: HashMap::new(),
            requester: Arc::from(requester),
            timestamp: Utc::now(),
        }
    }

    /// Add parameter efficiently using string interning
    pub fn add_parameter(&mut self, key: &str, value: serde_json::Value) {
        let key_arc = intern_universal_string(key);
        self.parameters.insert(key_arc, Arc::new(value));
    }

    /// Get parameter efficiently without allocation
    pub fn get_parameter(&self, key: &str) -> Option<&Arc<serde_json::Value>> {
        self.parameters
            .iter()
            .find(|(k, _)| k.as_ref() == key)
            .map(|(_, v)| v)
    }

    /// Add context efficiently using string interning
    pub fn add_context(&mut self, key: &str, value: serde_json::Value) {
        let key_arc = intern_universal_string(key);
        self.context.insert(key_arc, Arc::new(value));
    }
}

/// Universal response format with ``Arc<str>`` optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalResponse {
    /// Request ID as `Arc<str>` for efficient correlation
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub request_id: Arc<str>,

    /// Response status
    pub status: ResponseStatus,

    /// Response data
    pub data: serde_json::Value,

    /// Metadata with `Arc<str>` keys and Arc<serde_json::Value> values
    #[serde(
        serialize_with = "serialize_parameters",
        deserialize_with = "deserialize_parameters"
    )]
    pub metadata: HashMap<Arc<str>, Arc<serde_json::Value>>,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl UniversalResponse {
    /// Create a successful response with `Arc<str>` optimization
    pub fn success(request_id: Arc<str>, data: serde_json::Value) -> Self {
        Self {
            request_id,
            status: ResponseStatus::Success,
            data,
            metadata: HashMap::new(),
            processing_time_ms: 0,
            timestamp: Utc::now(),
        }
    }

    /// Create an error response with `Arc<str>` optimization
    pub fn error(request_id: Arc<str>, code: &str, message: &str) -> Self {
        Self {
            request_id,
            status: ResponseStatus::Error {
                code: Arc::from(code),
                message: Arc::from(message),
            },
            data: serde_json::Value::Null,
            metadata: HashMap::new(),
            processing_time_ms: 0,
            timestamp: Utc::now(),
        }
    }

    /// Add metadata efficiently using string interning
    pub fn add_metadata(&mut self, key: &str, value: serde_json::Value) {
        let key_arc = intern_universal_string(key);
        self.metadata.insert(key_arc, Arc::new(value));
    }
}

/// Response status with ``Arc<str>`` optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseStatus {
    Success,
    Error {
        #[serde(
            serialize_with = "serialize_arc_str",
            deserialize_with = "deserialize_arc_str"
        )]
        code: Arc<str>,
        #[serde(
            serialize_with = "serialize_arc_str",
            deserialize_with = "deserialize_arc_str"
        )]
        message: Arc<str>,
    },
    Partial {
        completed: usize,
        total: usize,
    },
}

/// Service health status with ``Arc<str>`` optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub healthy: bool,
    /// Health message as `Arc<str>`
    #[serde(
        serialize_with = "serialize_optional_arc_str",
        deserialize_with = "deserialize_optional_arc_str"
    )]
    pub message: Option<Arc<str>>,
    /// Metrics with `Arc<str>` keys and Arc<serde_json::Value> values
    #[serde(
        serialize_with = "serialize_parameters",
        deserialize_with = "deserialize_parameters"
    )]
    pub metrics: HashMap<Arc<str>, Arc<serde_json::Value>>,
}

impl ServiceHealth {
    /// Create healthy status
    pub fn healthy() -> Self {
        Self {
            healthy: true,
            message: None,
            metrics: HashMap::new(),
        }
    }

    /// Create unhealthy status with message
    pub fn unhealthy(message: &str) -> Self {
        Self {
            healthy: false,
            message: Some(Arc::from(message)),
            metrics: HashMap::new(),
        }
    }
}

// Serde helper functions for Arc<str> serialization
fn serialize_arc_str<S>(arc_str: &Arc<str>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(arc_str)
}

fn deserialize_arc_str<'de, D>(deserializer: D) -> Result<Arc<str>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Arc::from(s))
}

fn serialize_optional_arc_str<S>(opt: &Option<Arc<str>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match opt {
        Some(arc_str) => serializer.serialize_some(arc_str.as_ref()),
        None => serializer.serialize_none(),
    }
}

fn deserialize_optional_arc_str<'de, D>(deserializer: D) -> Result<Option<Arc<str>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt_string = Option::<String>::deserialize(deserializer)?;
    Ok(opt_string.map(Arc::from))
}

fn serialize_parameters<S>(
    map: &HashMap<Arc<str>, Arc<serde_json::Value>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let json_map: HashMap<&str, &serde_json::Value> =
        map.iter().map(|(k, v)| (k.as_ref(), v.as_ref())).collect();
    json_map.serialize(serializer)
}

fn deserialize_parameters<'de, D>(
    deserializer: D,
) -> Result<HashMap<Arc<str>, Arc<serde_json::Value>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let json_map = HashMap::<String, serde_json::Value>::deserialize(deserializer)?;
    Ok(json_map
        .into_iter()
        .map(|(k, v)| (intern_universal_string(&k), Arc::new(v)))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- intern_universal_string ----

    #[test]
    fn test_intern_known_operation() {
        let a = intern_universal_string("health_check");
        let b = intern_universal_string("health_check");
        // Pre-interned strings share the same Arc
        assert!(Arc::ptr_eq(&a, &b));
        assert_eq!(&*a, "health_check");
    }

    #[test]
    fn test_intern_unknown_string() {
        let s = intern_universal_string("something_totally_new");
        assert_eq!(&*s, "something_totally_new");
    }

    #[test]
    fn test_intern_all_predefined() {
        let known = [
            "health_check",
            "get_capabilities",
            "process_request",
            "get_status",
            "register_service",
            "discover_services",
            "ai_coordinator",
            "context_manager",
            "plugin_manager",
            "security_service",
            "metrics_collector",
            "success",
            "error",
            "pending",
            "partial",
            "timeout",
            "version",
            "timestamp",
            "processing_time",
            "request_id",
            "user_id",
            "session_id",
        ];
        for k in &known {
            let interned = intern_universal_string(k);
            assert_eq!(&*interned, *k, "Mismatch for key '{}'", k);
        }
    }

    // ---- UniversalRequest ----

    #[test]
    fn test_request_new() {
        let req = UniversalRequest::new("req-1", "health_check", "squirrel");
        assert_eq!(&*req.request_id, "req-1");
        assert_eq!(&*req.operation, "health_check");
        assert_eq!(&*req.requester, "squirrel");
        assert!(req.parameters.is_empty());
        assert!(req.context.is_empty());
    }

    #[test]
    fn test_request_add_and_get_parameter() {
        let mut req = UniversalRequest::new("req-2", "process_request", "client");
        req.add_parameter("key1", serde_json::json!("value1"));
        req.add_parameter("key2", serde_json::json!(42));

        let val = req.get_parameter("key1").expect("key1 should exist");
        assert_eq!(val.as_ref(), &serde_json::json!("value1"));

        let val = req.get_parameter("key2").expect("key2 should exist");
        assert_eq!(val.as_ref(), &serde_json::json!(42));

        assert!(req.get_parameter("nonexistent").is_none());
    }

    #[test]
    fn test_request_add_context() {
        let mut req = UniversalRequest::new("req-3", "get_status", "admin");
        req.add_context("trace_id", serde_json::json!("abc-123"));

        assert_eq!(req.context.len(), 1);
    }

    #[test]
    fn test_request_serde_roundtrip() {
        let mut req = UniversalRequest::new("req-rt", "health_check", "test-client");
        req.add_parameter("timeout", serde_json::json!(30));
        req.add_context("env", serde_json::json!("production"));

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: UniversalRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(&*deserialized.request_id, "req-rt");
        assert_eq!(&*deserialized.operation, "health_check");
        assert_eq!(&*deserialized.requester, "test-client");
        assert!(deserialized.get_parameter("timeout").is_some());
    }

    // ---- UniversalResponse ----

    #[test]
    fn test_response_success() {
        let resp =
            UniversalResponse::success(Arc::from("req-1"), serde_json::json!({"result": "ok"}));
        assert_eq!(&*resp.request_id, "req-1");
        assert!(matches!(resp.status, ResponseStatus::Success));
        assert_eq!(resp.data, serde_json::json!({"result": "ok"}));
        assert_eq!(resp.processing_time_ms, 0);
    }

    #[test]
    fn test_response_error() {
        let resp = UniversalResponse::error(Arc::from("req-2"), "NOT_FOUND", "Resource not found");
        assert_eq!(&*resp.request_id, "req-2");
        match &resp.status {
            ResponseStatus::Error { code, message } => {
                assert_eq!(&**code, "NOT_FOUND");
                assert_eq!(&**message, "Resource not found");
            }
            _ => panic!("Expected Error status"),
        }
        assert_eq!(resp.data, serde_json::Value::Null);
    }

    #[test]
    fn test_response_add_metadata() {
        let mut resp = UniversalResponse::success(Arc::from("req-3"), serde_json::json!(null));
        resp.add_metadata("version", serde_json::json!("1.0.0"));
        resp.add_metadata("processing_time", serde_json::json!(42));

        assert_eq!(resp.metadata.len(), 2);
    }

    #[test]
    fn test_response_serde_roundtrip() {
        let mut resp =
            UniversalResponse::success(Arc::from("req-rt"), serde_json::json!({"count": 5}));
        resp.add_metadata("version", serde_json::json!("2.0"));

        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: UniversalResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(&*deserialized.request_id, "req-rt");
        assert!(matches!(deserialized.status, ResponseStatus::Success));
    }

    #[test]
    fn test_error_response_serde_roundtrip() {
        let resp = UniversalResponse::error(Arc::from("req-err"), "500", "internal");
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: UniversalResponse = serde_json::from_str(&json).unwrap();

        match &deserialized.status {
            ResponseStatus::Error { code, message } => {
                assert_eq!(&**code, "500");
                assert_eq!(&**message, "internal");
            }
            _ => panic!("Expected Error status after deserialization"),
        }
    }

    // ---- ServiceHealth ----

    #[test]
    fn test_service_health_healthy() {
        let health = ServiceHealth::healthy();
        assert!(health.healthy);
        assert!(health.message.is_none());
        assert!(health.metrics.is_empty());
    }

    #[test]
    fn test_service_health_unhealthy() {
        let health = ServiceHealth::unhealthy("database connection lost");
        assert!(!health.healthy);
        assert_eq!(
            &**health.message.as_ref().unwrap(),
            "database connection lost"
        );
    }

    #[test]
    fn test_service_health_serde_roundtrip() {
        let health = ServiceHealth::unhealthy("out of memory");
        let json = serde_json::to_string(&health).unwrap();
        let deserialized: ServiceHealth = serde_json::from_str(&json).unwrap();

        assert!(!deserialized.healthy);
        assert_eq!(&**deserialized.message.as_ref().unwrap(), "out of memory");
    }

    #[test]
    fn test_service_health_healthy_serde_roundtrip() {
        let health = ServiceHealth::healthy();
        let json = serde_json::to_string(&health).unwrap();
        let deserialized: ServiceHealth = serde_json::from_str(&json).unwrap();

        assert!(deserialized.healthy);
        assert!(deserialized.message.is_none());
    }

    // ---- ResponseStatus ----

    #[test]
    fn test_response_status_partial() {
        let status = ResponseStatus::Partial {
            completed: 3,
            total: 10,
        };
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: ResponseStatus = serde_json::from_str(&json).unwrap();

        match deserialized {
            ResponseStatus::Partial { completed, total } => {
                assert_eq!(completed, 3);
                assert_eq!(total, 10);
            }
            _ => panic!("Expected Partial status"),
        }
    }
}
