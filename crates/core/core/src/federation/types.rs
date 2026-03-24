// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::{FederationStatus, FederationTopology};

/// Tunables for node identity, discovery, scaling, and federation networking.
#[derive(Debug, Clone)]
pub struct FederationConfig {
    /// Unique identifier for this node in the federation.
    pub node_id: String,
    /// Primary listen port for Squirrel on this node.
    pub port: u16,
    /// Endpoints or URLs used to discover other federation members.
    pub federation_discovery_urls: Vec<String>,
    /// Whether automatic scale-up and scale-down loops are enabled.
    pub auto_scaling_enabled: bool,
    /// Minimum Squirrel instances to retain when scaling down.
    pub min_instances: u32,
    /// Maximum Squirrel instances allowed on this node or cluster slice.
    pub max_instances: u32,
    /// Utilization fraction above which scale-up is considered.
    pub scale_up_threshold: f64,
    /// Utilization fraction below which scale-down is considered.
    pub scale_down_threshold: f64,
    /// Interval between periodic health checks of federation peers.
    pub health_check_interval: chrono::Duration,
    /// Timeout for federation RPC or join operations.
    pub federation_timeout: chrono::Duration,
    /// Master switch for federation features versus standalone mode.
    pub federation_enabled: bool,
    /// Optional region label for topology-aware routing.
    pub region: Option<String>,
    /// Optional availability zone within the region.
    pub zone: Option<String>,
    /// Cap on concurrently managed local instances on this node.
    pub max_local_instances: u32,
    /// Interval between auto-scaling evaluations.
    pub scaling_check_interval: chrono::Duration,
    /// Logical network shape used for federation messaging assumptions.
    pub topology: FederationTopology,
    /// Port exposed for federation control or peer traffic.
    pub federation_port: u16,
}

/// Snapshot of a remote or peer federation member and its observed health.
#[derive(Debug, Clone)]
pub struct FederationNode {
    /// Stable node identifier within the federation.
    pub id: String,
    /// Base URL or socket address for contacting this node.
    pub endpoint: String,
    /// Optional region label for locality.
    pub region: Option<String>,
    /// Optional zone label within the region.
    pub zone: Option<String>,
    /// Advertised capability strings used for routing and admission.
    pub capabilities: Vec<String>,
    /// Maximum concurrent work units this node claims to accept.
    pub capacity: u32,
    /// Observed load against `capacity` at last measurement.
    pub current_load: u32,
    /// Aggregated health classification from probes or heartbeats.
    pub health: NodeHealth,
    /// Last time this node responded to discovery or health traffic.
    pub last_seen: DateTime<Utc>,
    /// Arbitrary key-value metadata from registration or gossip.
    pub metadata: HashMap<String, String>,
}

/// Coarse health classification for a federation node or link.
#[derive(Debug, Clone)]
pub enum NodeHealth {
    /// Fully responsive within SLOs.
    Healthy,
    /// Partially impaired but still contactable.
    Degraded,
    /// Failing checks or returning errors.
    Unhealthy,
    /// No recent successful contact; treated as absent.
    Unreachable,
}

/// Thresholds and cooldowns governing automatic scaling decisions.
#[derive(Debug)]
pub struct ScalingPolicy {
    /// Utilization level that triggers scale-up when sustained.
    pub scale_up_threshold: f64,
    /// Utilization level that triggers scale-down when sustained.
    pub scale_down_threshold: f64,
    /// Minimum time between consecutive scale-up actions.
    pub scale_up_cooldown: chrono::Duration,
    /// Minimum time between consecutive scale-down actions.
    pub scale_down_cooldown: chrono::Duration,
    /// Lower bound on instance count enforced by the policy.
    pub min_instances: u32,
    /// Upper bound on instance count enforced by the policy.
    pub max_instances: u32,
    /// Multiplier applied when computing the next target instance count.
    pub scale_factor: f64,
}

// Supporting types
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[expect(dead_code, reason = "deserialized from JSON at runtime")]
struct NodeInfo {
    node_id: String,
    region: Option<String>,
    zone: Option<String>,
    capabilities: Vec<String>,
    capacity: u32,
    current_load: u32,
    metadata: HashMap<String, String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[expect(dead_code, reason = "deserialized from JSON at runtime")]
struct JoinRequest {
    node_id: String,
    endpoint: String,
    region: Option<String>,
    zone: Option<String>,
    capabilities: Vec<String>,
    capacity: u32,
}

/// Point-in-time summary of federation membership and load for observability.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FederationStats {
    /// This node's identifier.
    pub node_id: String,
    /// Identifier of the logical federation this node belongs to.
    pub federation_id: String,
    /// Current lifecycle state of the federation from this node's view.
    pub status: FederationStatus,
    /// Number of Squirrel instances managed locally.
    pub local_instances: u32,
    /// Count of known peer nodes participating in the federation.
    pub federation_nodes: u32,
    /// Sum of advertised capacity across tracked members.
    pub total_capacity: u32,
    /// Blended utilization metric in the 0–1 range.
    pub current_utilization: f64,
    /// Whether this node currently holds the leader role.
    pub is_leader: bool,
}

impl Default for FederationConfig {
    fn default() -> Self {
        Self {
            node_id: format!("node-{}", uuid::Uuid::new_v4()),
            port: 8080,
            federation_discovery_urls: Vec::new(),
            auto_scaling_enabled: true,
            min_instances: 1,
            max_instances: 10,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.3,
            health_check_interval: chrono::Duration::seconds(30),
            federation_timeout: chrono::Duration::seconds(60),
            federation_enabled: false,
            region: None,
            zone: None,
            max_local_instances: 10,
            scaling_check_interval: chrono::Duration::seconds(60),
            topology: FederationTopology::Star,
            federation_port: 8090,
        }
    }
}
