// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Federation Module - Universal Cross-Platform Sovereignty
//!
//! This module implements the core federation capabilities for creating
//! sovereign, universal, and federated AI systems that can operate
//! across platforms, languages, and trust boundaries.
//!
//! ## Architecture Principles
//!
//! - **Universal Agnosticism**: Platform and language-independent execution
//! - **Data Sovereignty**: User-controlled data ownership and privacy
//! - **True Federation**: Multi-node coordination with consensus mechanisms
//! - **Cross-Platform**: Seamless operation across diverse environments

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod consensus;
pub mod cross_platform;
pub mod federation_network;
pub mod sovereign_data;
pub mod universal_executor;

use crate::traits::PrimalError;

/// Federation-specific error types
#[derive(Debug, thiserror::Error)]
pub enum FederationError {
    /// Unsupported platform error
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),

    /// Unsupported language error
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    /// Resource limit exceeded error
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    /// Execution not found error
    #[error("Execution not found: {0}")]
    ExecutionNotFound(Uuid),

    /// Not implemented error
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Consensus failure error
    #[error("Consensus failure: {0}")]
    ConsensusFailure(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Security violation error
    #[error("Security violation: {0}")]
    SecurityViolation(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Timeout error
    #[error("Timeout error: {0}")]
    TimeoutError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Already running error
    #[error("Already running: {0}")]
    AlreadyRunning(String),

    /// Peer not found error
    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    /// Broadcast failed error
    #[error("Broadcast failed: {0}")]
    BroadcastFailed(String),

    /// Connection closed error
    #[error("Connection closed: {0}")]
    ConnectionClosed(String),

    /// No messages available error
    #[error("No messages available: {0}")]
    NoMessagesAvailable(String),
}

/// Result type for federation operations
pub type FederationResult<T> = Result<T, FederationError>;

/// Universal execution environment abstraction
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    /// Linux variants with specific distributions
    Linux(LinuxVariant),
    /// Windows variants with version information
    Windows(WindowsVariant),
    /// macOS variants with version information
    MacOS(MacOSVariant),
    /// WebAssembly runtime environment
    WebAssembly,
    /// Container runtime (Docker, Podman, etc.)
    Container(ContainerRuntime),
    /// Cloud provider platforms
    Cloud(CloudProvider),
    /// Mobile platforms
    Mobile(MobileVariant),
    /// IoT and embedded devices
    Embedded(EmbeddedPlatform),
}

/// Linux distribution variants
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinuxVariant {
    /// Ubuntu distribution
    Ubuntu,
    /// Debian distribution
    Debian,
    /// CentOS distribution
    CentOS,
    /// Red Hat Enterprise Linux
    RHEL,
    /// Fedora distribution
    Fedora,
    /// Arch Linux distribution
    Arch,
    /// Alpine Linux distribution
    Alpine,
    /// Generic Linux distribution
    Generic(String),
}

/// Windows variants
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WindowsVariant {
    /// Windows 10
    Windows10,
    /// Windows 11
    Windows11,
    /// Windows Server 2019
    WindowsServer2019,
    /// Windows Server 2022
    WindowsServer2022,
    /// Generic Windows version
    Generic(String),
}

/// macOS variants
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MacOSVariant {
    /// macOS Monterey
    Monterey,
    /// macOS Ventura
    Ventura,
    /// macOS Sonoma
    Sonoma,
    /// Generic macOS version
    Generic(String),
}

/// Container runtime types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContainerRuntime {
    /// Docker container runtime
    Docker,
    /// Podman container runtime
    Podman,
    /// Containerd container runtime
    Containerd,
    /// CRI-O container runtime
    CriO,
    /// Generic container runtime
    Generic(String),
}

/// Cloud provider platforms
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CloudProvider {
    /// Amazon Web Services
    AWS,
    /// Microsoft Azure
    Azure,
    /// Google Cloud Platform
    GCP,
    /// DigitalOcean
    DigitalOcean,
    /// Generic cloud provider
    Generic(String),
}

/// Mobile platform variants
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MobileVariant {
    /// Apple iOS
    #[expect(non_camel_case_types, reason = "iOS is standard platform identifier")]
    iOS,
    /// Google Android
    Android,
    /// Huawei HarmonyOS
    HarmonyOS,
    /// Generic mobile platform
    Generic(String),
}

/// Embedded platform types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmbeddedPlatform {
    /// Raspberry Pi
    RaspberryPi,
    /// Arduino
    Arduino,
    /// STM32 microcontroller
    STM32,
    /// ESP32 microcontroller
    ESP32,
    /// Generic embedded platform
    Generic(String),
}

/// Node in the federation network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationNode {
    /// Unique node identifier
    pub id: Uuid,
    /// Node address (IP or hostname)
    pub address: String,
    /// Node port
    pub port: u16,
    /// Node public key for encryption
    pub public_key: String,
    /// Node capabilities
    pub capabilities: Vec<String>,
    /// Node status
    pub status: NodeStatus,
    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,
    /// Node metadata
    pub metadata: HashMap<String, String>,
}

