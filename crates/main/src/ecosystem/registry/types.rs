// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors
#![allow(deprecated)]
#![allow(dead_code)] // Registry types awaiting full ecosystem wiring

//! Core types for the ecosystem registry manager

// Backward compatibility: kept for deserialization of legacy data (EcosystemPrimalType in DiscoveredService, PrimalApiRequest)
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use crate::ecosystem::EcosystemPrimalType;

/// String interning for common service registry values
/// Uses capability constants for discovery; legacy primal names for backward compatibility.
///
/// **TRUE PRIMAL**: Discovery uses capability names (storage, compute, security, etc.).
/// Legacy primal names below are ONLY for display/fallback identifiers when deserializing
/// external data - NOT for discovery routing. Actual discovery is capability-based.
static REGISTRY_STRINGS: LazyLock<HashMap<&'static str, Arc<str>>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    use universal_constants::capabilities;
    // Capability-based (preferred for discovery)
    map.insert(
        capabilities::SELF_PRIMAL_NAME,
        Arc::from(capabilities::SELF_PRIMAL_NAME),
    );
    map.insert(
        capabilities::SERVICE_MESH_CAPABILITY,
        Arc::from(capabilities::SERVICE_MESH_CAPABILITY),
    );
    map.insert(
        capabilities::COMPUTE_CAPABILITY,
        Arc::from(capabilities::COMPUTE_CAPABILITY),
    );
    map.insert(
        capabilities::SECURITY_CAPABILITY,
        Arc::from(capabilities::SECURITY_CAPABILITY),
    );
    map.insert(
        capabilities::STORAGE_CAPABILITY,
        Arc::from(capabilities::STORAGE_CAPABILITY),
    );
    map.insert(
        capabilities::ECOSYSTEM_CAPABILITY,
        Arc::from(capabilities::ECOSYSTEM_CAPABILITY),
    );
    // Legacy primal names: display/fallback only, NOT for discovery routing.
    // Used when deserializing config or external responses that reference primal IDs.
    map.insert("songbird", Arc::from("songbird"));
    map.insert("toadstool", Arc::from("toadstool"));
    map.insert("beardog", Arc::from("beardog"));
    map.insert("nestgate", Arc::from("nestgate"));
    map.insert("biomeos", Arc::from("biomeos"));

    // Common capabilities
    map.insert("ai_coordination", Arc::from("ai_coordination"));
    map.insert("service_mesh", Arc::from("service_mesh"));
    map.insert("security", Arc::from("security"));
    map.insert("monitoring", Arc::from("monitoring"));
    map.insert("discovery", Arc::from("discovery"));
    map.insert("orchestration", Arc::from("orchestration"));
    map.insert("intelligence", Arc::from("intelligence"));
    map.insert("biome_integration", Arc::from("biome_integration"));

    // Common metadata keys
    map.insert("version", Arc::from("version"));
    map.insert("environment", Arc::from("environment"));
    map.insert("region", Arc::from("region"));
    map.insert("instance_id", Arc::from("instance_id"));
    map.insert("last_updated", Arc::from("last_updated"));
    map.insert("health_endpoint", Arc::from("health_endpoint"));
    map.insert("metrics_endpoint", Arc::from("metrics_endpoint"));

    // Common operation names
    map.insert("register", Arc::from("register"));
    map.insert("discover", Arc::from("discover"));
    map.insert("health_check", Arc::from("health_check"));
    map.insert("metrics", Arc::from("metrics"));

    map
});

