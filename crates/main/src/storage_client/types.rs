// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Universal Storage Client Types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// CONFIGURATION TYPES
// ============================================================================

/// Configuration for universal storage client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageClientConfig {
    /// Timeout for storage operations
    pub operation_timeout: std::time::Duration,

    /// Maximum retries for failed operations
    pub max_retries: u32,

    /// Preferred storage capabilities
    pub preferred_capabilities: Vec<StorageCapabilityPreference>,

    /// Data classification requirements
    pub data_classification: DataClassification,

    /// Performance requirements
    pub performance_requirements: PerformanceRequirements,
}

/// Storage capability preferences for intelligent routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageCapabilityPreference {
    /// Capability type
    pub capability: StorageCapabilityType,

    /// Priority weight (0.0 - 1.0)
    pub weight: f64,

    /// Required vs optional
    pub required: bool,
}

/// Types of storage capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageCapabilityType {
    /// High-performance object storage
    ObjectStorage {
        compression: bool,
        encryption: bool,
        replication: bool,
    },

    /// File system storage
    FileSystem {
        posix_compliance: bool,
        atomic_operations: bool,
    },

    /// Database storage
    Database {
        acid_compliance: bool,
        query_capabilities: Vec<String>,
    },

    /// Cache storage
    Cache {
        ttl_support: bool,
        eviction_policies: Vec<String>,
    },

    /// Archive/cold storage
    Archive {
        retrieval_time: std::time::Duration,
        cost_optimization: bool,
    },
}

/// Data classification for compliance and routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataClassification {
    /// Public data - no special requirements
    Public,

    /// Internal data - basic security
    Internal,

    /// Confidential data - encryption required
    Confidential,

    /// Restricted data - maximum security
    Restricted,
}

/// Performance requirements for intelligent routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRequirements {
    /// Maximum acceptable latency
    pub max_latency_ms: u64,

    /// Minimum throughput (MB/s)
    pub min_throughput_mbps: f64,

    /// Availability requirement (0.0 - 1.0)
    pub availability_sla: f64,

    /// Durability requirement (number of 9s)
    pub durability_nines: u8,
}

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

/// Universal storage request - AI-first design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalStorageRequest {
    /// Unique request identifier
    pub request_id: Uuid,

    /// Operation type
    pub operation: StorageOperation,

    /// Object/data identifier
    pub object_key: String,

    /// Data payload (for write operations)
    pub data: Option<Vec<u8>>,

    /// Request metadata
    pub metadata: HashMap<String, String>,

    /// Data classification
    pub classification: DataClassification,

    /// Performance requirements
    pub requirements: PerformanceRequirements,

    /// AI context for intelligent routing
    pub ai_context: AIRequestContext,
}

/// Types of storage operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageOperation {
    /// Store data
    Store,

    /// Retrieve data
    Retrieve,

    /// List objects
    List,

    /// Delete data
    Delete,

    /// Copy data
    Copy { destination: String },

    /// Move data
    Move { destination: String },

    /// Create snapshot
    Snapshot,

    /// Restore from snapshot
    Restore { snapshot_id: String },
}

/// AI context for intelligent request routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRequestContext {
    /// Expected access frequency
    pub access_frequency: AccessFrequency,

    /// Data lifetime expectation
    pub data_lifetime: std::time::Duration,

    /// Sharing requirements
    pub sharing_scope: SharingScope,

    /// Processing pipeline requirements
    pub processing_hints: Vec<String>,
}

/// Expected access frequency for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessFrequency {
    /// Frequently accessed (hot data)
    Hot,

    /// Occasionally accessed (warm data)  
    Warm,

    /// Rarely accessed (cold data)
    Cold,

    /// Archive data (rarely retrieved)
    Archive,
}

/// Data sharing scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SharingScope {
    /// Private to user
    Private,

    /// Shared within team
    Team,

    /// Shared within organization
    Organization,

    /// Public access
    Public,
}

/// Universal storage response - AI-first design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalStorageResponse {
    /// Request identifier
    pub request_id: Uuid,

    /// Operation success
    pub success: bool,

    /// Response data
    pub data: Option<Vec<u8>>,

    /// Response metadata
    pub metadata: HashMap<String, String>,

    /// Provider that handled the request
    pub provider_id: String,

    /// Performance metrics
    pub performance: PerformanceMetrics,

    /// AI insights and recommendations
    pub ai_insights: AIStorageInsights,

    /// Error information (if applicable)
    pub error: Option<String>,
}

/// Performance metrics for the operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Operation latency (ms)
    pub latency_ms: f64,

    /// Data throughput (MB/s)
    pub throughput_mbps: f64,

    /// Provider health score during operation
    pub provider_health: f64,

    /// Cost estimate
    pub estimated_cost: f64,
}

/// AI insights and recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIStorageInsights {
    /// Confidence in operation success
    pub confidence_score: f64,

    /// Suggested optimizations
    pub optimizations: Vec<String>,

    /// Alternative providers for consideration
    pub alternative_providers: Vec<String>,

    /// Predicted future access patterns
    pub access_predictions: Vec<AccessPattern>,

    /// Cost optimization recommendations
    pub cost_recommendations: Vec<String>,
}

/// Predicted access patterns for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPattern {
    /// Pattern type
    pub pattern_type: String,

    /// Confidence in prediction
    pub confidence: f64,

    /// Suggested optimizations
    pub optimizations: Vec<String>,
}

// ============================================================================
// DEFAULT IMPLEMENTATIONS
// ============================================================================

impl Default for StorageClientConfig {
    fn default() -> Self {
        Self {
            operation_timeout: std::time::Duration::from_secs(300),
            max_retries: 3,
            preferred_capabilities: vec![StorageCapabilityPreference {
                capability: StorageCapabilityType::ObjectStorage {
                    compression: true,
                    encryption: true,
                    replication: true,
                },
                weight: 0.9,
                required: true,
            }],
            data_classification: DataClassification::Internal,
            performance_requirements: PerformanceRequirements {
                max_latency_ms: 5000,
                min_throughput_mbps: 10.0,
                availability_sla: 0.99,
                durability_nines: 11,
            },
        }
    }
}
