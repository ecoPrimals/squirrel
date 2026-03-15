// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Service-related types for the composition system

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use super::super::super::providers::UniversalAIProvider;
use super::dependency::ServiceDependency;
use crate::resilience::retry::{RetryConfig, BackoffStrategy};

// Import canonical config types
use crate::config::ScalingConfig;

/// AI Service representation
#[derive(Debug, Clone)]
pub struct AIService {
    /// Service ID
    pub id: String,
    
    /// Service name
    pub name: String,
    
    /// Service description
    pub description: String,
    
    /// Service configuration
    pub config: ServiceConfig,
    
    /// Service capabilities
    pub capabilities: Vec<ServiceCapability>,
    
    /// Service dependencies
    pub dependencies: Vec<ServiceDependency>,
    
    /// Service health status
    pub health: Arc<RwLock<ServiceHealth>>,
    
    /// Service metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Service provider
    pub provider: Arc<dyn UniversalAIProvider>,
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service type
    pub service_type: ServiceType,
    
    /// Service endpoint
    pub endpoint: String,
    
    /// Service authentication
    pub auth: Option<ServiceAuth>,
    
    /// Service timeout
    pub timeout: Duration,
    
    /// Service retry settings
    pub retry: RetryConfig,
    
    /// Service resource limits
    pub resources: ResourceLimits,
    
    /// Service scaling configuration
    pub scaling: ScalingConfig,
    
    /// Service version
    pub version: Option<String>,
}

/// Service types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceType {
    /// AI inference service
    Inference,
    
    /// AI training service
    Training,
    
    /// AI preprocessing service
    Preprocessing,
    
    /// AI postprocessing service
    Postprocessing,
    
    /// AI aggregation service
    Aggregation,
    
    /// AI validation service
    Validation,
    
    /// AI monitoring service
    Monitoring,
    
    /// Custom service type
    Custom(String),
}

/// Service authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAuth {
    /// Authentication type
    pub auth_type: AuthType,
    
    /// Authentication credentials
    pub credentials: HashMap<String, String>,
    
    /// Authentication metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Authentication types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    /// No authentication
    None,
    
    /// API key authentication
    ApiKey,
    
    /// Bearer token authentication
    Bearer,
    
    /// OAuth2 authentication
    OAuth2,
    
    /// Basic authentication
    Basic,
    
    /// Custom authentication
    Custom(String),
}

/// Resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage (bytes)
    pub max_memory: u64,
    
    /// Maximum CPU usage (cores)
    pub max_cpu: f64,
    
    /// Maximum execution time
    pub max_execution_time: Duration,
    
    /// Maximum concurrent requests
    pub max_concurrent_requests: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        // Load unified config for environment-aware timeout values
        let config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .and_then(|loaded| loaded.try_into_config().ok());
        
        let max_execution_time = if let Some(cfg) = config {
            cfg.timeouts.get_custom_timeout("svc_max_execution")
                .unwrap_or_else(|| Duration::from_secs(300))
        } else {
            Duration::from_secs(300) // 5 minutes
        };
        
        Self {
            max_memory: 1024 * 1024 * 1024, // 1GB
            max_cpu: 1.0,
            max_execution_time,
            max_concurrent_requests: 10,
        }
    }
}

// ScalingConfig is now imported from crate::config (includes Default impl)

/// Service capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapability {
    /// Capability name
    pub name: String,
    
    /// Capability description
    pub description: String,
    
    /// Capability parameters
    pub parameters: serde_json::Value,
    
    /// Capability constraints
    pub constraints: Vec<CapabilityConstraint>,
    
    /// Capability performance metrics
    pub performance: Option<CapabilityPerformance>,
}

/// Capability constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityConstraint {
    /// Constraint type
    pub constraint_type: ConstraintType,
    
    /// Constraint value
    pub value: serde_json::Value,
    
    /// Constraint description
    pub description: String,
}