/// Get ```Arc<str>``` for registry string with zero allocation for common values
///
/// **TRUE PRIMAL**: Capability constants resolve to capability names for discovery.
/// Squirrel knows its own name ("squirrel"); other primal names are display/fallback only.
/// Discovery routing uses capability names, not primal hostnames.
#[must_use]
pub fn intern_registry_string(s: &str) -> Arc<str> {
    use universal_constants::capabilities;
    // Literal capability names first - must preserve for DiscoveredService.capabilities
    match s {
        "storage" => Arc::from("storage"),
        "compute" => Arc::from("compute"),
        "security" => Arc::from("security"),
        "discovery" => Arc::from("discovery"),
        "ai_coordination" => Arc::from("ai_coordination"),
        // Squirrel can know its own name (self-knowledge)
        "squirrel" => Arc::from("squirrel"),
        // Legacy primal names: display/fallback only when deserializing external data.
        // NOT for discovery routing—use capability constants for that.
        "songbird" => Arc::from("songbird"),
        "toadstool" => Arc::from("toadstool"),
        "beardog" => Arc::from("beardog"),
        "nestgate" => Arc::from("nestgate"),
        "biomeos" => Arc::from("biomeos"),
        // Capability constants -> capability names (for discovery, NOT primal names)
        n if n == capabilities::SELF_PRIMAL_NAME => Arc::from("squirrel"),
        n if n == capabilities::SERVICE_MESH_CAPABILITY => {
            Arc::from(capabilities::SERVICE_MESH_CAPABILITY)
        }
        n if n == capabilities::COMPUTE_CAPABILITY => Arc::from(capabilities::COMPUTE_CAPABILITY),
        n if n == capabilities::SECURITY_CAPABILITY => Arc::from(capabilities::SECURITY_CAPABILITY),
        n if n == capabilities::STORAGE_CAPABILITY => Arc::from(capabilities::STORAGE_CAPABILITY),
        n if n == capabilities::ECOSYSTEM_CAPABILITY => {
            Arc::from(capabilities::ECOSYSTEM_CAPABILITY)
        }
        "active" => Arc::from("active"),
        "inactive" => Arc::from("inactive"),
        "error" => Arc::from("error"),
        "network" => Arc::from("network"),
        _ => Arc::from(s), // Allocate for uncommon strings
    }
}

/// Registry state tracking with ``Arc<str>`` optimization
#[derive(Debug, Default)]
pub struct RegistryState {
    /// Service registrations with `Arc<str>` keys for zero-copy performance
    pub registered_services: HashMap<Arc<str>, Arc<crate::ecosystem::EcosystemServiceRegistration>>,
    /// Service discovery cache with `Arc<str>` keys and `Arc<DiscoveredService>` values
    pub service_discovery_cache: HashMap<Arc<str>, Arc<DiscoveredService>>,
    pub last_discovery_sync: Option<DateTime<Utc>>,
    pub registration_attempts: u32,
}

/// Discovered service information with `Arc<str>` optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredService {
    /// Service ID as `Arc<str>` for efficient sharing
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub service_id: Arc<str>,
    pub primal_type: EcosystemPrimalType,
    /// Endpoint as `Arc<str>` for efficient sharing
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub endpoint: Arc<str>,
    /// Health endpoint as `Arc<str>`
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub health_endpoint: Arc<str>,
    /// API version as `Arc<str>`
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub api_version: Arc<str>,
    /// Capabilities as `Arc<str>` for efficient sharing
    #[serde(
        serialize_with = "serialize_arc_str_vec",
        deserialize_with = "deserialize_arc_str_vec"
    )]
    pub capabilities: Vec<Arc<str>>,
    /// Metadata with `Arc<str>` keys and values for zero-copy
    #[serde(
        serialize_with = "serialize_arc_str_map",
        deserialize_with = "deserialize_arc_str_map"
    )]
    pub metadata: HashMap<Arc<str>, Arc<str>>,
    pub discovered_at: DateTime<Utc>,
    pub last_health_check: Option<DateTime<Utc>>,
    pub health_status: ServiceHealthStatus,
}

impl DiscoveredService {
    /// Create new `DiscoveredService` with string interning optimization
    pub fn new(
        service_id: &str,
        primal_type: EcosystemPrimalType,
        endpoint: &str,
        health_endpoint: &str,
        api_version: &str,
        capabilities: Vec<&str>,
        metadata: HashMap<&str, &str>,
    ) -> Self {
        Self {
            service_id: intern_registry_string(service_id),
            primal_type,
            endpoint: Arc::from(endpoint),
            health_endpoint: Arc::from(health_endpoint),
            api_version: intern_registry_string(api_version),
            capabilities: capabilities
                .into_iter()
                .map(intern_registry_string)
                .collect(),
            metadata: metadata
                .into_iter()
                .map(|(k, v)| (intern_registry_string(k), Arc::from(v)))
                .collect(),
            discovered_at: Utc::now(),
            last_health_check: None,
            health_status: ServiceHealthStatus::Unknown,
        }
    }

    /// Efficient lookup of metadata without allocation
    #[must_use]
    pub fn get_metadata(&self, key: &str) -> Option<&Arc<str>> {
        self.metadata
            .iter()
            .find(|(k, _)| k.as_ref() == key)
            .map(|(_, v)| v)
    }

    /// Check if service has capability without allocation
    #[must_use]
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities
            .iter()
            .any(|cap| cap.as_ref() == capability)
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

fn serialize_arc_str_vec<S>(vec: &[Arc<str>], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let strings: Vec<&str> = vec.iter().map(std::convert::AsRef::as_ref).collect();
    strings.serialize(serializer)
}

fn deserialize_arc_str_vec<'de, D>(deserializer: D) -> Result<Vec<Arc<str>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let strings = Vec::<String>::deserialize(deserializer)?;
    Ok(strings.into_iter().map(Arc::from).collect())
}

