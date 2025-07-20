//! Zero-copy optimization tests
//!
//! Tests for the zero-copy optimization infrastructure and implementations

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use squirrel::ecosystem::registry::{
    DiscoveredService, EcosystemRegistryConfig, ServiceHealthStatus,
};
use squirrel::ecosystem::{
    EcosystemPrimalType, EcosystemRegistryManager, EcosystemServiceRegistration,
};
use squirrel::error::PrimalError;
use squirrel::monitoring::metrics::MetricsCollector;
use squirrel::primal_provider::SquirrelPrimalProvider;
use squirrel::session::{
    Session, SessionConfig, SessionManagerImpl, SessionMetadata, SessionState,
};
use squirrel::universal::PrimalContext;
use squirrel_mcp_config::EcosystemConfig;

#[tokio::test]
async fn test_session_arc_optimization() {
    // Test that sessions use Arc<Session> for efficient sharing
    let config = SessionConfig::default();
    let session_manager = SessionManagerImpl::new(config);

    let client_info = HashMap::new();

    // Create a session
    let session_id = session_manager
        .create_session(client_info)
        .await
        .expect("Failed to create session");

    // Get the session (should return Arc<Session>)
    let session_arc1 = session_manager
        .get_session(&session_id)
        .await
        .expect("Failed to get session")
        .expect("Session not found");

    let session_arc2 = session_manager
        .get_session(&session_id)
        .await
        .expect("Failed to get session")
        .expect("Session not found");

    // Verify both Arcs point to the same data (cheap clone test)
    assert!(Arc::ptr_eq(&session_arc1, &session_arc2));
    assert_eq!(
        session_arc1.metadata.session_id,
        session_arc2.metadata.session_id
    );
}

#[tokio::test]
async fn test_registry_arc_optimization() {
    // Test that service registrations use Arc for efficient sharing
    let config = EcosystemRegistryConfig::default();
    let metrics = Arc::new(MetricsCollector::new());
    let registry_manager = EcosystemRegistryManager::new(config, metrics);

    let registration = create_test_service_registration();

    // Register service (should store as Arc<EcosystemServiceRegistration>)
    registry_manager
        .register_squirrel_service(registration.clone())
        .await
        .expect("Failed to register service");

    // Get discovered services (should efficiently clone Arc)
    let discovered_services = registry_manager.get_discovered_services().await;

    // Verify the optimization doesn't break functionality
    assert!(!discovered_services.is_empty());
}

#[tokio::test]
async fn test_ai_provider_string_optimization() {
    // Test AI provider selection uses static strings
    let context = PrimalContext::default();
    let config = EcosystemConfig::default();

    let provider = SquirrelPrimalProvider::new(config, context).expect("Failed to create provider");

    // Test multiple provider selections to verify string optimization
    let initial_metrics = provider.get_zero_copy_metrics();

    // Make several AI inference requests that should trigger string optimization
    let ai_request = serde_json::json!({
        "task_type": "text_generation",
        "messages": [{"role": "user", "content": "Hello"}],
        "model": "gpt-4"
    });

    // This should use cached strings for provider selection
    let _response = provider
        .handle_ai_inference_request(ai_request.clone())
        .await
        .expect("AI inference failed");

    let final_metrics = provider.get_zero_copy_metrics();

    // Verify optimization metrics were updated
    assert!(
        final_metrics.get("total_operations").unwrap().as_u64()
            > initial_metrics.get("total_operations").unwrap().as_u64()
    );
    assert!(
        final_metrics
            .get("clone_operations_avoided")
            .unwrap()
            .as_u64()
            >= 1
    );
}

#[tokio::test]
async fn test_zero_copy_metrics_tracking() {
    // Test that zero-copy metrics are properly tracked
    let context = PrimalContext::default();
    let config = EcosystemConfig::default();

    let provider = SquirrelPrimalProvider::new(config, context).expect("Failed to create provider");

    // Reset metrics to start clean
    provider.reset_zero_copy_metrics();

    let initial_metrics = provider.get_zero_copy_metrics();
    assert_eq!(initial_metrics.get("total_operations").unwrap().as_u64(), 0);

    // Perform operations that should trigger metrics
    let ai_requests = vec![
        serde_json::json!({
            "task_type": "code_generation",
            "messages": [{"role": "user", "content": "Write a function"}],
        }),
        serde_json::json!({
            "task_type": "analysis",
            "messages": [{"role": "user", "content": "Analyze this data"}],
        }),
        serde_json::json!({
            "task_type": "local",
            "messages": [{"role": "user", "content": "Local processing"}],
        }),
    ];

    for request in ai_requests {
        let _response = provider
            .handle_ai_inference_request(request)
            .await
            .expect("AI inference failed");
    }

    let final_metrics = provider.get_zero_copy_metrics();

    // Verify metrics show optimization activity
    assert!(final_metrics.get("total_operations").unwrap().as_u64() >= 3);
    assert!(
        final_metrics
            .get("clone_operations_avoided")
            .unwrap()
            .as_u64()
            >= 3
    );
    assert!(final_metrics.get("efficiency_ratio").unwrap().as_f64() > 0.0);
}

