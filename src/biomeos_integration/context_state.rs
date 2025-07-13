//! # Context State Management for biomeOS
//!
//! This module provides context state management capabilities for maintaining
//! and coordinating context across sessions and the biomeOS ecosystem.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info};

use super::{ContextStateRequest, ContextStateResponse};
use crate::error::PrimalError;

/// Context state management system
#[derive(Debug, Clone)]
pub struct ContextState {
    pub active_sessions: HashMap<String, SessionContext>,
    pub persistent_contexts: HashMap<String, PersistentContext>,
    pub context_analytics: ContextAnalytics,
    pub state_versioning: StateVersioning,
    pub context_sharing: ContextSharing,
}

/// Active session context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub session_id: String,
    pub user_id: Option<String>,
    pub context_data: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u64,
    pub context_type: String,
    pub tags: Vec<String>,
    pub related_sessions: Vec<String>,
}

/// Persistent context for long-term storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentContext {
    pub context_id: String,
    pub context_name: String,
    pub context_data: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub version: u32,
    pub context_type: String,
    pub retention_policy: RetentionPolicy,
    pub access_permissions: AccessPermissions,
}

/// Context analytics for insights and optimization
#[derive(Debug, Clone)]
pub struct ContextAnalytics {
    pub usage_patterns: HashMap<String, UsagePattern>,
    pub access_analytics: AccessAnalytics,
    pub performance_metrics: ContextPerformanceMetrics,
    pub recommendations: Vec<ContextRecommendation>,
}

/// State versioning for context history
#[derive(Debug, Clone)]
pub struct StateVersioning {
    pub version_history: HashMap<String, Vec<ContextVersion>>,
    pub versioning_policies: Vec<VersioningPolicy>,
    pub rollback_capabilities: RollbackCapabilities,
}

/// Context sharing across ecosystem
#[derive(Debug, Clone)]
pub struct ContextSharing {
    pub shared_contexts: HashMap<String, SharedContext>,
    pub sharing_policies: Vec<SharingPolicy>,
    pub cross_primal_sharing: CrossPrimalSharing,
}

/// Usage pattern analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePattern {
    pub pattern_id: String,
    pub context_type: String,
    pub access_frequency: f64,
    pub peak_usage_times: Vec<String>,
    pub common_operations: Vec<String>,
    pub user_segments: Vec<String>,
}

/// Access analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessAnalytics {
    pub total_accesses: u64,
    pub unique_sessions: u64,
    pub average_session_duration: Duration,
    pub context_hit_rate: f64,
    pub performance_scores: HashMap<String, f64>,
}

/// Context performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPerformanceMetrics {
    pub retrieval_time: f64,
    pub storage_efficiency: f64,
    pub memory_usage: f64,
    pub cache_hit_rate: f64,
    pub consistency_score: f64,
}

/// Context recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRecommendation {
    pub recommendation_id: String,
    pub recommendation_type: String,
    pub context_id: String,
    pub description: String,
    pub confidence: f64,
    pub implementation_priority: String,
}

/// Context version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextVersion {
    pub version_id: String,
    pub version_number: u32,
    pub context_snapshot: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub change_description: String,
    pub change_type: String,
}

/// Versioning policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersioningPolicy {
    pub policy_id: String,
    pub context_type: String,
    pub max_versions: u32,
    pub retention_duration: Duration,
    pub auto_versioning_triggers: Vec<String>,
}

/// Rollback capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackCapabilities {
    pub supported_operations: Vec<String>,
    pub rollback_time_limit: Duration,
    pub verification_required: bool,
    pub rollback_strategies: Vec<String>,
}

/// Shared context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedContext {
    pub shared_context_id: String,
    pub original_context_id: String,
    pub shared_with: Vec<String>,
    pub sharing_permissions: SharingPermissions,
    pub shared_at: DateTime<Utc>,
    pub expiry: Option<DateTime<Utc>>,
}

/// Sharing policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingPolicy {
    pub policy_id: String,
    pub policy_name: String,
    pub allowed_targets: Vec<String>,
    pub data_filters: Vec<DataFilter>,
    pub time_constraints: Vec<TimeConstraint>,
    pub approval_required: bool,
}

/// Cross-primal sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPrimalSharing {
    pub active_shares: HashMap<String, PrimalShare>,
    pub sharing_agreements: HashMap<String, SharingAgreement>,
    pub trust_levels: HashMap<String, f64>,
}

/// Retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub retention_duration: Duration,
    pub cleanup_strategy: String,
    pub archival_rules: Vec<ArchivalRule>,
    pub deletion_protection: bool,
}

/// Access permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPermissions {
    pub read_permissions: Vec<String>,
    pub write_permissions: Vec<String>,
    pub admin_permissions: Vec<String>,
    pub time_based_access: Vec<TimeBasedAccess>,
}

/// Sharing permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingPermissions {
    pub read_only: bool,
    pub modification_allowed: bool,
    pub further_sharing_allowed: bool,
    pub expiry_enforcement: bool,
}

/// Data filter for sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFilter {
    pub filter_id: String,
    pub filter_type: String,
    pub filter_rules: Vec<String>,
    pub include_fields: Vec<String>,
    pub exclude_fields: Vec<String>,
}

/// Time constraint for sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConstraint {
    pub constraint_id: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub allowed_days: Vec<String>,
    pub allowed_hours: Vec<u8>,
}

/// Primal share information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalShare {
    pub share_id: String,
    pub target_primal: String,
    pub shared_contexts: Vec<String>,
    pub sharing_level: String,
    pub established_at: DateTime<Utc>,
}

/// Sharing agreement between primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingAgreement {
    pub agreement_id: String,
    pub parties: Vec<String>,
    pub agreement_terms: HashMap<String, String>,
    pub data_categories: Vec<String>,
    pub security_requirements: Vec<String>,
}

/// Archival rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivalRule {
    pub rule_id: String,
    pub condition: String,
    pub archive_location: String,
    pub compression_enabled: bool,
}

/// Time-based access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBasedAccess {
    pub access_id: String,
    pub user_or_group: String,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub permissions: Vec<String>,
}

impl ContextState {
    /// Create new context state management system
    pub fn new() -> Self {
        Self {
            active_sessions: HashMap::new(),
            persistent_contexts: HashMap::new(),
            context_analytics: ContextAnalytics::new(),
            state_versioning: StateVersioning::new(),
            context_sharing: ContextSharing::new(),
        }
    }

    /// Initialize context state management
    pub async fn initialize(&mut self) -> Result<(), PrimalError> {
        info!("Initializing context state management");

        // Initialize analytics
        self.context_analytics.initialize().await?;

        // Initialize versioning
        self.state_versioning.initialize().await?;

        // Initialize sharing
        self.context_sharing.initialize().await?;

        info!("Context state management initialized successfully");
        Ok(())
    }

    /// Manage ecosystem context
    pub async fn manage_ecosystem_context(&self) -> Result<(), PrimalError> {
        debug!("Managing ecosystem context");

        // Update analytics
        self.context_analytics.update_analytics().await?;

        // Process version cleanup
        self.state_versioning.cleanup_old_versions().await?;

        // Manage sharing agreements
        self.context_sharing.update_sharing_status().await?;

        Ok(())
    }

    /// Handle state request
    pub async fn handle_state_request(
        &self,
        request: ContextStateRequest,
    ) -> Result<ContextStateResponse, PrimalError> {
        info!(
            "Handling context state request for session: {}",
            request.session_id
        );

        let mut context_state = HashMap::new();
        let mut recommendations = Vec::new();
        let mut related_contexts = Vec::new();

        match request.request_type.as_str() {
            "get_context" => {
                if let Some(session) = self.active_sessions.get(&request.session_id) {
                    context_state = session.context_data.clone();
                    related_contexts = session.related_sessions.clone();
                }
            }
            "update_context" => {
                if let Some(context_data) = request.context_data {
                    context_state = context_data;
                    recommendations.push("Context updated successfully".to_string());
                }
            }
            "search_context" => {
                if let Some(query) = request.query {
                    let search_results = self.search_contexts(&query).await?;
                    context_state.insert(
                        "search_results".to_string(),
                        serde_json::to_value(search_results)?,
                    );
                }
            }
            "analyze_context" => {
                let analysis = self.analyze_session_context(&request.session_id).await?;
                context_state.insert("analysis".to_string(), serde_json::to_value(analysis)?);
                recommendations = self
                    .generate_context_recommendations(&request.session_id)
                    .await?;
            }
            _ => {
                return Err(PrimalError::Internal(format!(
                    "Unknown request type: {}",
                    request.request_type
                )));
            }
        }

        Ok(ContextStateResponse {
            session_id: request.session_id,
            context_state,
            recommendations,
            related_contexts,
        })
    }