fn serialize_arc_str_map<S>(
    map: &HashMap<Arc<str>, Arc<str>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let string_map: HashMap<&str, &str> =
        map.iter().map(|(k, v)| (k.as_ref(), v.as_ref())).collect();
    string_map.serialize(serializer)
}

fn deserialize_arc_str_map<'de, D>(deserializer: D) -> Result<HashMap<Arc<str>, Arc<str>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let string_map = HashMap::<String, String>::deserialize(deserializer)?;
    Ok(string_map
        .into_iter()
        .map(|(k, v)| (Arc::from(k), Arc::from(v)))
        .collect())
}

// Additional serde helper functions
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

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceHealthStatus {
    Unknown,
    Healthy,
    Degraded,
    Unhealthy,
    Offline,
}

/// Standard API request for inter-primal communication with ``Arc<str>`` optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalApiRequest {
    /// Request ID as `Arc<str>` for efficient sharing across async boundaries
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub request_id: Arc<str>,
    pub from_primal: EcosystemPrimalType,
    pub to_primal: EcosystemPrimalType,
    /// Operation name as `Arc<str>` with string interning
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub operation: Arc<str>,
    pub payload: serde_json::Value,
    /// Headers with `Arc<str>` keys and values for zero-copy
    #[serde(
        serialize_with = "serialize_arc_str_map",
        deserialize_with = "deserialize_arc_str_map"
    )]
    pub headers: HashMap<Arc<str>, Arc<str>>,
    pub timeout: Duration,
}

impl PrimalApiRequest {
    /// Create new `PrimalApiRequest` with string interning optimization
    #[must_use]
    pub fn new(
        request_id: &str,
        from_primal: EcosystemPrimalType,
        to_primal: EcosystemPrimalType,
        operation: &str,
        payload: serde_json::Value,
        headers: HashMap<&str, &str>,
        timeout: Duration,
    ) -> Self {
        Self {
            request_id: Arc::from(request_id),
            from_primal,
            to_primal,
            operation: intern_registry_string(operation),
            payload,
            headers: headers
                .into_iter()
                .map(|(k, v)| (intern_registry_string(k), Arc::from(v)))
                .collect(),
            timeout,
        }
    }

    /// Efficient header lookup without allocation
    #[must_use]
    pub fn get_header(&self, key: &str) -> Option<&Arc<str>> {
        self.headers
            .iter()
            .find(|(k, _)| k.as_ref() == key)
            .map(|(_, v)| v)
    }
}

/// Standard API response for inter-primal communication with ``Arc<str>`` optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalApiResponse {
    /// Request ID as `Arc<str>` for efficient correlation
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub request_id: Arc<str>,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    /// Error message as `Arc<str>` for efficient sharing
    #[serde(
        serialize_with = "serialize_optional_arc_str",
        deserialize_with = "deserialize_optional_arc_str"
    )]
    pub error: Option<Arc<str>>,
    /// Headers with `Arc<str>` keys and values for zero-copy
    #[serde(
        serialize_with = "serialize_arc_str_map",
        deserialize_with = "deserialize_arc_str_map"
    )]
    pub headers: HashMap<Arc<str>, Arc<str>>,
    pub processing_time: Duration,
}

impl PrimalApiResponse {
    /// Create new `PrimalApiResponse` with string optimization
    pub fn new(
        request_id: Arc<str>,
        success: bool,
        data: Option<serde_json::Value>,
        error: Option<&str>,
        headers: HashMap<&str, &str>,
        processing_time: Duration,
    ) -> Self {
        Self {
            request_id,
            success,
            data,
            error: error.map(Arc::from),
            headers: headers
                .into_iter()
                .map(|(k, v)| (intern_registry_string(k), Arc::from(v)))
                .collect(),
            processing_time,
        }
    }
}

/// Ecosystem registry events with ``Arc<str>`` optimization for efficient event sharing
#[derive(Debug, Clone)]
pub enum EcosystemRegistryEvent {
    /// Service discovered in the ecosystem
    ServiceDiscovered {
        service_id: Arc<str>,
        primal_type: crate::EcosystemPrimalType,
        endpoint: Arc<str>,
        capabilities: Vec<Arc<str>>, // Add the missing capabilities field
    },

