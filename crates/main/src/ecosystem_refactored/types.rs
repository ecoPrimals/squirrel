//! Shared types for ecosystem management
//!
//! This module contains common types used across all ecosystem submodules.
//! Following the principle of semantic cohesion, these types are used by
//! multiple ecosystem components and belong together.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::PrimalError;
use crate::primal_provider::SquirrelPrimalProvider;
use crate::universal::{PrimalContext, SecurityLevel};

/// Ecosystem service registration for Squirrel AI primal
///
/// This struct follows the standardized format for service discovery
/// and registration within the ecoPrimals ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemServiceRegistration {
    /// Unique service identifier
    pub service_id: String,
    /// Human-readable service name
    pub service_name: String,
    /// Service type/category
    pub service_type: String,
    /// Service version
    pub version: String,
    /// Service endpoints
    pub endpoints: ServiceEndpoints,
    /// Service capabilities
    pub capabilities: ServiceCapabilities,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
    /// Security configuration
    pub security: SecurityConfig,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Standardized primal types for ecosystem integration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EcosystemPrimalType {
    /// Squirrel - AI agent platform
    Squirrel,
    /// BearDog - Security services
    BearDog,
    /// Songbird - Coordination/service mesh
    Songbird,
    /// ToadStool - Distributed compute
    ToadStool,
    /// NestGate - Storage services
    NestGate,
    /// BiomeOS - OS integration
    BiomeOS,
}

impl EcosystemPrimalType {
    /// Get primal name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Squirrel => "squirrel",
            Self::BearDog => "beardog",
            Self::Songbird => "songbird",
            Self::ToadStool => "toadstool",
            Self::NestGate => "nestgate",
            Self::BiomeOS => "biomeos",
        }
    }
}

/// Service capabilities with proper Default implementation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceCapabilities {
    pub capabilities: Vec<String>,
}

/// Service endpoints with proper Default implementation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceEndpoints {
    pub http: Option<String>,
    pub grpc: Option<String>,
    pub websocket: Option<String>,
}

/// Health check configuration with Default implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub endpoint: String,
    pub interval_seconds: u64,
    pub timeout_seconds: u64,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            endpoint: "/health".to_string(),
            interval_seconds: 30,
            timeout_seconds: 5,
        }
    }
}

/// Resource requirements specification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceSpec {
    pub cpu_cores: Option<f64>,
    pub memory_mb: Option<u64>,
    pub disk_gb: Option<u64>,
}

/// Security configuration for ecosystem integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub requires_authentication: bool,
    pub requires_encryption: bool,
    pub allowed_origins: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            requires_authentication: true,
            requires_encryption: true,
            allowed_origins: vec!["*".to_string()],
        }
    }
}

/// Resource requirements specification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub minimum: ResourceSpec,
    pub recommended: ResourceSpec,
    pub maximum: Option<ResourceSpec>,
}

/// Health status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall health score (0.0 to 1.0)
    pub health_score: f64,
    /// Individual component health statuses
    pub component_statuses: HashMap<String, ComponentHealth>,
    /// Timestamp of last health check
    pub last_check: DateTime<Utc>,
    /// List of health errors (if any)
    pub health_errors: Vec<String>,
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Health status
    pub status: String,
    /// Last check timestamp
    pub last_check: DateTime<Utc>,
    /// Error message (if any)
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Ecosystem status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemStatus {
    /// Overall status
    pub status: String,
    /// Ecosystem manager status
    pub manager: EcosystemManagerStatus,
    /// Service mesh status
    pub service_mesh: ServiceMeshStatus,
    /// Cross-primal communication status
    pub cross_primal: CrossPrimalStatus,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Service mesh status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshStatus {
    /// Connection status
    pub connected: bool,
    /// Endpoint
    pub endpoint: String,
    /// Last heartbeat
    pub last_heartbeat: Option<DateTime<Utc>>,
}

/// Cross-primal communication status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPrimalStatus {
    /// Connected primals
    pub connected_primals: Vec<String>,
    /// Failed connections
    pub failed_connections: Vec<String>,
}

/// Ecosystem manager status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemManagerStatus {
    /// Current status
    pub status: String,
    /// Initialization timestamp
    pub initialized_at: Option<DateTime<Utc>>,
    /// Last registration timestamp
    pub last_registration: Option<DateTime<Utc>>,
    /// Active registrations
    pub active_registrations: Vec<String>,
    /// Health status
    pub health_status: HealthStatus,
    /// Error count
    pub error_count: u64,
    /// Last error
    pub last_error: Option<String>,
}

impl std::fmt::Display for EcosystemPrimalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

