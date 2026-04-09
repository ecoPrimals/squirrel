// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Data types for [`super::ContextState`] and related context management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Active session context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Unique session identifier.
    pub session_id: String,
    /// Optional user identifier.
    pub user_id: Option<String>,
    /// Context key-value data.
    pub context_data: HashMap<String, serde_json::Value>,
    /// When the session was created.
    pub created_at: DateTime<Utc>,
    /// When the session was last accessed.
    pub last_accessed: DateTime<Utc>,
    /// Number of access operations.
    pub access_count: u64,
    /// Type of context (e.g., "conversation", "workflow").
    pub context_type: String,
    /// Tags for categorization.
    pub tags: Vec<String>,
    /// Related session IDs.
    pub related_sessions: Vec<String>,
}

/// Persistent context for long-term storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentContext {
    /// Unique context identifier.
    pub context_id: String,
    /// Human-readable context name.
    pub context_name: String,
    /// Context key-value data.
    pub context_data: HashMap<String, serde_json::Value>,
    /// When the context was created.
    pub created_at: DateTime<Utc>,
    /// When the context was last updated.
    pub last_updated: DateTime<Utc>,
    /// Version number.
    pub version: u32,
    /// Type of context.
    pub context_type: String,
    /// Retention policy for this context.
    pub retention_policy: RetentionPolicy,
    /// Access permissions.
    pub access_permissions: AccessPermissions,
}

/// Context analytics for insights and optimization
#[derive(Debug, Clone)]
pub struct ContextAnalytics {
    /// Usage patterns by pattern ID.
    pub usage_patterns: HashMap<String, UsagePattern>,
    /// Access analytics.
    pub access_analytics: AccessAnalytics,
    /// Performance metrics.
    pub performance_metrics: ContextPerformanceMetrics,
    /// AI-generated recommendations.
    pub recommendations: Vec<ContextRecommendation>,
}

/// State versioning for context history
#[derive(Debug, Clone)]
pub struct StateVersioning {
    /// Version history by context ID.
    pub version_history: HashMap<String, Vec<ContextVersion>>,
    /// Versioning policies.
    pub versioning_policies: Vec<VersioningPolicy>,
    /// Rollback capabilities.
    pub rollback_capabilities: RollbackCapabilities,
}

/// Context sharing across ecosystem
#[derive(Debug, Clone)]
pub struct ContextSharing {
    /// Shared contexts by ID.
    pub shared_contexts: HashMap<String, SharedContext>,
    /// Sharing policies.
    pub sharing_policies: Vec<SharingPolicy>,
    /// Cross-primal sharing configuration.
    pub cross_primal_sharing: CrossPrimalSharing,
}

/// Usage pattern analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePattern {
    /// Unique pattern identifier.
    pub pattern_id: String,
    /// Type of context.
    pub context_type: String,
    /// Access frequency (accesses per time unit).
    pub access_frequency: f64,
    /// Peak usage time windows.
    pub peak_usage_times: Vec<String>,
    /// Most common operations.
    pub common_operations: Vec<String>,
    /// User segment identifiers.
    pub user_segments: Vec<String>,
}

/// Access analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessAnalytics {
    /// Total number of context accesses.
    pub total_accesses: u64,
    /// Number of unique sessions.
    pub unique_sessions: u64,
    /// Average session duration.
    pub average_session_duration: Duration,
    /// Cache/context hit rate (0.0–1.0).
    pub context_hit_rate: f64,
    /// Performance scores by metric name.
    pub performance_scores: HashMap<String, f64>,
}

/// Context performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPerformanceMetrics {
    /// Average retrieval time in seconds.
    pub retrieval_time: f64,
    /// Storage efficiency (0.0–1.0).
    pub storage_efficiency: f64,
    /// Memory usage (0.0–1.0).
    pub memory_usage: f64,
    /// Cache hit rate (0.0–1.0).
    pub cache_hit_rate: f64,
    /// Data consistency score (0.0–1.0).
    pub consistency_score: f64,
}

/// Context recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRecommendation {
    /// Unique recommendation identifier.
    pub recommendation_id: String,
    /// Type of recommendation.
    pub recommendation_type: String,
    /// Target context ID.
    pub context_id: String,
    /// Recommendation description.
    pub description: String,
    /// Confidence score (0.0–1.0).
    pub confidence: f64,
    /// Implementation priority (e.g., "high", "low").
    pub implementation_priority: String,
}

/// Context version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextVersion {
    /// Unique version identifier.
    pub version_id: String,
    /// Version number.
    pub version_number: u32,
    /// Snapshot of context data at this version.
    pub context_snapshot: HashMap<String, serde_json::Value>,
    /// When this version was created.
    pub created_at: DateTime<Utc>,
    /// Description of changes.
    pub change_description: String,
    /// Type of change (e.g., "create", "update").
    pub change_type: String,
}

/// Versioning policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersioningPolicy {
    /// Unique policy identifier.
    pub policy_id: String,
    /// Context type this policy applies to.
    pub context_type: String,
    /// Maximum versions to retain.
    pub max_versions: u32,
    /// How long to retain versions.
    pub retention_duration: Duration,
    /// Events that trigger auto-versioning.
    pub auto_versioning_triggers: Vec<String>,
}

/// Rollback capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackCapabilities {
    /// Operations that support rollback.
    pub supported_operations: Vec<String>,
    /// Maximum time window for rollback.
    pub rollback_time_limit: Duration,
    /// Whether verification is required before rollback.
    pub verification_required: bool,
    /// Available rollback strategies.
    pub rollback_strategies: Vec<String>,
}