#[tokio::test]
async fn test_context_analysis_optimization() {
    // Test that context analysis uses optimized string operations
    let context = PrimalContext::default();
    let config = EcosystemConfig::default();

    let provider = SquirrelPrimalProvider::new(config, context).expect("Failed to create provider");

    provider.reset_zero_copy_metrics();

    let context_request = serde_json::json!({
        "context": {
            "text": "This is a great analysis of AI technology and machine learning algorithms"
        },
        "analysis_type": "topics"
    });

    let _response = provider
        .handle_context_analysis(context_request)
        .await
        .expect("Context analysis failed");

    let metrics = provider.get_zero_copy_metrics();

    // Verify context analysis triggered optimizations
    assert!(metrics.get("total_operations").unwrap().as_u64() >= 1);
}

#[tokio::test]
async fn test_session_management_optimization() {
    // Test that session management operations use zero-copy patterns
    let context = PrimalContext::default();
    let config = EcosystemConfig::default();

    let provider = SquirrelPrimalProvider::new(config, context).expect("Failed to create provider");

    provider.reset_zero_copy_metrics();

    // Test session creation
    let create_request = serde_json::json!({
        "action": "create",
        "user_id": "test-user",
        "session_type": "ai_chat"
    });

    let create_response = provider
        .handle_session_management(create_request)
        .await
        .expect("Session creation failed");

    let session_id = create_response
        .get("session")
        .and_then(|s| s.get("session_id"))
        .and_then(|id| id.as_str())
        .expect("Session ID not found");

    // Test session retrieval
    let get_request = serde_json::json!({
        "action": "get",
        "session_id": session_id
    });

    let _get_response = provider
        .handle_session_management(get_request)
        .await
        .expect("Session retrieval failed");

    let metrics = provider.get_zero_copy_metrics();

    // Verify session operations used optimizations
    assert!(metrics.get("total_operations").unwrap().as_u64() >= 2);
}

#[test]
fn test_discovered_service_arc_sharing() {
    // Test that DiscoveredService uses Arc for efficient sharing
    use chrono::Utc;

    let service = Arc::new(DiscoveredService {
        service_id: "test-service".to_string(),
        primal_type: EcosystemPrimalType::Squirrel,
        endpoint: "http://localhost:8080".to_string(),
        health_endpoint: "http://localhost:8080/health".to_string(),
        api_version: "v1".to_string(),
        capabilities: vec!["mcp".to_string()],
        metadata: HashMap::new(),
        discovered_at: Utc::now(),
        last_health_check: None,
        health_status: ServiceHealthStatus::Unknown,
    });

    // Clone the Arc (should be cheap)
    let service_clone = service.clone();

    // Verify they point to the same data
    assert!(Arc::ptr_eq(&service, &service_clone));
    assert_eq!(service.service_id, service_clone.service_id);
}

#[tokio::test]
async fn test_memory_efficiency_under_load() {
    // Test that zero-copy optimizations maintain efficiency under load
    let context = PrimalContext::default();
    let config = EcosystemConfig::default();

    let provider = SquirrelPrimalProvider::new(config, context).expect("Failed to create provider");

    provider.reset_zero_copy_metrics();

    // Simulate high load with many identical requests
    let request_template = serde_json::json!({
        "task_type": "text_generation",
        "messages": [{"role": "user", "content": "Hello"}],
        "model": "gpt-4"
    });

    let num_requests = 100;
    let mut tasks = Vec::new();

    for _ in 0..num_requests {
        let provider_ref = &provider;
        let request = request_template.clone();
        tasks.push(async move { provider_ref.handle_ai_inference_request(request).await });
    }

    // Execute all requests concurrently
    let results = futures::future::join_all(tasks).await;

    // Verify all requests succeeded
    for result in results {
        result.expect("Request failed under load");
    }

    let metrics = provider.get_zero_copy_metrics();

    // Verify optimizations scaled with load
    assert!(metrics.get("total_operations").unwrap().as_u64() >= num_requests as u64);
    assert!(metrics.get("clone_operations_avoided").unwrap().as_u64() >= num_requests as u64);

    // Verify efficiency ratio remains high under load
    let efficiency = metrics.get("efficiency_ratio").unwrap().as_f64();
    assert!(
        efficiency > 0.8,
        "Efficiency ratio {} too low under load",
        efficiency
    );
}

