// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors
#![expect(deprecated, reason = "Backward compatibility during migration")]
#![expect(dead_code, reason = "Registry types awaiting full ecosystem wiring")]

//! Core types for the ecosystem registry manager

// Backward compatibility: kept for deserialization of legacy data (EcosystemPrimalType in DiscoveredService, PrimalApiRequest)
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use crate::ecosystem::EcosystemPrimalType;
use universal_constants::primal_names;

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
    map.insert(primal_names::SONGBIRD, Arc::from(primal_names::SONGBIRD));
    map.insert(primal_names::TOADSTOOL, Arc::from(primal_names::TOADSTOOL));
    map.insert(primal_names::BEARDOG, Arc::from(primal_names::BEARDOG));
    map.insert(primal_names::NESTGATE, Arc::from(primal_names::NESTGATE));
    map.insert(primal_names::BIOMEOS, Arc::from(primal_names::BIOMEOS));

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
        primal_names::SQUIRREL => Arc::from(primal_names::SQUIRREL),
        // Legacy primal names: display/fallback only when deserializing external data.
        // NOT for discovery routing—use capability constants for that.
        primal_names::SONGBIRD => Arc::from(primal_names::SONGBIRD),
        primal_names::TOADSTOOL => Arc::from(primal_names::TOADSTOOL),
        primal_names::BEARDOG => Arc::from(primal_names::BEARDOG),
        primal_names::NESTGATE => Arc::from(primal_names::NESTGATE),
        primal_names::BIOMEOS => Arc::from(primal_names::BIOMEOS),
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
    /// Timestamp of last discovery sync
    pub last_discovery_sync: Option<DateTime<Utc>>,
    /// Number of registration attempts made
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
    /// Type of primal providing this service
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
    /// When the service was first discovered.
    pub discovered_at: DateTime<Utc>,
    /// Timestamp of the last health check, if performed.
    pub last_health_check: Option<DateTime<Utc>>,
    /// Current health status of the service.
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

// Additional serde helper functions (serde passes &Option<T> for serialize_with)
#[expect(clippy::ref_option, reason = "Optional reference; API design")]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceHealthStatus {
    /// Health status not yet determined
    Unknown,
    /// Service is operating normally
    Healthy,
    /// Service is degraded but functional
    Degraded,
    /// Service is unhealthy
    Unhealthy,
    /// Service is offline
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
    /// Source primal type
    pub from_primal: EcosystemPrimalType,
    /// Target primal type
    pub to_primal: EcosystemPrimalType,
    /// Operation name as `Arc<str>` with string interning
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub operation: Arc<str>,
    /// Request payload
    pub payload: serde_json::Value,
    /// Headers with `Arc<str>` keys and values for zero-copy
    #[serde(
        serialize_with = "serialize_arc_str_map",
        deserialize_with = "deserialize_arc_str_map"
    )]
    pub headers: HashMap<Arc<str>, Arc<str>>,
    /// Request timeout
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
    /// Whether the request succeeded
    pub success: bool,
    /// Response data when successful
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
    /// Time taken to process the request
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
        /// Discovered service ID
        service_id: Arc<str>,
        /// Type of primal
        primal_type: crate::EcosystemPrimalType,
        /// Service endpoint
        endpoint: Arc<str>,
        /// Service capabilities
        capabilities: Vec<Arc<str>>,
    },

    /// Service registered with ecosystem
    ServiceRegistered {
        /// Registered service ID
        service_id: Arc<str>,
        /// Type of primal
        primal_type: crate::EcosystemPrimalType,
        /// Service endpoint
        endpoint: Arc<str>,
    },

    /// Service error occurred
    ServiceError {
        /// Service ID where error occurred
        service_id: Arc<str>,
        /// Error message
        error: Arc<str>,
        /// When the error occurred
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Service health status changed
    ServiceHealthChanged {
        /// Service ID
        service_id: Arc<str>,
        /// Type of primal
        primal_type: EcosystemPrimalType,
        /// Previous health status
        old_status: ServiceHealthStatus,
        /// New health status
        new_status: ServiceHealthStatus,
    },
    /// Service went offline
    ServiceOffline {
        /// Service ID
        service_id: Arc<str>,
        /// Type of primal
        primal_type: EcosystemPrimalType,
        /// Reason for going offline
        reason: Arc<str>,
    },
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Health status determined by the check
    pub status: ServiceHealthStatus,
    /// Time taken to perform the check
    pub processing_time: Duration,
    /// Error message if check failed
    pub error: Option<String>,
}