/// Node status in the federation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeStatus {
    /// Node is active and available
    Active,
    /// Node is temporarily unavailable
    Inactive,
    /// Node has failed
    Failed,
    /// Node is being maintained
    Maintenance,
    /// Node status is unknown
    Unknown,
}

/// Federation network trait
#[async_trait]
pub trait FederationNetwork: Send + Sync {
    /// Join the federation network
    async fn join(&self, node: FederationNode) -> FederationResult<()>;

    /// Leave the federation network
    async fn leave(&self, node_id: Uuid) -> FederationResult<()>;

    /// Discover nodes in the network
    async fn discover_nodes(&self) -> FederationResult<Vec<FederationNode>>;

    /// Send message to a specific node
    async fn send_message(&self, node_id: Uuid, message: Vec<u8>) -> FederationResult<Vec<u8>>;

    /// Broadcast message to all nodes
    async fn broadcast_message(&self, message: Vec<u8>) -> FederationResult<()>;

    /// Get network status
    async fn network_status(&self) -> FederationResult<NetworkStatus>;
}

/// Network status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    /// Total number of nodes
    pub total_nodes: usize,
    /// Number of active nodes
    pub active_nodes: usize,
    /// Number of failed nodes
    pub failed_nodes: usize,
    /// Network health score (0.0 to 1.0)
    pub health_score: f64,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Consensus mechanism trait
#[async_trait]
pub trait ConsensusManager: Send + Sync {
    /// Propose a new value for consensus
    async fn propose(&self, value: Vec<u8>) -> FederationResult<ConsensusResult>;

    /// Vote on a proposed value
    async fn vote(&self, proposal_id: Uuid, vote: Vote) -> FederationResult<()>;

    /// Get consensus result
    async fn get_result(&self, proposal_id: Uuid) -> FederationResult<ConsensusResult>;

    /// Get current consensus state
    async fn get_state(&self) -> FederationResult<ConsensusState>;
}

/// Consensus result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    /// Proposal identifier
    pub proposal_id: Uuid,
    /// Consensus status
    pub status: ConsensusStatus,
    /// Agreed value (if consensus reached)
    pub value: Option<Vec<u8>>,
    /// Number of votes for
    pub votes_for: u32,
    /// Number of votes against
    pub votes_against: u32,
    /// Participating nodes
    pub participating_nodes: Vec<Uuid>,
}

/// Consensus status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ConsensusStatus {
    /// Consensus reached
    Agreed,
    /// Consensus not reached
    Disagreed,
    /// Consensus in progress
    #[default]
    InProgress,
    /// Consensus timed out
    TimedOut,
    /// Consensus failed
    Failed,
}

/// Vote type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Vote {
    /// Vote in favor
    For,
    /// Vote against
    Against,
    /// Abstain from voting
    Abstain,
}

/// Consensus state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusState {
    /// Current round number
    pub round: u64,
    /// Active proposals
    pub active_proposals: Vec<Uuid>,
    /// Recent consensus results
    pub recent_results: Vec<ConsensusResult>,
    /// Node participation stats
    pub participation_stats: HashMap<Uuid, ParticipationStats>,
}

/// Node participation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationStats {
    /// Total proposals participated in
    pub total_proposals: u32,
    /// Proposals voted for
    pub votes_for: u32,
    /// Proposals voted against
    pub votes_against: u32,
    /// Proposals abstained from
    pub abstentions: u32,
    /// Participation rate (0.0 to 1.0)
    pub participation_rate: f64,
}

/// Sovereign data management trait
#[async_trait]
pub trait SovereignDataManager: Send + Sync {
    /// Store data with sovereignty metadata
    async fn store_data(&self, data: SovereignData) -> FederationResult<DataId>;

    /// Retrieve data by ID
    async fn retrieve_data(&self, id: DataId) -> FederationResult<SovereignData>;

    /// Delete data
    async fn delete_data(&self, id: DataId) -> FederationResult<()>;

    /// List data owned by a specific owner
    async fn list_data(&self, owner: &str) -> FederationResult<Vec<DataId>>;

    /// Check data access permissions
    async fn check_access(&self, id: DataId, requester: &str) -> FederationResult<bool>;
}

/// Sovereign data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignData {
    /// Data identifier
    pub id: DataId,
    /// Data owner
    pub owner: String,
    /// Data content
    pub content: Vec<u8>,
    /// Access permissions
    pub permissions: DataPermissions,
    /// Encryption metadata
    pub encryption: EncryptionMetadata,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,
}

/// Data identifier type
pub type DataId = Uuid;

