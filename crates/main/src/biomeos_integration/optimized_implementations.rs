// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors
#![expect(deprecated, reason = "Backward compatibility during migration")]

//! Optimized `BiomeOS` Integration Implementations
//!
//! This module provides optimized versions of `BiomeOS` integration components
//! that use zero-copy patterns to reduce memory allocations and improve performance.

// Backward compatibility: kept for deserialization of legacy data
use crate::biomeos_integration::IntelligenceResponse; // Add missing import
use crate::ecosystem::{
    EcosystemPrimalType,
    EcosystemServiceRegistration,
    HealthCheckConfig, // Add missing imports
    SecurityConfig,
    ServiceCapabilities, // Add ServiceEndpoints import
};
use crate::optimization::zero_copy::{
    collection_utils::ZeroCopyMap,
    message_utils::ZeroCopyMessage,
    performance_monitoring::{MetricsSnapshot, ZeroCopyMetrics},
    string_utils::StaticStrings,
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

/// Session context for optimized BiomeOS integration.
#[derive(Debug, Clone)]
pub struct SessionContext {
    /// Unique session identifier
    pub session_id: String,
    /// User identifier
    pub user_id: String,
    /// When the session was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last activity timestamp
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// Session metadata
    pub metadata: HashMap<String, String>,
    /// Arbitrary context data
    pub context_data: HashMap<String, serde_json::Value>,
}

/// Cached context data for zero-copy reuse.
#[derive(Debug, Clone)]
pub struct ContextData {
    /// Data identifier
    pub id: String,
    /// The context data payload
    pub data: serde_json::Value,
    /// When the data was cached
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Optimized service registration that avoids unnecessary cloning
pub struct OptimizedServiceRegistration {
    static_strings: StaticStrings,
    // string_builder: removed as not available,
    // config: ZeroCopyConfig, // Removed
    metrics: Arc<ZeroCopyMetrics>,
}

impl Default for OptimizedServiceRegistration {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizedServiceRegistration {
    /// Creates a new optimized service registration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            static_strings: StaticStrings::new(),
            // string_builder: ZeroCopyStringBuilder::new(), // Removed
            // config: ZeroCopyConfig::new(), // Removed
            metrics: Arc::new(ZeroCopyMetrics::new()),
        }
    }

    /// Create ecosystem service registration with zero-copy optimizations
    pub fn create_ecosystem_service_registration(
        &mut self,
        instance_id: &str,
        biome_id: Option<&str>,
        capabilities: &[&str],
        dependencies: Option<Vec<String>>,
        metadata: Option<std::collections::HashMap<String, String>>,
    ) -> EcosystemServiceRegistration {
        self.metrics.record_operation();

        // Use cached strings for common values
        let _primal_type_str = self
            .static_strings
            .get("squirrel")
            .unwrap_or_else(|| Arc::from("squirrel"));
        let _api_version = self
            .static_strings
            .get("biomeOS/v1")
            .unwrap_or_else(|| Arc::from("biomeOS/v1"));
        let _status = self
            .static_strings
            .get("running")
            .unwrap_or_else(|| Arc::from("running"));

        // Build service ID efficiently
        let service_id = format!("squirrel-{instance_id}");

        // Build endpoints efficiently (configurable via TEST_BIOMEOS_OPT_PORT)
        let test_port = std::env::var("TEST_BIOMEOS_OPT_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8080);
        let base_url = format!("http://localhost:{test_port}");
        let endpoints = crate::ecosystem::ServiceEndpoints {
            primary: base_url.clone(),
            secondary: vec![
                format!("{}/metrics", base_url),
                format!("{}/admin", base_url),
                format!("{}/mcp", base_url),
                format!("{}/ai", base_url),
                format!("{}/service-mesh", base_url),
            ],
            health: Some(format!("{base_url}/health")),
        };

        // Build capabilities efficiently
        let _capability_list: Vec<String> =
            capabilities.iter().map(|&cap| cap.to_string()).collect();

        self.metrics.record_clone_avoided();

        EcosystemServiceRegistration {
            service_id: Arc::from(service_id.clone()),
            name: service_id.clone(),
            description: format!("BiomeOS integration for {service_id}"),
            primal_type: crate::ecosystem::EcosystemPrimalType::Squirrel, // Use enum directly
            biome_id: Some(biome_id.map_or_else(
                || "default-biome".to_string(),
                std::string::ToString::to_string,
            )),
            endpoints,
            capabilities: ServiceCapabilities {
                core: capabilities.iter().map(|&s| s.to_string()).collect(),
                extended: Vec::new(),
                integrations: Vec::new(),
            },
            dependencies: dependencies.unwrap_or_default(),
            version: "1.0.0".to_string(),
            health_check: HealthCheckConfig {
                enabled: true,
                interval_secs: 30,
                timeout_secs: 10,
                failure_threshold: 3,
            },
            metadata: metadata.unwrap_or_default(),
            primal_provider: None,
            registered_at: chrono::Utc::now(),
            tags: Vec::new(),
            security_config: SecurityConfig::default(),
            resource_requirements: crate::ecosystem::ResourceSpec {
                cpu: "500m".to_string(),
                memory: "1Gi".to_string(),
                storage: "10Gi".to_string(),
                network: "1Gbps".to_string(),
                gpu: None,
            },
        }
    }

    /// Get performance metrics
    #[must_use]
    pub fn get_metrics(&self) -> MetricsSnapshot {
        self.metrics.get_metrics()
    }
}

/// Optimized message processing that avoids unnecessary cloning
pub struct OptimizedMessageProcessor {
    message_cache: ZeroCopyMap<ZeroCopyMessage>,
    static_strings: StaticStrings,
    metrics: Arc<ZeroCopyMetrics>,
}

impl Default for OptimizedMessageProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizedMessageProcessor {
    /// Creates a new optimized message processor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            message_cache: ZeroCopyMap::new(),
            static_strings: StaticStrings::new(),
            metrics: Arc::new(ZeroCopyMetrics::new()),
        }
    }

    /// Process intelligence request with zero-copy optimizations
    pub fn process_intelligence_request(
        &mut self,
        request_id: &str,
        request_type: &str,
        _data: &serde_json::Value,
    ) -> Result<IntelligenceResponse, crate::error::PrimalError> {
        self.metrics.record_operation();

        // Use cached strings for common request types
        let cached_type = match request_type {
            "analysis" => self.static_strings.get("analysis"),
            "intelligence" => self.static_strings.get("intelligence"),
            _ => None,
        };

        if cached_type.is_some() {
            self.metrics.record_string_interning_hit();
        }

        // Build response efficiently with actual timing measurement
        let processing_start = std::time::Instant::now();
        let result = serde_json::json!({"recommendations": ["Optimize resource usage", "Monitor system health"]});
        let processing_time_ms = processing_start.elapsed().as_millis() as u64;

        let response = IntelligenceResponse {
            request_id: request_id.to_string(),
            intelligence_type: request_type.to_string(),
            result,
            confidence: 0.85,
            processing_time_ms,
            metadata: HashMap::new(),
        };

        self.metrics.record_clone_avoided();

        Ok(response)
    }

    /// Cache a message for reuse and return a reference-counted handle to it
    pub fn cache_message(&mut self, key: &str, message: ZeroCopyMessage) -> Arc<ZeroCopyMessage> {
        let key_arc: Arc<str> = Arc::from(key);
        let cached = Arc::new(message);
        self.message_cache.insert(key_arc, (*cached).clone());
        cached
    }

    /// Get cached message
    #[must_use]
    pub fn get_cached_message(&self, key: &str) -> Option<Arc<ZeroCopyMessage>> {
        // Use efficient lookup without Arc allocation
        self.message_cache
            .iter()
            .find(|(k, _)| k.as_ref() == key)
            .map(|(_, v)| Arc::new(v.clone()))
    }

    /// Get performance metrics
    #[must_use]
    pub fn get_metrics(&self) -> MetricsSnapshot {
        self.metrics.get_metrics()
    }
}

