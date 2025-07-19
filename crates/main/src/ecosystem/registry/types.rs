//! Core types for the ecosystem registry manager

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::ecosystem::EcosystemPrimalType;

/// Registry state tracking
#[derive(Debug, Default)]
pub struct RegistryState {
    pub registered_services: HashMap<String, Arc<crate::ecosystem::EcosystemServiceRegistration>>,
    pub service_discovery_cache: HashMap<String, Arc<DiscoveredService>>,
    pub last_discovery_sync: Option<DateTime<Utc>>,
    pub registration_attempts: u32,
}

/// Discovered service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredService {
    pub service_id: String,
    pub primal_type: EcosystemPrimalType,
    pub endpoint: String,
    pub health_endpoint: String,
    pub api_version: String,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub discovered_at: DateTime<Utc>,
    pub last_health_check: Option<DateTime<Utc>>,
    pub health_status: ServiceHealthStatus,
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

/// Standard API request for inter-primal communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalApiRequest {
    pub request_id: String,
    pub from_primal: EcosystemPrimalType,
    pub to_primal: EcosystemPrimalType,
    pub operation: String,
    pub payload: serde_json::Value,
    pub headers: HashMap<String, String>,
    pub timeout: Duration,
}

/// Standard API response for inter-primal communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalApiResponse {
    pub request_id: String,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub headers: HashMap<String, String>,
    pub processing_time: Duration,
}

/// Ecosystem registry events
#[derive(Debug, Clone)]
pub enum EcosystemRegistryEvent {
    ServiceRegistered {
        service_id: String,
        primal_type: EcosystemPrimalType,
        endpoint: String,
    },
    ServiceDiscovered {
        service_id: String,
        primal_type: EcosystemPrimalType,
        endpoint: String,
        capabilities: Vec<String>,
    },
    ServiceHealthChanged {
        service_id: String,
        primal_type: EcosystemPrimalType,
        old_status: ServiceHealthStatus,
        new_status: ServiceHealthStatus,
    },
    ServiceOffline {
        service_id: String,
        primal_type: EcosystemPrimalType,
        reason: String,
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