/// Ecosystem status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemStatus {
    /// Overall health score (0.0 to 1.0)
    pub overall_health: f64,
    /// Status of each primal
    pub primal_statuses: Vec<PrimalStatus>,
    /// Number of registered services
    pub registered_services: usize,
    /// Number of active coordinations
    pub active_coordinations: usize,
    /// Timestamp of last full sync
    pub last_full_sync: Option<DateTime<Utc>>,
    /// Size of discovery cache
    pub discovery_cache_size: usize,
}

/// Primal status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalStatus {
    /// Type of primal
    pub primal_type: EcosystemPrimalType,
    /// Current service status
    pub status: ServiceStatus,
    /// Service endpoint URL
    pub endpoint: String,
    /// Service version
    pub version: String,
    /// Capability identifiers
    pub capabilities: Vec<String>,
    /// Health score (0.0 to 1.0)
    pub health_score: f64,
    /// Average response time
    pub response_time: Duration,
    /// When the primal was last seen
    pub last_seen: DateTime<Utc>,
    /// Number of recent errors
    pub error_count: u32,
    /// Coordination features supported
    pub coordination_features: Vec<String>,
}

/// Service status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceStatus {
    /// Status not yet determined
    Unknown,
    /// Currently discovering services
    Discovering,
    /// Currently registering
    Registering,
    /// Operating normally
    Healthy,
    /// Degraded but functional
    Degraded,
    /// Unhealthy
    Unhealthy,
    /// Offline
    Offline,
    /// Recovering from failure
    Recovering,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
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

    #[test]
    fn intern_registry_string_covers_capability_and_fallback_branches() {
        use universal_constants::capabilities;

        assert_eq!(intern_registry_string("storage").as_ref(), "storage");
        assert_eq!(intern_registry_string("compute").as_ref(), "compute");
        assert_eq!(intern_registry_string("security").as_ref(), "security");
        assert_eq!(intern_registry_string("discovery").as_ref(), "discovery");
        assert_eq!(intern_registry_string("active").as_ref(), "active");
        assert_eq!(intern_registry_string("inactive").as_ref(), "inactive");
        assert_eq!(intern_registry_string("error").as_ref(), "error");
        assert_eq!(
            intern_registry_string(capabilities::COMPUTE_CAPABILITY).as_ref(),
            capabilities::COMPUTE_CAPABILITY
        );
        let custom = intern_registry_string("unique-custom-id-xyz");
        assert_eq!(custom.as_ref(), "unique-custom-id-xyz");
    }

    #[test]
    fn discovered_service_serde_roundtrip() {
        let s = DiscoveredService::new(
            "svc-serde",
            EcosystemPrimalType::Squirrel,
            "unix:///tmp/x.sock",
            "unix:///tmp/x.sock",
            "2.0",
            vec!["storage"],
            HashMap::from([("region", "eu")]),
        );
        let json = serde_json::to_string(&s).expect("ser");
        let back: DiscoveredService = serde_json::from_str(&json).expect("de");
        assert_eq!(back.service_id.as_ref(), "svc-serde");
        assert_eq!(
            back.get_metadata("region").map(std::convert::AsRef::as_ref),
            Some("eu")
        );
    }

    #[test]
    fn primal_api_response_with_error_roundtrip() {
        let rid = Arc::from("r1");
        let resp = PrimalApiResponse::new(
            Arc::clone(&rid),
            false,
            None,
            Some("failed"),
            HashMap::from([("x-trace", "t1")]),
            Duration::from_millis(1),
        );
        let json = serde_json::to_string(&resp).expect("ser");
        let back: PrimalApiResponse = serde_json::from_str(&json).expect("de");
        assert!(!back.success);
        assert_eq!(
            back.error.as_ref().map(std::convert::AsRef::as_ref),
            Some("failed")
        );
    }

    #[test]
    fn ecosystem_status_serde_roundtrip() {
        let st = EcosystemStatus {
            overall_health: 0.75,
            primal_statuses: vec![],
            registered_services: 2,
            active_coordinations: 1,
            last_full_sync: Some(Utc::now()),
            discovery_cache_size: 3,
        };
        let v = serde_json::to_value(&st).expect("should succeed");
        let back: EcosystemStatus = serde_json::from_value(v).expect("should succeed");
        assert!((back.overall_health - 0.75).abs() < f64::EPSILON);
        assert_eq!(back.discovery_cache_size, 3);
    }

    #[test]
    fn registry_state_default_is_empty() {
        let rs = RegistryState::default();
        assert!(rs.registered_services.is_empty());
        assert!(rs.service_discovery_cache.is_empty());
        assert_eq!(rs.registration_attempts, 0);
    }

    #[test]
    fn ecosystem_registry_event_debug_smoke() {
        let ev = EcosystemRegistryEvent::ServiceDiscovered {
            service_id: Arc::from("s1"),
            primal_type: EcosystemPrimalType::Squirrel,
            endpoint: Arc::from("unix:///a"),
            capabilities: vec![Arc::from("compute")],
        };
        let s = format!("{ev:?}");
        assert!(s.contains("ServiceDiscovered"));
    }

    #[test]
    fn service_status_all_variants_serde() {
        for st in [
            ServiceStatus::Discovering,
            ServiceStatus::Registering,
            ServiceStatus::Recovering,
            ServiceStatus::Offline,
            ServiceStatus::Unhealthy,
        ] {
            let j = serde_json::to_string(&st).expect("should succeed");
            let _: ServiceStatus = serde_json::from_str(&j).expect("should succeed");
        }
    }

    #[test]
    fn intern_registry_string_additional_branches() {
        assert_eq!(
            intern_registry_string("ai_coordination").as_ref(),
            "ai_coordination"
        );
        assert_eq!(intern_registry_string("network").as_ref(), "network");
        assert_eq!(intern_registry_string("songbird").as_ref(), "songbird");
        assert_eq!(intern_registry_string("nestgate").as_ref(), "nestgate");
    }

    #[test]
    fn ecosystem_registry_event_variants_debug() {
        let ev = EcosystemRegistryEvent::ServiceRegistered {
            service_id: Arc::from("s1"),
            primal_type: EcosystemPrimalType::Songbird,
            endpoint: Arc::from("unix:///a"),
        };
        assert!(format!("{ev:?}").contains("ServiceRegistered"));
        let ev2 = EcosystemRegistryEvent::ServiceError {
            service_id: Arc::from("s2"),
            error: Arc::from("e"),
            timestamp: Utc::now(),
        };
        assert!(format!("{ev2:?}").contains("ServiceError"));
        let ev3 = EcosystemRegistryEvent::ServiceHealthChanged {
            service_id: Arc::from("s3"),
            primal_type: EcosystemPrimalType::Squirrel,
            old_status: ServiceHealthStatus::Unknown,
            new_status: ServiceHealthStatus::Healthy,
        };
        assert!(format!("{ev3:?}").contains("ServiceHealthChanged"));
        let ev4 = EcosystemRegistryEvent::ServiceOffline {
            service_id: Arc::from("s4"),
            primal_type: EcosystemPrimalType::BearDog,
            reason: Arc::from("shutdown"),
        };
        assert!(format!("{ev4:?}").contains("ServiceOffline"));
    }

    #[test]
    fn health_check_result_debug() {
        let h = HealthCheckResult {
            status: ServiceHealthStatus::Degraded,
            processing_time: Duration::from_millis(5),
            error: Some("x".to_string()),
        };
        let s = format!("{h:?}");
        assert!(s.contains("Degraded"));
    }

    #[test]
    fn primal_status_roundtrip() {
        let ps = PrimalStatus {
            primal_type: EcosystemPrimalType::Squirrel,
            status: ServiceStatus::Healthy,
            endpoint: "e".to_string(),
            version: "1".to_string(),
            capabilities: vec!["c".to_string()],
            health_score: 0.9,
            response_time: Duration::from_millis(1),
            last_seen: Utc::now(),
            error_count: 0,
            coordination_features: vec![],
        };
        let j = serde_json::to_value(&ps).expect("should succeed");
        let back: PrimalStatus = serde_json::from_value(j).expect("should succeed");
        assert_eq!(back.endpoint, "e");
    }
}