/// Constraint types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Maximum input size
    MaxInputSize,
    
    /// Maximum output size
    MaxOutputSize,
    
    /// Maximum processing time
    MaxProcessingTime,
    
    /// Required input format
    RequiredInputFormat,
    
    /// Required output format
    RequiredOutputFormat,
    
    /// Minimum quality score
    MinQualityScore,
    
    /// Resource requirements
    ResourceRequirements,
    
    /// Custom constraint
    Custom(String),
}

/// Capability performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityPerformance {
    /// Average latency
    pub avg_latency: Duration,
    
    /// Throughput (requests per second)
    pub throughput: f64,
    
    /// Success rate
    pub success_rate: f64,
    
    /// Quality score
    pub quality_score: f64,
    
    /// Cost per request
    pub cost_per_request: f64,
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    /// Health status
    pub status: HealthStatus,
    
    /// Last health check timestamp
    pub last_check: DateTime<Utc>,
    
    /// Response time
    pub response_time: Duration,
    
    /// Error rate
    pub error_rate: f64,
    
    /// Service availability
    pub availability: f64,
    
    /// Health metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for ServiceHealth {
    fn default() -> Self {
        Self {
            status: HealthStatus::Unknown,
            last_check: Utc::now(),
            response_time: Duration::from_millis(0),
            error_rate: 0.0,
            availability: 1.0,
            metadata: HashMap::new(),
        }
    }
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,
    
    /// Service is degraded
    Degraded,
    
    /// Service is unhealthy
    Unhealthy,
    
    /// Health status unknown
    Unknown,
}

/// Service discovery entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryEntry {
    /// Service ID
    pub service_id: String,
    
    /// Service name
    pub service_name: String,
    
    /// Service endpoint
    pub endpoint: String,
    
    /// Service port
    pub port: u16,
    
    /// Service tags
    pub tags: Vec<String>,
    
    /// Service metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check endpoint
    pub endpoint: String,
    
    /// Health check interval
    pub interval: Duration,
    
    /// Health check timeout
    pub timeout: Duration,
    
    /// Health check retries
    pub retries: u32,
    
    /// Expected status code
    pub expected_status: u16,
    
    /// Health check metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        // Load unified config for environment-aware timeout values
        let config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .and_then(|loaded| loaded.try_into_config().ok());
        
        let (interval, timeout) = if let Some(cfg) = config {
            let i = cfg.timeouts.get_custom_timeout("svc_health_interval")
                .unwrap_or_else(|| Duration::from_secs(30));
            let t = cfg.timeouts.get_custom_timeout("svc_health_timeout")
                .unwrap_or_else(|| Duration::from_secs(5));
            (i, t)
        } else {
            (Duration::from_secs(30), Duration::from_secs(5))
        };
        
        Self {
            endpoint: "/health".to_string(),
            interval,
            timeout,
            retries: 3,
            expected_status: 200,
            metadata: HashMap::new(),
        }
    }
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthStatus {
    /// Service ID
    pub service_id: String,
    
    /// Health status
    pub status: HealthStatus,
    
    /// Last check timestamp
    pub last_check: DateTime<Utc>,
    
    /// Check result details
    pub details: HashMap<String, serde_json::Value>,
}

/// Service health checker trait
#[async_trait::async_trait]
pub trait ServiceHealthChecker: Send + Sync + std::fmt::Debug {
    /// Check service health
    async fn check_health(&self, service_id: &str) -> Result<ServiceHealthStatus, crate::error::types::MCPError>;
    
    /// Get checker name
    fn checker_name(&self) -> &str;
}

/// Service discovery provider trait
#[async_trait::async_trait]
pub trait ServiceDiscoveryProvider: Send + Sync + std::fmt::Debug {
    /// Discover services
    async fn discover_services(&self) -> Result<Vec<ServiceDiscoveryEntry>, crate::error::types::MCPError>;
    
    /// Get provider name
    fn provider_name(&self) -> &str;
} 