    /// Service registered with ecosystem
    ServiceRegistered {
        service_id: Arc<str>,
        primal_type: crate::EcosystemPrimalType,
        endpoint: Arc<str>,
    },

    /// Service error occurred
    ServiceError {
        service_id: Arc<str>,
        error: Arc<str>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    ServiceHealthChanged {
        service_id: Arc<str>,
        primal_type: EcosystemPrimalType,
        old_status: ServiceHealthStatus,
        new_status: ServiceHealthStatus,
    },
    ServiceOffline {
        service_id: Arc<str>,
        primal_type: EcosystemPrimalType,
        reason: Arc<str>,
    },
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub status: ServiceHealthStatus,
    pub processing_time: Duration,
    pub error: Option<String>,
}

/// Ecosystem status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemStatus {
    pub overall_health: f64,
    pub primal_statuses: Vec<PrimalStatus>,
    pub registered_services: usize,
    pub active_coordinations: usize,
    pub last_full_sync: Option<DateTime<Utc>>,
    pub discovery_cache_size: usize,
}

/// Primal status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalStatus {
    pub primal_type: EcosystemPrimalType,
    pub status: ServiceStatus,
    pub endpoint: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub health_score: f64,
    pub response_time: Duration,
    pub last_seen: DateTime<Utc>,
    pub error_count: u32,
    pub coordination_features: Vec<String>,
}

/// Service status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceStatus {
    Unknown,
    Discovering,
    Registering,
    Healthy,
    Degraded,
    Unhealthy,
    Offline,
    Recovering,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_discovered_service_new() {
        let service = DiscoveredService::new(
            "svc-123",
            EcosystemPrimalType::Squirrel,
            "unix:///tmp/svc.sock",
            "unix:///tmp/svc.sock",
            "1.0",
            vec!["storage", "compute"],
            HashMap::new(),
        );

        assert_eq!(service.service_id.as_ref(), "svc-123");
        assert!(service.has_capability("storage"));
        assert!(service.has_capability("compute"));
        assert!(!service.has_capability("unknown"));
    }

    #[test]
    fn test_discovered_service_get_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("version", "1.0");
        metadata.insert("region", "us-west");

        let service = DiscoveredService::new(
            "test-svc",
            EcosystemPrimalType::Songbird,
            "http://localhost:8080",
            "http://localhost:8080/health",
            "1.0",
            vec!["service_mesh"],
            metadata,
        );

        assert!(service.get_metadata("version").is_some());
        assert!(service.get_metadata("region").is_some());
        assert!(service.get_metadata("missing").is_none());
    }

    #[test]
    fn test_primal_api_request_new() {
        let request = PrimalApiRequest::new(
            "req-123",
            EcosystemPrimalType::Squirrel,
            EcosystemPrimalType::Songbird,
            "discover",
            serde_json::json!({"capability": "storage"}),
            HashMap::new(),
            Duration::from_secs(30),
        );

        assert_eq!(request.request_id.as_ref(), "req-123");
        assert_eq!(request.operation.as_ref(), "discover");
    }

    #[test]
    fn test_primal_api_request_get_header() {
        let mut headers = HashMap::new();
        headers.insert("x-correlation-id", "corr-123");

        let request = PrimalApiRequest::new(
            "req-1",
            EcosystemPrimalType::Squirrel,
            EcosystemPrimalType::BearDog,
            "auth",
            serde_json::json!({}),
            headers,
            Duration::from_secs(5),
        );

        assert!(request.get_header("x-correlation-id").is_some());
        assert!(request.get_header("missing").is_none());
    }

    #[test]
    fn test_primal_api_response_new() {
        let request_id = Arc::from("req-123");
        let response = PrimalApiResponse::new(
            request_id,
            true,
            Some(serde_json::json!({"result": "ok"})),
            None,
            HashMap::new(),
            Duration::from_millis(50),
        );

        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_intern_registry_string() {
        let s = intern_registry_string("storage");
        assert_eq!(s.as_ref(), "storage");

        let s2 = intern_registry_string("squirrel");
        assert_eq!(s2.as_ref(), "squirrel");
    }

    #[test]
    fn test_service_health_status_variants() {
        let _ = ServiceHealthStatus::Unknown;
        let _ = ServiceHealthStatus::Healthy;
        let _ = ServiceHealthStatus::Degraded;
        let _ = ServiceHealthStatus::Unhealthy;
        let _ = ServiceHealthStatus::Offline;
    }

    #[test]
    fn test_service_status_variants() {
        let _ = ServiceStatus::Unknown;
        let _ = ServiceStatus::Healthy;
        let _ = ServiceStatus::Degraded;
    }
}