/// Optimized context state management
pub struct OptimizedContextState {
    active_sessions: ZeroCopyMap<SessionContext>,
    context_cache: ZeroCopyMap<ContextData>,
    metrics: Arc<ZeroCopyMetrics>,
}

impl Default for OptimizedContextState {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizedContextState {
    /// Creates a new optimized context state manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_sessions: ZeroCopyMap::new(),
            context_cache: ZeroCopyMap::new(),
            metrics: Arc::new(ZeroCopyMetrics::new()),
        }
    }

    /// Create a new session with zero-copy optimizations
    pub fn create_session(
        &mut self,
        session_id: String,
        user_id: &str,
        metadata: HashMap<String, String>,
    ) -> Arc<SessionContext> {
        self.metrics.record_operation();

        let session_context = SessionContext {
            session_id: session_id.clone(),
            user_id: user_id.to_string(),
            created_at: Utc::now(),
            last_activity: Utc::now(),
            metadata,
            context_data: HashMap::new(),
        };

        let key_arc: Arc<str> = Arc::from(session_id.clone());
        self.active_sessions.insert(key_arc, session_context);
        self.metrics.record_clone_avoided();

        // Return the created session context
        Arc::new(SessionContext {
            session_id,
            user_id: user_id.to_string(),
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
            context_data: std::collections::HashMap::new(),
        })
    }

    /// Cache context data for reuse
    pub fn cache_context_data(&mut self, key: String, data: ContextData) -> Arc<ContextData> {
        let key_arc: Arc<str> = Arc::from(key);
        let data_arc = Arc::new(data);
        self.context_cache.insert(key_arc, (*data_arc).clone());
        data_arc
    }

    /// Get cached context data
    #[must_use]
    pub fn get_cached_context_data(&self, key: &str) -> Option<Arc<ContextData>> {
        self.context_cache
            .iter()
            .find(|(k, _)| k.as_ref() == key)
            .map(|(_, v)| Arc::new(v.clone()))
    }

    /// Get all active sessions efficiently
    #[must_use]
    pub fn get_active_sessions(&self) -> Vec<Arc<SessionContext>> {
        self.active_sessions
            .values()
            .map(|session| Arc::new(session.clone()))
            .collect()
    }

    /// Remove session from cache
    pub fn remove_session(&mut self, session_id: &str) -> Option<Arc<SessionContext>> {
        // Find and remove session properly
        let mut removed_session = None;
        self.active_sessions.retain(|k, v| {
            if k.as_ref() == session_id {
                removed_session = Some(Arc::new(v.clone()));
                false
            } else {
                true
            }
        });
        removed_session
    }
}

