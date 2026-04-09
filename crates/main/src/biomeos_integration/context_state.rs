// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! # Context State Management for biomeOS
//!
//! This module provides context state management capabilities for maintaining
//! and coordinating context across sessions and the biomeOS ecosystem.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info};

pub use super::context_state_types::*;
use super::{ContextStateRequest, ContextStateResponse};
use crate::error::PrimalError;

/// Context state management system
#[derive(Debug, Clone)]
pub struct ContextState {
    /// Active sessions by session ID.
    pub active_sessions: HashMap<String, SessionContext>,
    /// Persisted contexts by context ID.
    pub persistent_contexts: HashMap<String, PersistentContext>,
    /// Analytics and usage insights.
    pub context_analytics: ContextAnalytics,
    /// Version history and rollback.
    pub state_versioning: StateVersioning,
    /// Cross-session and cross-primal sharing.
    pub context_sharing: ContextSharing,
}

impl ContextState {
    /// Create a new context state management system with comprehensive tracking
    ///
    /// Initializes a context state manager with:
    /// - Empty active sessions `HashMap` for session tracking
    /// - Empty persistent contexts `HashMap` for long-term storage
    /// - Context analytics system for usage pattern analysis
    /// - State versioning system for context history management
    /// - Context sharing system for cross-session data exchange
    ///
    /// # Returns
    ///
    /// A new `ContextState` instance ready for ecosystem context management
    #[must_use]
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
        debug!("Handling context state request: {}", request.request_id);

        Ok(ContextStateResponse {
            request_id: request.request_id,
            session_id: request.session_id,
            context_state: {
                let mut state = HashMap::new();
                state.insert(
                    "state".to_string(),
                    serde_json::Value::String("active".to_string()),
                );
                state.insert(
                    "session_count".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(self.active_sessions.len())),
                );
                state
            },
            recommendations: vec![
                "Monitor session activity".to_string(),
                "Optimize context persistence".to_string(),
            ],
            related_contexts: vec![],
        })
    }

    /// Health check
    pub async fn health_check(&self) -> Result<(), PrimalError> {
        debug!("Performing context state health check");

        if self.active_sessions.is_empty() && self.persistent_contexts.is_empty() {
            return Err(PrimalError::General(
                "Context state not initialized".to_string(),
            ));
        }

        // Check active sessions
        if self.active_sessions.is_empty() {
            debug!("No active sessions");
        }

        Ok(())
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

        info!("Created session context: {}", session_context.session_id);
        self.active_sessions.insert(session_id, session_context);
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
    #[must_use]
    pub fn get_active_sessions(&self) -> u32 {
        self.active_sessions.len() as u32
    }

    /// Get managed states count
    #[must_use]
    pub fn get_managed_states(&self) -> u32 {
        (self.active_sessions.len() + self.persistent_contexts.len()) as u32
    }

    /// Search for contexts across active sessions and persistent storage
    pub async fn search_context_data(&self, query: &str) -> Result<Vec<SearchResult>, PrimalError> {
        self.search_contexts(query).await
    }

    /// Analyze a specific session context for insights and metrics
    pub async fn analyze_session(&self, session_id: &str) -> Result<ContextAnalysis, PrimalError> {
        self.analyze_session_context(session_id).await
    }

    /// Generate optimization recommendations for a session
    pub async fn get_session_recommendations(
        &self,
        session_id: &str,
    ) -> Result<Vec<String>, PrimalError> {
        self.generate_context_recommendations(session_id).await
    }

    /// Get analytics data for all sessions
    pub async fn get_context_analytics(
        &self,
    ) -> Result<HashMap<String, ContextAnalysis>, PrimalError> {
        let mut analytics = HashMap::new();

        for session_id in self.active_sessions.keys() {
            if let Ok(analysis) = self.analyze_session_context(session_id).await {
                analytics.insert(session_id.clone(), analysis);
            }
        }

        Ok(analytics)
    }

    /// Search and analyze contexts with combined results
    pub async fn search_and_analyze(
        &self,
        query: &str,
    ) -> Result<Vec<(SearchResult, Option<ContextAnalysis>)>, PrimalError> {
        let search_results = self.search_contexts(query).await?;
        let mut combined_results = Vec::new();

        for result in search_results {
            let analysis = if result.result_type == "active_session" {
                self.analyze_session_context(&result.result_id).await.ok()
            } else {
                None
            };
            combined_results.push((result, analysis));
        }

        Ok(combined_results)
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
        _session: &SessionContext,
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

impl ContextAnalytics {
    fn new() -> Self {
        Self {
            usage_patterns: HashMap::new(),
            access_analytics: AccessAnalytics {
                total_accesses: 0,
                unique_sessions: 0,
                average_session_duration: Duration::from_secs(
                    std::env::var("AVERAGE_SESSION_DURATION_SECS")
                        .ok()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(300),
                ),
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
        debug!("Initializing context analytics — resetting counters");
        self.access_analytics.total_accesses = 0;
        self.access_analytics.unique_sessions = 0;
        self.recommendations.clear();
        Ok(())
    }

    async fn update_analytics(&self) -> Result<(), PrimalError> {
        debug!(
            total_accesses = self.access_analytics.total_accesses,
            unique_sessions = self.access_analytics.unique_sessions,
            patterns = self.usage_patterns.len(),
            "Analytics snapshot"
        );
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), PrimalError> {
        debug!(
            total_accesses = self.access_analytics.total_accesses,
            "Shutting down context analytics — final metrics"
        );
        self.recommendations.clear();
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
        debug!(
            policies = self.versioning_policies.len(),
            "Initializing state versioning"
        );
        self.version_history.clear();
        Ok(())
    }

    async fn cleanup_old_versions(&self) -> Result<(), PrimalError> {
        let total_versions: usize = self.version_history.values().map(Vec::len).sum();
        debug!(
            total_versions,
            contexts = self.version_history.len(),
            "Cleanup old versions — audit complete"
        );
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
#[path = "context_state_tests.rs"]
mod tests;