/// Shared context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedContext {
    /// Unique shared context identifier.
    pub shared_context_id: String,
    /// ID of the original context.
    pub original_context_id: String,
    /// IDs of entities this is shared with.
    pub shared_with: Vec<String>,
    /// Permissions for the share.
    pub sharing_permissions: SharingPermissions,
    /// When the share was created.
    pub shared_at: DateTime<Utc>,
    /// Optional expiry time.
    pub expiry: Option<DateTime<Utc>>,
}

/// Sharing policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingPolicy {
    /// Unique policy identifier.
    pub policy_id: String,
    /// Policy name.
    pub policy_name: String,
    /// Allowed sharing targets.
    pub allowed_targets: Vec<String>,
    /// Data filters to apply.
    pub data_filters: Vec<DataFilter>,
    /// Time-based constraints.
    pub time_constraints: Vec<TimeConstraint>,
    /// Whether approval is required for sharing.
    pub approval_required: bool,
}

/// Cross-primal sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPrimalSharing {
    /// Active shares by share ID.
    pub active_shares: HashMap<String, PrimalShare>,
    /// Sharing agreements by agreement ID.
    pub sharing_agreements: HashMap<String, SharingAgreement>,
    /// Trust levels by primal ID (0.0–1.0).
    pub trust_levels: HashMap<String, f64>,
}

/// Retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// How long to retain data.
    pub retention_duration: Duration,
    /// Cleanup strategy (e.g., "archive", "delete").
    pub cleanup_strategy: String,
    /// Rules for archival.
    pub archival_rules: Vec<ArchivalRule>,
    /// Whether deletion is protected.
    pub deletion_protection: bool,
}

/// Access permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPermissions {
    /// Entities with read permission.
    pub read_permissions: Vec<String>,
    /// Entities with write permission.
    pub write_permissions: Vec<String>,
    /// Entities with admin permission.
    pub admin_permissions: Vec<String>,
    /// Time-based access rules.
    pub time_based_access: Vec<TimeBasedAccess>,
}

/// Sharing permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingPermissions {
    /// Whether share is read-only.
    pub read_only: bool,
    /// Whether modification is allowed.
    pub modification_allowed: bool,
    /// Whether further sharing is allowed.
    pub further_sharing_allowed: bool,
    /// Whether expiry is enforced.
    pub expiry_enforcement: bool,
}

/// Data filter for sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFilter {
    /// Unique filter identifier.
    pub filter_id: String,
    /// Type of filter.
    pub filter_type: String,
    /// Filter rule expressions.
    pub filter_rules: Vec<String>,
    /// Fields to include.
    pub include_fields: Vec<String>,
    /// Fields to exclude.
    pub exclude_fields: Vec<String>,
}

/// Time constraint for sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConstraint {
    /// Unique constraint identifier.
    pub constraint_id: String,
    /// Optional start time.
    pub start_time: Option<DateTime<Utc>>,
    /// Optional end time.
    pub end_time: Option<DateTime<Utc>>,
    /// Allowed days (e.g., "mon", "tue").
    pub allowed_days: Vec<String>,
    /// Allowed hours (0–23).
    pub allowed_hours: Vec<u8>,
}

/// Primal share information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalShare {
    /// Unique share identifier.
    pub share_id: String,
    /// Target primal ID.
    pub target_primal: String,
    /// Context IDs being shared.
    pub shared_contexts: Vec<String>,
    /// Sharing level (e.g., "full", "summary").
    pub sharing_level: String,
    /// When the share was established.
    pub established_at: DateTime<Utc>,
}

/// Sharing agreement between primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingAgreement {
    /// Unique agreement identifier.
    pub agreement_id: String,
    /// Primal IDs that are parties to the agreement.
    pub parties: Vec<String>,
    /// Agreement terms as key-value pairs.
    pub agreement_terms: HashMap<String, String>,
    /// Data categories covered by the agreement.
    pub data_categories: Vec<String>,
    /// Security requirements.
    pub security_requirements: Vec<String>,
}

/// Archival rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivalRule {
    /// Unique rule identifier.
    pub rule_id: String,
    /// Condition that triggers archival.
    pub condition: String,
    /// Archive storage location.
    pub archive_location: String,
    /// Whether compression is enabled.
    pub compression_enabled: bool,
}

/// Time-based access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBasedAccess {
    /// Unique access rule identifier.
    pub access_id: String,
    /// User or group identifier.
    pub user_or_group: String,
    /// When access becomes valid.
    pub valid_from: DateTime<Utc>,
    /// When access expires.
    pub valid_until: DateTime<Utc>,
    /// Granted permissions.
    pub permissions: Vec<String>,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Unique result identifier
    pub result_id: String,
    /// Type of result (e.g., "context", "session")
    pub result_type: String,
    /// Relevance score from 0.0 to 1.0
    pub relevance_score: f64,
    /// Snippet of matching context
    pub context_snippet: String,
}

/// Context analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAnalysis {
    /// Session identifier
    pub session_id: String,
    /// Data complexity score (0.0 to 1.0)
    pub data_complexity: f64,
    /// Usage frequency score
    pub usage_frequency: f64,
    /// Recency score (higher for more recent access)
    pub recency_score: f64,
    /// Relationship strength score
    pub relationship_strength: f64,
}