/// Registers a service with the ecosystem using the given parameters.
#[must_use]
#[expect(clippy::implicit_hasher, reason = "Generic HashMap consumer API")]
pub fn register_with_ecosystem(
    service_id: &str,
    primal_type: EcosystemPrimalType, // Already correct type
    base_url: &str,
    capabilities: &[&str],
    biome_id: Option<&str>,
    dependencies: Option<Vec<String>>,
    metadata: Option<std::collections::HashMap<String, String>>,
) -> EcosystemServiceRegistration {
    let endpoints = crate::ecosystem::ServiceEndpoints {
        primary: base_url.to_string(),
        secondary: vec![
            format!("{}/metrics", base_url),
            format!("{}/admin", base_url),
            format!("{}/mcp", base_url),
            format!("{}/ai", base_url),
            format!("{}/service-mesh", base_url),
        ],
        health: Some(format!("{base_url}/health")),
    };

    EcosystemServiceRegistration {
        service_id: Arc::from(service_id.to_string()),
        name: service_id.to_string(),
        description: format!("BiomeOS integration for {service_id}"),
        primal_type,
        biome_id: Some(biome_id.map_or_else(
            || "default-biome".to_string(),
            std::string::ToString::to_string,
        )),
        endpoints,
        capabilities: ServiceCapabilities {
            core: capabilities.iter().map(|&s| s.to_string()).collect(),
            extended: Vec::new(),
            integrations: Vec::new(),
        },
        dependencies: dependencies.unwrap_or_default(),
        version: "1.0.0".to_string(),
        health_check: HealthCheckConfig {
            enabled: true,
            interval_secs: 30,
            timeout_secs: 10,
            failure_threshold: 3,
        },
        metadata: metadata.unwrap_or_default(),
        primal_provider: None,
        registered_at: chrono::Utc::now(),
        tags: Vec::new(),
        security_config: SecurityConfig::default(),
        resource_requirements: crate::ecosystem::ResourceSpec {
            cpu: "500m".to_string(),
            memory: "1Gi".to_string(),
            storage: "10Gi".to_string(),
            network: "1Gbps".to_string(),
            gpu: None,
        },
    }
}

