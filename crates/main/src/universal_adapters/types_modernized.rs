//! Modernized Universal Adapter Types with Arc<str> Optimization
//!
//! This module provides modernized universal adapter types that use Arc<str>
//! for dramatic performance improvements in request/response handling.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;

/// String interning for common universal adapter values
lazy_static! {
    static ref UNIVERSAL_STRINGS: HashMap<&'static str, Arc<str>> = {
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
    };
}

/// Get Arc<str> for universal adapter string with zero allocation for common values
pub fn intern_universal_string(s: &str) -> Arc<str> {
    UNIVERSAL_STRINGS.get(s)
        .cloned()
        .unwrap_or_else(|| Arc::from(s))
}

/// Universal request format with Arc<str> optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalRequest {
    /// Request ID as Arc<str> for efficient sharing
    #[serde(serialize_with = "serialize_arc_str", deserialize_with = "deserialize_arc_str")]
    pub request_id: Arc<str>,
    
    /// Operation name with string interning
    #[serde(serialize_with = "serialize_arc_str", deserialize_with = "deserialize_arc_str")]
    pub operation: Arc<str>,
    
    /// Parameters with Arc<str> keys and Arc<serde_json::Value> values
    #[serde(serialize_with = "serialize_parameters", deserialize_with = "deserialize_parameters")]
    pub parameters: HashMap<Arc<str>, Arc<serde_json::Value>>,
    
    /// Context with Arc<str> keys and Arc<serde_json::Value> values
    #[serde(serialize_with = "serialize_parameters", deserialize_with = "deserialize_parameters")]
    pub context: HashMap<Arc<str>, Arc<serde_json::Value>>,
    
    /// Requester as Arc<str>
    #[serde(serialize_with = "serialize_arc_str", deserialize_with = "deserialize_arc_str")]
    pub requester: Arc<str>,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl UniversalRequest {
    /// Create new UniversalRequest with string interning optimization
    pub fn new(
        request_id: &str,
        operation: &str,
        requester: &str,
    ) -> Self {
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
        self.parameters.iter()
            .find(|(k, _)| k.as_ref() == key)
            .map(|(_, v)| v)
    }

    /// Add context efficiently using string interning
    pub fn add_context(&mut self, key: &str, value: serde_json::Value) {
        let key_arc = intern_universal_string(key);
        self.context.insert(key_arc, Arc::new(value));
    }
}

/// Universal response format with Arc<str> optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalResponse {
    /// Request ID as Arc<str> for efficient correlation
    #[serde(serialize_with = "serialize_arc_str", deserialize_with = "deserialize_arc_str")]
    pub request_id: Arc<str>,
    
    /// Response status
    pub status: ResponseStatus,
    
    /// Response data
    pub data: serde_json::Value,
    
    /// Metadata with Arc<str> keys and Arc<serde_json::Value> values
    #[serde(serialize_with = "serialize_parameters", deserialize_with = "deserialize_parameters")]
    pub metadata: HashMap<Arc<str>, Arc<serde_json::Value>>,
    
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl UniversalResponse {
    /// Create a successful response with Arc<str> optimization
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

    /// Create an error response with Arc<str> optimization
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

/// Response status with Arc<str> optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseStatus {
    Success,
    Error { 
        #[serde(serialize_with = "serialize_arc_str", deserialize_with = "deserialize_arc_str")]
        code: Arc<str>, 
        #[serde(serialize_with = "serialize_arc_str", deserialize_with = "deserialize_arc_str")]
        message: Arc<str> 
    },
    Partial { completed: usize, total: usize },
}

/// Service health status with Arc<str> optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub healthy: bool,
    /// Health message as Arc<str>
    #[serde(serialize_with = "serialize_optional_arc_str", deserialize_with = "deserialize_optional_arc_str")]
    pub message: Option<Arc<str>>,
    /// Metrics with Arc<str> keys and Arc<serde_json::Value> values
    #[serde(serialize_with = "serialize_parameters", deserialize_with = "deserialize_parameters")]
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
    Ok(opt_string.map(|s| Arc::from(s)))
}

fn serialize_parameters<S>(map: &HashMap<Arc<str>, Arc<serde_json::Value>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let json_map: HashMap<&str, &serde_json::Value> = map.iter()
        .map(|(k, v)| (k.as_ref(), v.as_ref()))
        .collect();
    json_map.serialize(serializer)
}

fn deserialize_parameters<'de, D>(deserializer: D) -> Result<HashMap<Arc<str>, Arc<serde_json::Value>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let json_map = HashMap::<String, serde_json::Value>::deserialize(deserializer)?;
    Ok(json_map.into_iter()
        .map(|(k, v)| (intern_universal_string(&k), Arc::new(v)))
        .collect())
} 