// Helper function to create test service registration
fn create_test_service_registration() -> EcosystemServiceRegistration {
    use chrono::Utc;
    use squirrel::ecosystem::*;

    EcosystemServiceRegistration {
        service_id: "test-squirrel-service".to_string(),
        primal_type: EcosystemPrimalType::Squirrel,
        name: "Test Squirrel".to_string(),
        description: "Test service".to_string(),
        biome_id: None,
        version: "1.0.0".to_string(),
        capabilities: ServiceCapabilities {
            core: vec!["mcp".to_string()],
            extended: vec![],
            integrations: vec![],
        },
        endpoints: ServiceEndpoints {
            health: "http://localhost:8080/health".to_string(),
            metrics: "http://localhost:8080/metrics".to_string(),
            admin: "http://localhost:8080/admin".to_string(),
            websocket: "ws://localhost:8080/ws".to_string(),
            mcp: "http://localhost:8080/mcp".to_string(),
            ai_coordination: "http://localhost:8080/ai".to_string(),
            service_mesh: "http://localhost:8080/mesh".to_string(),
        },
        dependencies: vec![],
        tags: vec![],
        primal_provider: Some("squirrel".to_string()),
        health_check: HealthCheckConfig {
            enabled: true,
            interval_seconds: 30,
            timeout_seconds: 5,
            failure_threshold: 3,
            success_threshold: 2,
        },
        security_config: SecurityConfig {
            authentication_required: false,
            allowed_origins: vec!["*".to_string()],
            rate_limiting: false,
            max_requests_per_minute: 1000,
        },
        resource_requirements: ResourceSpec {
            min_memory_mb: 256,
            max_memory_mb: 1024,
            min_cpu_cores: 1.0,
            max_cpu_cores: 4.0,
            storage_mb: 100,
            network_bandwidth_mbps: 10.0,
        },
        metadata: HashMap::new(),
        registered_at: Utc::now(),
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn benchmark_session_operations() {
        // Benchmark session operations with and without Arc optimization
        let config = SessionConfig::default();
        let session_manager = SessionManagerImpl::new(config);

        let client_info = HashMap::new();

        // Benchmark session creation
        let start = Instant::now();
        let mut session_ids = Vec::new();

        for i in 0..100 {
            let mut client_info_clone = client_info.clone();
            client_info_clone.insert("batch".to_string(), format!("session-{}", i));

            let session_id = session_manager
                .create_session(client_info_clone)
                .await
                .expect("Failed to create session");
            session_ids.push(session_id);
        }

        let creation_time = start.elapsed();

        // Benchmark session retrieval (should benefit from Arc sharing)
        let start = Instant::now();

        for session_id in &session_ids {
            let _session = session_manager
                .get_session(session_id)
                .await
                .expect("Failed to get session");
        }

        let retrieval_time = start.elapsed();

        println!("Session creation time: {:?}", creation_time);
        println!("Session retrieval time: {:?}", retrieval_time);

        // Retrieval should be significantly faster due to Arc optimization
        assert!(
            retrieval_time < creation_time,
            "Retrieval should be faster than creation due to Arc optimization"
        );
    }

    #[tokio::test]
    async fn benchmark_provider_selection() {
        // Benchmark AI provider selection with string caching
        let context = PrimalContext::default();
        let config = EcosystemConfig::default();

        let provider =
            SquirrelPrimalProvider::new(config, context).expect("Failed to create provider");

        provider.reset_zero_copy_metrics();

        let start = Instant::now();

        // Repeatedly select providers (should benefit from string caching)
        let task_types = vec!["text_generation", "code_generation", "analysis", "local"];

        for _ in 0..1000 {
            for task_type in &task_types {
                let request = serde_json::json!({
                    "task_type": task_type,
                    "messages": [{"role": "user", "content": "test"}]
                });

                let _response = provider
                    .handle_ai_inference_request(request)
                    .await
                    .expect("AI inference failed");
            }
        }

        let total_time = start.elapsed();
        let metrics = provider.get_zero_copy_metrics();

        println!("Provider selection benchmark time: {:?}", total_time);
        println!(
            "Zero-copy metrics: {}",
            serde_json::to_string_pretty(&metrics).unwrap()
        );

        // Verify optimizations were effective
        let efficiency = metrics.get("efficiency_ratio").unwrap().as_f64();
        assert!(
            efficiency > 0.9,
            "Expected high efficiency, got {}",
            efficiency
        );
    }
}