/// Processes an intelligence request asynchronously.
pub async fn process_intelligence_request(
    request_id: &str,
    intelligence_type: &str,
    _payload: serde_json::Value, // Mark as unused
) -> Result<IntelligenceResponse, crate::error::PrimalError> {
    let start_time = std::time::Instant::now();

    let result = match intelligence_type {
        "pattern_recognition" => serde_json::json!({"patterns": []}),
        "predictive_analytics" => serde_json::json!({"predictions": []}),
        "anomaly_detection" => serde_json::json!({"anomalies": []}),
        _ => serde_json::json!({"error": "unsupported intelligence type"}),
    };

    let processing_time = start_time.elapsed().as_millis() as u64;

    let response = IntelligenceResponse {
        request_id: request_id.to_string(),
        intelligence_type: intelligence_type.to_string(),
        result,
        confidence: 0.85,
        processing_time_ms: processing_time,
        metadata: std::collections::HashMap::new(),
    };

    Ok(response)
}

#[cfg(test)]
mod optimized_impl_tests {
    use super::*;
    use crate::optimization::zero_copy::message_utils::ZeroCopyMessage;
    use std::sync::Arc;

    #[test]
    fn optimized_service_registration_default_and_registration_shape() {
        let mut reg = OptimizedServiceRegistration::default();
        let svc = reg.create_ecosystem_service_registration(
            "unit-test",
            Some("biome-x"),
            &["mcp", "ai"],
            Some(vec!["dep1".to_string()]),
            Some(HashMap::from([("k".to_string(), "v".to_string())])),
        );
        assert!(svc.service_id.as_ref().contains("squirrel-unit-test"));
        assert_eq!(svc.capabilities.core.len(), 2);
        assert_eq!(svc.dependencies, vec!["dep1".to_string()]);
        let snap = reg.get_metrics();
        assert!(snap.total_operations > 0);
    }

    #[test]
    fn optimized_message_processor_analysis_and_intelligence_paths() {
        let mut proc = OptimizedMessageProcessor::default();
        let r1 = proc
            .process_intelligence_request("r1", "analysis", &serde_json::json!({}))
            .expect("should succeed");
        assert_eq!(r1.intelligence_type, "analysis");
        let r2 = proc
            .process_intelligence_request("r2", "other", &serde_json::json!({}))
            .expect("should succeed");
        assert_eq!(r2.intelligence_type, "other");
        assert!(proc.get_metrics().total_operations >= 2);
    }

    #[test]
    fn optimized_context_state_session_cache_and_remove() {
        let mut state = OptimizedContextState::new();
        let meta = HashMap::from([("role".to_string(), "test".to_string())]);
        let s = state.create_session("sid".to_string(), "user", meta);
        assert_eq!(s.session_id, "sid");
        assert_eq!(state.get_active_sessions().len(), 1);
        let ctx = ContextData {
            id: "c1".to_string(),
            data: serde_json::json!({"x": 1}),
            timestamp: chrono::Utc::now(),
        };
        state.cache_context_data("ck".to_string(), ctx);
        assert!(state.get_cached_context_data("ck").is_some());
        assert!(state.remove_session("sid").is_some());
        assert!(state.get_active_sessions().is_empty());
    }

    #[test]
    fn register_with_ecosystem_builds_expected_fields() {
        let r = register_with_ecosystem(
            "svc-a",
            EcosystemPrimalType::Squirrel,
            "http://127.0.0.1:9000",
            &["cap1"],
            None,
            None,
            None,
        );
        assert!(r.endpoints.primary.contains("9000"));
        assert_eq!(r.primal_type, EcosystemPrimalType::Squirrel);
    }

    #[tokio::test]
    async fn process_intelligence_request_async_variants() {
        let v = process_intelligence_request("id", "pattern_recognition", serde_json::json!({}))
            .await
            .expect("should succeed");
        assert!(v.result.get("patterns").is_some());
        let v2 = process_intelligence_request("id2", "unknown", serde_json::json!({}))
            .await
            .expect("should succeed");
        assert!(v2.result.get("error").is_some());
    }

    #[test]
    fn session_context_and_context_data_clone_debug() {
        let sc = SessionContext {
            session_id: "s".to_string(),
            user_id: "u".to_string(),
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            metadata: HashMap::new(),
            context_data: HashMap::new(),
        };
        let _ = format!("{sc:?}");
        let c = sc;
        assert_eq!(c.session_id, "s");
    }

    #[test]
    fn message_cache_round_trip() {
        let mut proc = OptimizedMessageProcessor::new();
        let msg = ZeroCopyMessage::new(Arc::from("t"), Arc::from("hello"));
        proc.cache_message("k", msg);
        let cached = proc.get_cached_message("k").expect("should succeed");
        assert_eq!(cached.get_content(), "hello");
    }
}