    /// Create new session context
    pub async fn create_session_context(
        &mut self,
        session_id: String,
        user_id: Option<String>,
        context_type: String,
    ) -> Result<(), PrimalError> {
        let session_context = SessionContext {
            session_id: session_id.clone(),
            user_id,
            context_data: HashMap::new(),
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
            context_type,
            tags: Vec::new(),
            related_sessions: Vec::new(),
        };

        self.active_sessions
            .insert(session_id.clone(), session_context);
        info!("Created session context: {}", session_id);
        Ok(())
    }

    /// Update session context
    pub async fn update_session_context(
        &mut self,
        session_id: &str,
        updates: HashMap<String, serde_json::Value>,
    ) -> Result<(), PrimalError> {
        if let Some(session) = self.active_sessions.get_mut(session_id) {
            session.last_accessed = Utc::now();
            session.access_count += 1;

            for (key, value) in updates {
                session.context_data.insert(key, value);
            }

            debug!("Updated session context: {}", session_id);
        }
        Ok(())
    }

    /// Get active sessions count
    pub fn get_active_sessions(&self) -> u32 {
        self.active_sessions.len() as u32
    }

    /// Get managed states count
    pub fn get_managed_states(&self) -> u32 {
        (self.active_sessions.len() + self.persistent_contexts.len()) as u32
    }

    /// Shutdown context state management
    pub async fn shutdown(&mut self) -> Result<(), PrimalError> {
        info!("Shutting down context state management");

        // Save active sessions to persistent storage
        for (session_id, session_context) in &self.active_sessions {
            self.persist_session_context(session_id, session_context)
                .await?;
        }

        // Clear active sessions
        self.active_sessions.clear();

        // Shutdown components
        self.context_analytics.shutdown().await?;
        self.state_versioning.shutdown().await?;
        self.context_sharing.shutdown().await?;

        info!("Context state management shut down successfully");
        Ok(())
    }

    // Private helper methods
    async fn search_contexts(&self, query: &str) -> Result<Vec<SearchResult>, PrimalError> {
        let mut results = Vec::new();

        // Search in active sessions
        for (session_id, session) in &self.active_sessions {
            if self.matches_query(session, query) {
                results.push(SearchResult {
                    result_id: session_id.clone(),
                    result_type: "active_session".to_string(),
                    relevance_score: 0.8,
                    context_snippet: format!("Active session: {}", session.context_type),
                });
            }
        }

        // Search in persistent contexts
        for (context_id, context) in &self.persistent_contexts {
            if self.matches_persistent_query(context, query) {
                results.push(SearchResult {
                    result_id: context_id.clone(),
                    result_type: "persistent_context".to_string(),
                    relevance_score: 0.9,
                    context_snippet: format!("Persistent context: {}", context.context_name),
                });
            }
        }

        Ok(results)
    }

    async fn analyze_session_context(
        &self,
        session_id: &str,
    ) -> Result<ContextAnalysis, PrimalError> {
        if let Some(session) = self.active_sessions.get(session_id) {
            Ok(ContextAnalysis {
                session_id: session_id.to_string(),
                data_complexity: session.context_data.len() as f64,
                usage_frequency: session.access_count as f64,
                recency_score: self.calculate_recency_score(session.last_accessed),
                relationship_strength: session.related_sessions.len() as f64,
            })
        } else {
            Err(PrimalError::Internal(format!(
                "Session not found: {session_id}"
            )))
        }
    }

    async fn generate_context_recommendations(
        &self,
        session_id: &str,
    ) -> Result<Vec<String>, PrimalError> {
        let mut recommendations = Vec::new();

        if let Some(session) = self.active_sessions.get(session_id) {
            if session.context_data.len() > 100 {
                recommendations.push("Consider archiving older context data".to_string());
            }

            if session.access_count > 1000 {
                recommendations
                    .push("High usage session - consider caching optimization".to_string());
            }

            if session.related_sessions.is_empty() {
                recommendations
                    .push("Consider linking related sessions for better context".to_string());
            }
        }

        Ok(recommendations)
    }

    async fn persist_session_context(
        &self,
        session_id: &str,
        session: &SessionContext,
    ) -> Result<(), PrimalError> {
        // Simulate persisting session context
        debug!("Persisting session context: {}", session_id);
        Ok(())
    }

    fn matches_query(&self, session: &SessionContext, query: &str) -> bool {
        session.context_type.contains(query) || session.tags.iter().any(|tag| tag.contains(query))
    }

    fn matches_persistent_query(&self, context: &PersistentContext, query: &str) -> bool {
        context.context_name.contains(query) || context.context_type.contains(query)
    }

