//! Core types for the ecosystem registry manager

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use uuid::Uuid;

use crate::ecosystem::EcosystemPrimalType;

/// String interning for common service registry values
lazy_static! {
    static ref REGISTRY_STRINGS: HashMap<&'static str, Arc<str>> = {
        let mut map = HashMap::new();
        // Common service IDs and types
        map.insert("squirrel", Arc::from("squirrel"));
        map.insert("songbird", Arc::from("songbird"));
        map.insert("toadstool", Arc::from("toadstool"));
        map.insert("beardog", Arc::from("beardog"));
        map.insert("ecosystem", Arc::from("ecosystem"));

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
    };
}

/// Get Arc<str> for registry string with zero allocation for common values
pub fn intern_registry_string(s: &str) -> Arc<str> {
    // Common registry strings for zero-allocation lookups
    match s {
        "squirrel" => Arc::from("squirrel"),
        "songbird" => Arc::from("songbird"),
        "toadstool" => Arc::from("toadstool"),
        "beardog" => Arc::from("beardog"),
        "active" => Arc::from("active"),
        "inactive" => Arc::from("inactive"),
        "error" => Arc::from("error"),
        "discovery" => Arc::from("discovery"),
        "ai_coordination" => Arc::from("ai_coordination"),
        "storage" => Arc::from("storage"),
        "compute" => Arc::from("compute"),
        "security" => Arc::from("security"),
        _ => Arc::from(s), // Allocate for uncommon strings
    }
}

/// Registry state tracking with Arc<str> optimization
#[derive(Debug, Default)]
pub struct RegistryState {
    /// Service registrations with Arc<str> keys for zero-copy performance
    pub registered_services: HashMap<Arc<str>, Arc<crate::ecosystem::EcosystemServiceRegistration>>,
    /// Service discovery cache with Arc<str> keys and Arc<DiscoveredService> values
    pub service_discovery_cache: HashMap<Arc<str>, Arc<DiscoveredService>>,
    pub last_discovery_sync: Option<DateTime<Utc>>,
    pub registration_attempts: u32,
}

/// Discovered service information with Arc<str> optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredService {
    /// Service ID as Arc<str> for efficient sharing
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub service_id: Arc<str>,
    pub primal_type: EcosystemPrimalType,
    /// Endpoint as Arc<str> for efficient sharing
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub endpoint: Arc<str>,
    /// Health endpoint as Arc<str>
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub health_endpoint: Arc<str>,
    /// API version as Arc<str>
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub api_version: Arc<str>,
    /// Capabilities as Arc<str> for efficient sharing
    #[serde(
        serialize_with = "serialize_arc_str_vec",
        deserialize_with = "deserialize_arc_str_vec"
    )]
    pub capabilities: Vec<Arc<str>>,
    /// Metadata with Arc<str> keys and values for zero-copy
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
    /// Create new DiscoveredService with string interning optimization
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
                .map(|cap| intern_registry_string(cap))
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
    pub fn get_metadata(&self, key: &str) -> Option<&Arc<str>> {
        self.metadata
            .iter()
            .find(|(k, _)| k.as_ref() == key)
            .map(|(_, v)| v)
    }

    /// Check if service has capability without allocation
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

fn serialize_arc_str_vec<S>(vec: &Vec<Arc<str>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let strings: Vec<&str> = vec.iter().map(|arc| arc.as_ref()).collect();
    strings.serialize(serializer)
}

fn deserialize_arc_str_vec<'de, D>(deserializer: D) -> Result<Vec<Arc<str>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let strings = Vec::<String>::deserialize(deserializer)?;
    Ok(strings.into_iter().map(|s| Arc::from(s)).collect())
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
    Ok(opt_string.map(|s| Arc::from(s)))
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

/// Standard API request for inter-primal communication with Arc<str> optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalApiRequest {
    /// Request ID as Arc<str> for efficient sharing across async boundaries
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub request_id: Arc<str>,
    pub from_primal: EcosystemPrimalType,
    pub to_primal: EcosystemPrimalType,
    /// Operation name as Arc<str> with string interning
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub operation: Arc<str>,
    pub payload: serde_json::Value,
    /// Headers with Arc<str> keys and values for zero-copy
    #[serde(
        serialize_with = "serialize_arc_str_map",
        deserialize_with = "deserialize_arc_str_map"
    )]
    pub headers: HashMap<Arc<str>, Arc<str>>,
    pub timeout: Duration,
}

impl PrimalApiRequest {
    /// Create new PrimalApiRequest with string interning optimization
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
    pub fn get_header(&self, key: &str) -> Option<&Arc<str>> {
        self.headers
            .iter()
            .find(|(k, _)| k.as_ref() == key)
            .map(|(_, v)| v)
    }
}

/// Standard API response for inter-primal communication with Arc<str> optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalApiResponse {
    /// Request ID as Arc<str> for efficient correlation
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub request_id: Arc<str>,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    /// Error message as Arc<str> for efficient sharing
    #[serde(
        serialize_with = "serialize_optional_arc_str",
        deserialize_with = "deserialize_optional_arc_str"
    )]
    pub error: Option<Arc<str>>,
    /// Headers with Arc<str> keys and values for zero-copy
    #[serde(
        serialize_with = "serialize_arc_str_map",
        deserialize_with = "deserialize_arc_str_map"
    )]
    pub headers: HashMap<Arc<str>, Arc<str>>,
    pub processing_time: Duration,
}

impl PrimalApiResponse {
    /// Create new PrimalApiResponse with string optimization
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
            error: error.map(|e| Arc::from(e)),
            headers: headers
                .into_iter()
                .map(|(k, v)| (intern_registry_string(k), Arc::from(v)))
                .collect(),
            processing_time,
        }
    }
}

/// Ecosystem registry events with Arc<str> optimization for efficient event sharing
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