/// Data permissions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataPermissions {
    /// Users with read access
    pub read_users: Vec<String>,
    /// Users with write access
    pub write_users: Vec<String>,
    /// Users with admin access
    pub admin_users: Vec<String>,
    /// Public read access
    pub public_read: bool,
    /// Public write access
    pub public_write: bool,
}

/// Encryption metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    /// Encryption algorithm used
    pub algorithm: String,
    /// Key derivation function
    pub key_derivation: String,
    /// Initialization vector
    pub iv: Vec<u8>,
    /// Salt for key derivation
    pub salt: Vec<u8>,
    /// Whether data is encrypted
    pub encrypted: bool,
}

/// Cross-platform executor trait
#[async_trait]
pub trait CrossPlatformExecutor: Send + Sync {
    /// Execute code on a remote platform
    async fn execute_remote(
        &self,
        platform: Platform,
        code: String,
    ) -> FederationResult<ExecutionResult>;

    /// Get available platforms
    async fn available_platforms(&self) -> FederationResult<Vec<Platform>>;

    /// Check platform availability
    async fn is_platform_available(&self, platform: &Platform) -> FederationResult<bool>;
}

/// Execution result for cross-platform operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Execution success status
    pub success: bool,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit code
    pub exit_code: Option<i32>,
    /// Execution duration
    pub duration_ms: u64,
}

/// Default implementations for common types
impl Default for NodeStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

impl Default for EncryptionMetadata {
    fn default() -> Self {
        Self {
            algorithm: "AES-256-GCM".to_string(),
            key_derivation: "PBKDF2".to_string(),
            iv: vec![],
            salt: vec![],
            encrypted: false,
        }
    }
}

/// Conversion between federation and primal errors
impl From<FederationError> for PrimalError {
    fn from(err: FederationError) -> Self {
        match err {
            FederationError::UnsupportedPlatform(msg) => PrimalError::NotImplemented(msg),
            FederationError::UnsupportedLanguage(msg) => PrimalError::NotImplemented(msg),
            FederationError::ResourceLimitExceeded(msg) => PrimalError::Resource(msg),
            FederationError::ExecutionNotFound(id) => {
                PrimalError::NotFound(format!("execution:{id}"))
            }
            FederationError::NotImplemented(msg) => PrimalError::NotImplemented(msg),
            FederationError::ConsensusFailure(msg) => {
                PrimalError::InternalError(format!("Consensus failure: {}", msg))
            }
            FederationError::NetworkError(msg) => PrimalError::NetworkError(msg),
            FederationError::SecurityViolation(msg) => PrimalError::Security(msg),
            FederationError::SerializationError(msg) => PrimalError::Internal(msg),
            FederationError::ConfigurationError(msg) => PrimalError::Configuration(msg),
            FederationError::TimeoutError(msg) => PrimalError::Timeout(msg),
            FederationError::InternalError(msg) => PrimalError::InternalError(msg),
            FederationError::AlreadyRunning(msg) => PrimalError::AlreadyExists(msg),
            FederationError::PeerNotFound(msg) => PrimalError::NotFound(msg),
            FederationError::BroadcastFailed(msg) => PrimalError::NetworkError(msg),
            FederationError::ConnectionClosed(msg) => PrimalError::NetworkError(msg),
            FederationError::NoMessagesAvailable(msg) => PrimalError::NotFound(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_serialization() {
        let platform = Platform::Linux(LinuxVariant::Ubuntu);
        let serialized = serde_json::to_string(&platform).unwrap();
        let deserialized: Platform = serde_json::from_str(&serialized).unwrap();
        assert_eq!(platform, deserialized);
    }

    #[test]
    fn test_federation_node_creation() {
        let node = FederationNode {
            id: Uuid::new_v4(),
            address: "192.168.1.100".to_string(),
            port: 8080,
            public_key: "test_key".to_string(),
            capabilities: vec!["execute".to_string(), "store".to_string()],
            status: NodeStatus::Active,
            last_seen: Utc::now(),
            metadata: HashMap::new(),
        };

        assert_eq!(node.status, NodeStatus::Active);
        assert_eq!(node.capabilities.len(), 2);
    }

    #[test]
    fn test_sovereign_data_permissions() {
        let permissions = DataPermissions {
            read_users: vec!["user1".to_string(), "user2".to_string()],
            write_users: vec!["user1".to_string()],
            admin_users: vec!["admin".to_string()],
            public_read: false,
            public_write: false,
        };

        assert_eq!(permissions.read_users.len(), 2);
        assert_eq!(permissions.write_users.len(), 1);
        assert!(!permissions.public_read);
    }

    #[test]
    fn test_federation_error_conversion() {
        let fed_error = FederationError::UnsupportedPlatform("test".to_string());
        let primal_error: PrimalError = fed_error.into();

        match primal_error {
            PrimalError::NotImplemented(feature) => assert_eq!(feature, "test"),
            _ => panic!("Unexpected error type"),
        }
    }
}