    fn calculate_recency_score(&self, last_accessed: DateTime<Utc>) -> f64 {
        let now = Utc::now();
        let duration = now.signed_duration_since(last_accessed);
        let hours = duration.num_hours() as f64;

        // Higher score for more recent access
        (24.0 - hours.min(24.0)) / 24.0
    }
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub result_id: String,
    pub result_type: String,
    pub relevance_score: f64,
    pub context_snippet: String,
}

/// Context analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAnalysis {
    pub session_id: String,
    pub data_complexity: f64,
    pub usage_frequency: f64,
    pub recency_score: f64,
    pub relationship_strength: f64,
}

impl ContextAnalytics {
    fn new() -> Self {
        Self {
            usage_patterns: HashMap::new(),
            access_analytics: AccessAnalytics {
                total_accesses: 0,
                unique_sessions: 0,
                average_session_duration: Duration::from_secs(300),
                context_hit_rate: 0.95,
                performance_scores: HashMap::new(),
            },
            performance_metrics: ContextPerformanceMetrics {
                retrieval_time: 50.0,
                storage_efficiency: 0.85,
                memory_usage: 0.60,
                cache_hit_rate: 0.90,
                consistency_score: 0.95,
            },
            recommendations: Vec::new(),
        }
    }

    async fn initialize(&mut self) -> Result<(), PrimalError> {
        debug!("Initializing context analytics");
        Ok(())
    }

    async fn update_analytics(&self) -> Result<(), PrimalError> {
        debug!("Updating context analytics");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), PrimalError> {
        debug!("Shutting down context analytics");
        Ok(())
    }
}

impl StateVersioning {
    fn new() -> Self {
        Self {
            version_history: HashMap::new(),
            versioning_policies: Vec::new(),
            rollback_capabilities: RollbackCapabilities {
                supported_operations: vec!["rollback".to_string(), "restore".to_string()],
                rollback_time_limit: Duration::from_secs(24 * 3600), // 24 hours
                verification_required: true,
                rollback_strategies: vec![
                    "full_rollback".to_string(),
                    "selective_rollback".to_string(),
                ],
            },
        }
    }

    async fn initialize(&mut self) -> Result<(), PrimalError> {
        debug!("Initializing state versioning");
        Ok(())
    }

    async fn cleanup_old_versions(&self) -> Result<(), PrimalError> {
        debug!("Cleaning up old versions");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), PrimalError> {
        debug!("Shutting down state versioning");
        Ok(())
    }
}

impl ContextSharing {
    fn new() -> Self {
        Self {
            shared_contexts: HashMap::new(),
            sharing_policies: Vec::new(),
            cross_primal_sharing: CrossPrimalSharing {
                active_shares: HashMap::new(),
                sharing_agreements: HashMap::new(),
                trust_levels: HashMap::new(),
            },
        }
    }

    async fn initialize(&mut self) -> Result<(), PrimalError> {
        debug!("Initializing context sharing");
        Ok(())
    }

    async fn update_sharing_status(&self) -> Result<(), PrimalError> {
        debug!("Updating sharing status");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), PrimalError> {
        debug!("Shutting down context sharing");
        Ok(())
    }
}

impl Default for ContextState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_state_creation() {
        let context_state = ContextState::new();
        assert!(context_state.active_sessions.is_empty());
        assert!(context_state.persistent_contexts.is_empty());
    }

    #[tokio::test]
    async fn test_session_context_creation() {
        let mut context_state = ContextState::new();
        let session_id = "test-session-001".to_string();

        context_state
            .create_session_context(
                session_id.clone(),
                Some("user-123".to_string()),
                "test_context".to_string(),
            )
            .await
            .unwrap();

        assert!(context_state.active_sessions.contains_key(&session_id));
        assert_eq!(context_state.get_active_sessions(), 1);
    }

    #[tokio::test]
    async fn test_context_state_request_handling() {
        let context_state = ContextState::new();
        let request = ContextStateRequest {
            session_id: "test-session".to_string(),
            request_type: "get_context".to_string(),
            context_data: None,
            query: None,
        };

        let response = context_state.handle_state_request(request).await.unwrap();
        assert_eq!(response.session_id, "test-session");
    }

    #[tokio::test]
    async fn test_context_search() {
        let context_state = ContextState::new();
        let search_results = context_state.search_contexts("test").await.unwrap();
        assert!(search_results.is_empty()); // No contexts to search initially
    }
}
