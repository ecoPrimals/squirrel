//! Universal AI Provider Types
//!
//! Core type definitions for capability-based AI provider discovery and integration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Universal capability registry - purely capability-based, no service identities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRegistry {
    /// All discovered capabilities in the ecosystem
    pub capabilities: HashMap<String, Vec<CapabilityProvider>>,
    /// Performance characteristics indexed by capability
    pub performance_index: HashMap<String, Vec<PerformanceProfile>>,
    /// Last registry update
    pub last_updated: std::time::SystemTime,
}

/// A provider of a specific capability - identity-agnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityProvider {
    /// Anonymous service identifier (generated, not semantic)
    pub provider_id: Uuid,
    /// The specific capability this provider offers
    pub capability: AICapability,
    /// How to communicate with this capability
    pub interface: CapabilityInterface,
    /// Performance characteristics
    pub performance: PerformanceProfile,
    /// Resource requirements
    pub resources: ResourceProfile,
    /// Trust and reliability metrics
    pub trust_metrics: TrustMetrics,
}

/// Pure capability definition - what can be done, not who does it
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AICapability {
    /// Capability type (e.g., "text-generation", "image-analysis", "code-completion")
    pub capability_type: String,
    /// Input data formats accepted
    pub input_formats: Vec<DataFormat>,
    /// Output data formats provided
    pub output_formats: Vec<DataFormat>,
    /// Processing characteristics
    pub processing_type: ProcessingType,
    /// Quality characteristics
    pub quality_profile: QualityProfile,
    /// Cost characteristics
    pub cost_profile: CostProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataFormat {
    Text,
    JSON,
    Binary,
    Stream,
    MultiPart,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingType {
    /// Synchronous request-response
    Synchronous,
    /// Asynchronous with polling
    Asynchronous,
    /// Real-time streaming
    Streaming,
    /// Batch processing
    Batch,
    /// Interactive/conversational
    Interactive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityProfile {
    /// Accuracy score (0.0 - 1.0)
    pub accuracy: f64,
    /// Consistency score (0.0 - 1.0)  
    pub consistency: f64,
    /// Context understanding (0.0 - 1.0)
    pub context_understanding: f64,
    /// Specialization areas
    pub specializations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostProfile {
    /// Cost per request
    pub cost_per_request: f64,
    /// Cost per unit (token, word, etc.)
    pub cost_per_unit: f64,
    /// Is this capability free?
    pub is_free: bool,
    /// Cost tier
    pub tier: CostTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostTier {
    Free,
    Low,
    Medium,
    High,
    Premium,
}

/// How to communicate with a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInterface {
    /// Communication protocol
    pub protocol: CommunicationProtocol,
    /// Connection details
    pub endpoint: EndpointInfo,
    /// Authentication requirements
    pub auth: AuthRequirements,
    /// Message format
    pub message_format: MessageFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationProtocol {
    HTTP,
    WebSocket,
    #[serde(rename = "gRPC")]
    GRpc,
    MessageQueue,
    ProcessCall,
    FileExchange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointInfo {
    pub address: String,
    pub port: Option<u16>,
    pub path: Option<String>,
    pub tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthRequirements {
    None,
    ApiKey { header: String },
    Bearer { token_endpoint: Option<String> },
    Basic { realm: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageFormat {
    JSON,
    ProtocolBuffer,
    MessagePack,
    Custom(String),
}

/// Performance characteristics of a capability provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    pub average_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub throughput_requests_per_second: f64,
    pub availability: f64,
    pub reliability: f64,
}

/// Resource requirements for a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceProfile {
    pub compute_intensity: ComputeIntensity,
    pub memory_usage_mb: Option<u64>,
    pub network_bandwidth_mbps: Option<f64>,
    pub storage_requirements: Option<StorageRequirements>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeIntensity {
    Light,
    Medium,
    Heavy,
    Extreme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRequirements {
    pub persistent: bool,
    pub size_mb: u64,
    pub iops_required: Option<u32>,
}

/// Trust and reliability metrics for a capability provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustMetrics {
    pub uptime_percentage: f64,
    pub error_rate: f64,
    pub response_consistency: f64,
    pub security_score: f64,
    pub community_rating: Option<f64>,
}

/// Performance tracking for requests
#[derive(Debug, Clone)]
pub struct PerformanceHistory {
    pub metrics: Vec<RequestMetric>,
    pub average_latency: f64,
    pub success_rate: f64,
    pub last_updated: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub struct RequestMetric {
    pub timestamp: std::time::Instant,
    pub latency_ms: u64,
    pub success: bool,
    pub bytes_transferred: Option<u64>,
}
