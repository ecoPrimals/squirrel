//! Comprehensive Ecosystem Integration Tests
//!
//! This test suite validates cross-primal communication, coordination, and
//! integration patterns for the complete ecoPrimals ecosystem.

use squirrel::biomeos_integration::*;
use squirrel::error::PrimalError;
use squirrel::security::UniversalSecurityAdapter;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;

/// Test environment for ecosystem integration
#[derive(Clone)]
pub struct EcosystemTestEnvironment {
    pub songbird_endpoint: String,
    pub toadstool_endpoint: String,
    pub nestgate_endpoint: String,
    pub beardog_endpoint: String,
    pub squirrel_instance: Arc<RwLock<SquirrelBiomeOSIntegration>>,
    pub security_adapter: Arc<UniversalSecurityAdapter>,
}

impl EcosystemTestEnvironment {
    /// Create a new test environment with mock endpoints
    pub async fn new() -> Self {
        let squirrel_instance = Arc::new(RwLock::new(SquirrelBiomeOSIntegration::new(
            "test-biome".to_string(),
        )));

        let security_adapter = Arc::new(
            UniversalSecurityAdapter::new("test-config".to_string(), HashMap::new())
                .await
                .unwrap(),
        );

        Self {
            songbird_endpoint: "http://localhost:8080".to_string(),
            toadstool_endpoint: "http://localhost:8081".to_string(),
            nestgate_endpoint: "http://localhost:8082".to_string(),
            beardog_endpoint: "http://localhost:8083".to_string(),
            squirrel_instance,
            security_adapter,
        }
    }

    /// Initialize all ecosystem components
    pub async fn initialize(&self) -> Result<(), PrimalError> {
        // Initialize Squirrel integration
        let mut squirrel = self.squirrel_instance.write().await;
        squirrel.initialize().await?;

        // Initialize security adapter
        // Security adapter is already initialized in new()

        Ok(())
    }

    /// Shutdown all ecosystem components
    pub async fn shutdown(&self) -> Result<(), PrimalError> {
        let mut squirrel = self.squirrel_instance.write().await;
        squirrel.shutdown().await?;
        Ok(())
    }
}

/// Test ecosystem service discovery and registration
#[tokio::test]
async fn test_ecosystem_service_discovery() -> Result<(), Box<dyn std::error::Error>> {
    let env = EcosystemTestEnvironment::new().await;
    env.initialize().await?;

    // Test service registration
    {
        let squirrel = env.squirrel_instance.read().await;
        let registration = squirrel.create_service_registration().await?;

        // Verify registration structure
        assert_eq!(registration.primal_type, "squirrel");
        assert!(!registration.service_id.is_empty());
        assert!(!registration.capabilities.is_empty());
        assert!(registration.endpoints.len() > 0);
    }

    // Test service discovery
    {
        let squirrel = env.squirrel_instance.read().await;
        let services = squirrel.discover_ecosystem_services().await?;

        // Should discover at least the squirrel service
        assert!(services.len() >= 1);

        // Verify service discovery format
        for service in &services {
            assert!(!service.service_id.is_empty());
            assert!(!service.endpoint.is_empty());
            assert!(!service.capabilities.is_empty());
        }
    }

    env.shutdown().await?;
    Ok(())
}

/// Test cross-primal communication patterns
#[tokio::test]
async fn test_cross_primal_communication() -> Result<(), Box<dyn std::error::Error>> {
    let env = EcosystemTestEnvironment::new().await;
    env.initialize().await?;

    // Test ToadStool compute delegation
    {
        let squirrel = env.squirrel_instance.read().await;
        let compute_request = ComputeRequest {
            request_id: "test-compute-001".to_string(),
            task_type: "ai_inference".to_string(),
            payload: serde_json::json!({
                "model": "gpt-4",
                "prompt": "Hello, world!",
                "max_tokens": 100
            }),
            resource_requirements: ResourceRequirements {
                cpu_cores: 2.0,
                memory_gb: 4.0,
                gpu_required: false,
                storage_gb: 1.0,
                network_bandwidth_mbps: 100.0,
            },
            timeout_seconds: 30,
            metadata: HashMap::new(),
        };

        // This would normally delegate to ToadStool
        let result = squirrel.delegate_compute_task(&compute_request).await;

        // For testing, we expect it to handle gracefully even without ToadStool
        assert!(result.is_ok() || result.unwrap_err().to_string().contains("ToadStool"));
    }

    // Test NestGate storage integration
    {
        let squirrel = env.squirrel_instance.read().await;
        let storage_request = StorageRequest {
            request_id: "test-storage-001".to_string(),
            operation: "store".to_string(),
            path: "/ai/models/test-model".to_string(),
            data: b"test model data".to_vec(),
            metadata: HashMap::new(),
        };

        // This would normally use NestGate for storage
        let result = squirrel.store_data(&storage_request).await;

        // For testing, we expect it to handle gracefully
        assert!(result.is_ok() || result.unwrap_err().to_string().contains("NestGate"));
    }

    env.shutdown().await?;
    Ok(())
}

/// Test security integration across primals
#[tokio::test]
async fn test_cross_primal_security() -> Result<(), Box<dyn std::error::Error>> {
    let env = EcosystemTestEnvironment::new().await;
    env.initialize().await?;

    // Test BearDog authentication
    {
        let auth_request = AuthenticationRequest {
            request_id: "test-auth-001".to_string(),
            username: "test-user".to_string(),
            password: "test-password".to_string(),
            method: "password".to_string(),
            metadata: HashMap::new(),
        };

        let result = env.security_adapter.authenticate(auth_request).await;

        // Should handle authentication request
        assert!(result.is_ok());
        let auth_response = result.unwrap();
        assert_eq!(auth_response.request_id, "test-auth-001");
    }

    // Test authorization across services
    {
        let auth_request = AuthorizationRequest {
            request_id: "test-authz-001".to_string(),
            user_id: "test-user".to_string(),
            resource: "ai/models/gpt-4".to_string(),
            action: "read".to_string(),
            context: HashMap::new(),
        };

        let result = env.security_adapter.authorize(auth_request).await;

        // Should handle authorization request
        assert!(result.is_ok());
        let auth_response = result.unwrap();
        assert_eq!(auth_response.request_id, "test-authz-001");
    }

    env.shutdown().await?;
    Ok(())
}

/// Test ecosystem coordination patterns
#[tokio::test]
async fn test_ecosystem_coordination() -> Result<(), Box<dyn std::error::Error>> {
    let env = EcosystemTestEnvironment::new().await;
    env.initialize().await?;

    // Test multi-primal task coordination
    {
        let squirrel = env.squirrel_instance.read().await;
        let coordination_request = CoordinationRequest {
            request_id: "test-coord-001".to_string(),
            task_type: "ai_pipeline".to_string(),
            participants: vec![
                "squirrel".to_string(),
                "toadstool".to_string(),
                "nestgate".to_string(),
            ],
            workflow: serde_json::json!({
                "steps": [
                    {"primal": "squirrel", "action": "preprocess"},
                    {"primal": "toadstool", "action": "compute"},
                    {"primal": "nestgate", "action": "store"}
                ]
            }),
            timeout_seconds: 60,
            metadata: HashMap::new(),
        };

        let result = squirrel.coordinate_task(&coordination_request).await;

        // Should handle coordination request
        assert!(result.is_ok());
        let coord_response = result.unwrap();
        assert_eq!(coord_response.request_id, "test-coord-001");
    }

    env.shutdown().await?;
    Ok(())
}

/// Test ecosystem health monitoring
#[tokio::test]
async fn test_ecosystem_health_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    let env = EcosystemTestEnvironment::new().await;
    env.initialize().await?;

    // Test individual service health
    {
        let squirrel = env.squirrel_instance.read().await;
        let health_status = squirrel.get_health_status().await?;

        assert!(!health_status.service_id.is_empty());
        assert!(!health_status.status.is_empty());
        assert!(health_status.uptime_seconds > 0);
    }

    // Test ecosystem-wide health
    {
        let squirrel = env.squirrel_instance.read().await;
        let ecosystem_health = squirrel.get_ecosystem_health().await?;

        assert!(!ecosystem_health.is_empty());
        assert!(ecosystem_health.len() >= 1); // At least squirrel should be present

        for (primal, health) in &ecosystem_health {
            assert!(!primal.is_empty());
            assert!(!health.status.is_empty());
        }
    }

    env.shutdown().await?;
    Ok(())
}

/// Test biome.yaml manifest integration
#[tokio::test]
async fn test_biome_manifest_integration() -> Result<(), Box<dyn std::error::Error>> {
    let env = EcosystemTestEnvironment::new().await;
    env.initialize().await?;

    // Test manifest parsing and deployment
    {
        let squirrel = env.squirrel_instance.read().await;
        let manifest = BiomeManifestParser::generate_template();

        // Test manifest validation
        let parser = BiomeManifestParser::new();
        let yaml_content = serde_yaml::to_string(&manifest)?;
        let parsed = parser.parse_content(&yaml_content).await?;

        assert_eq!(parsed.metadata.name, "example-biome");
        assert!(!parsed.agents.is_empty());
        assert!(!parsed.services.is_empty());
    }

    // Test agent deployment from manifest
    {
        let squirrel = env.squirrel_instance.read().await;
        let deployment_manager = squirrel.get_agent_deployment_manager().await?;

        let manifest = BiomeManifestParser::generate_template();
        let deployed_agents = deployment_manager.deploy_from_manifest(&manifest).await?;

        assert!(!deployed_agents.is_empty());
        assert_eq!(deployed_agents[0].name, "data-analyst");
        assert_eq!(deployed_agents[0].status, "deployed");
    }

    env.shutdown().await?;
    Ok(())
}

/// Test ecosystem resilience and fault tolerance
#[tokio::test]
async fn test_ecosystem_resilience() -> Result<(), Box<dyn std::error::Error>> {
    let env = EcosystemTestEnvironment::new().await;
    env.initialize().await?;

    // Test service failure recovery
    {
        let squirrel = env.squirrel_instance.read().await;

        // Simulate service failure
        let failure_simulation = ServiceFailureSimulation {
            service_id: "test-service".to_string(),
            failure_type: "timeout".to_string(),
            duration_seconds: 5,
        };

        let result = squirrel.simulate_service_failure(&failure_simulation).await;

        // Should handle failure gracefully
        assert!(result.is_ok());
        let recovery_info = result.unwrap();
        assert!(!recovery_info.recovery_actions.is_empty());
    }

    // Test circuit breaker patterns
    {
        let squirrel = env.squirrel_instance.read().await;

        // Test circuit breaker state
        let circuit_state = squirrel.get_circuit_breaker_state("toadstool").await?;
        assert!(!circuit_state.is_empty());

        // Test automatic recovery
        let recovery_result = squirrel.test_circuit_recovery("toadstool").await?;
        assert!(recovery_result.recovery_successful);
    }

    env.shutdown().await?;
    Ok(())
}

/// Test ecosystem performance under load
#[tokio::test]
async fn test_ecosystem_performance() -> Result<(), Box<dyn std::error::Error>> {
    let env = EcosystemTestEnvironment::new().await;
    env.initialize().await?;

    // Test concurrent request handling
    {
        let squirrel = env.squirrel_instance.read().await;
        let mut handles = Vec::new();

        // Create 10 concurrent requests
        for i in 0..10 {
            let squirrel_clone = squirrel.clone();
            let handle = tokio::spawn(async move {
                let request = IntelligenceRequest {
                    request_id: format!("perf-test-{}", i),
                    request_type: "analysis".to_string(),
                    data: serde_json::json!({"test": i}),
                    metadata: HashMap::new(),
                };

                squirrel_clone.process_intelligence_request(request).await
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        let results = futures::future::join_all(handles).await;

        // All requests should complete successfully
        for result in results {
            assert!(result.is_ok());
            let response = result.unwrap();
            assert!(response.is_ok());
        }
    }

    // Test performance metrics
    {
        let squirrel = env.squirrel_instance.read().await;
        let metrics = squirrel.get_performance_metrics().await?;

        assert!(metrics.requests_processed > 0);
        assert!(metrics.average_response_time_ms > 0.0);
        assert!(metrics.success_rate >= 0.0 && metrics.success_rate <= 1.0);
    }

    env.shutdown().await?;
    Ok(())
}

/// Test ecosystem data flow patterns
#[tokio::test]
async fn test_ecosystem_data_flow() -> Result<(), Box<dyn std::error::Error>> {
    let env = EcosystemTestEnvironment::new().await;
    env.initialize().await?;

    // Test data pipeline: Squirrel -> ToadStool -> NestGate
    {
        let squirrel = env.squirrel_instance.read().await;

        // Step 1: Squirrel processes data
        let data_request = DataProcessingRequest {
            request_id: "data-flow-001".to_string(),
            data_type: "text".to_string(),
            payload: serde_json::json!({"text": "Hello, ecosystem!"}),
            processing_steps: vec![
                ProcessingStep {
                    step_id: "preprocess".to_string(),
                    primal: "squirrel".to_string(),
                    action: "tokenize".to_string(),
                    parameters: HashMap::new(),
                },
                ProcessingStep {
                    step_id: "compute".to_string(),
                    primal: "toadstool".to_string(),
                    action: "inference".to_string(),
                    parameters: HashMap::new(),
                },
                ProcessingStep {
                    step_id: "store".to_string(),
                    primal: "nestgate".to_string(),
                    action: "persist".to_string(),
                    parameters: HashMap::new(),
                },
            ],
            metadata: HashMap::new(),
        };

        let result = squirrel.process_data_pipeline(&data_request).await;

        // Should handle data pipeline request
        assert!(result.is_ok());
        let pipeline_response = result.unwrap();
        assert_eq!(pipeline_response.request_id, "data-flow-001");
        assert!(pipeline_response.steps_completed > 0);
    }

    env.shutdown().await?;
    Ok(())
}

/// Test ecosystem configuration management
#[tokio::test]
async fn test_ecosystem_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let env = EcosystemTestEnvironment::new().await;
    env.initialize().await?;

    // Test configuration synchronization
    {
        let squirrel = env.squirrel_instance.read().await;

        let config_update = ConfigurationUpdate {
            update_id: "config-001".to_string(),
            scope: "ecosystem".to_string(),
            changes: vec![
                ConfigChange {
                    key: "logging.level".to_string(),
                    value: "debug".to_string(),
                    target_primals: vec!["squirrel".to_string(), "toadstool".to_string()],
                },
                ConfigChange {
                    key: "security.encryption".to_string(),
                    value: "enabled".to_string(),
                    target_primals: vec!["beardog".to_string()],
                },
            ],
            metadata: HashMap::new(),
        };

        let result = squirrel.apply_configuration_update(&config_update).await;

        // Should handle configuration update
        assert!(result.is_ok());
        let update_response = result.unwrap();
        assert_eq!(update_response.update_id, "config-001");
        assert!(update_response.changes_applied > 0);
    }

    env.shutdown().await?;
    Ok(())
}

/// Test ecosystem event propagation
#[tokio::test]
async fn test_ecosystem_event_propagation() -> Result<(), Box<dyn std::error::Error>> {
    let env = EcosystemTestEnvironment::new().await;
    env.initialize().await?;

    // Test event publishing and subscription
    {
        let squirrel = env.squirrel_instance.read().await;

        // Subscribe to ecosystem events
        let subscription = squirrel
            .subscribe_to_ecosystem_events(vec![
                "primal.status.changed".to_string(),
                "task.completed".to_string(),
                "error.detected".to_string(),
            ])
            .await?;

        assert!(!subscription.subscription_id.is_empty());
        assert_eq!(subscription.event_types.len(), 3);

        // Publish a test event
        let test_event = EcosystemEvent {
            event_id: "test-event-001".to_string(),
            event_type: "primal.status.changed".to_string(),
            source_primal: "squirrel".to_string(),
            timestamp: chrono::Utc::now(),
            data: serde_json::json!({"status": "healthy"}),
            metadata: HashMap::new(),
        };

        let result = squirrel.publish_ecosystem_event(&test_event).await;

        // Should handle event publication
        assert!(result.is_ok());
        let publish_response = result.unwrap();
        assert_eq!(publish_response.event_id, "test-event-001");
        assert!(publish_response.subscribers_notified > 0);
    }

    env.shutdown().await?;
    Ok(())
}

/// Integration test timeout wrapper
async fn run_with_timeout<F, T>(
    test_name: &str,
    test_fn: F,
) -> Result<T, Box<dyn std::error::Error>>
where
    F: std::future::Future<Output = Result<T, Box<dyn std::error::Error>>>,
{
    match timeout(Duration::from_secs(30), test_fn).await {
        Ok(result) => result,
        Err(_) => {
            eprintln!("Test {} timed out after 30 seconds", test_name);
            Err("Test timeout".into())
        }
    }
}

/// Test ecosystem startup sequence
#[tokio::test]
async fn test_ecosystem_startup_sequence() -> Result<(), Box<dyn std::error::Error>> {
    run_with_timeout("ecosystem_startup_sequence", async {
        let env = EcosystemTestEnvironment::new().await;

        // Test ordered startup
        let startup_sequence = vec![
            ("beardog", "security"),
            ("nestgate", "storage"),
            ("songbird", "orchestration"),
            ("toadstool", "compute"),
            ("squirrel", "intelligence"),
        ];

        for (primal, service_type) in startup_sequence {
            let squirrel = env.squirrel_instance.read().await;
            let startup_result = squirrel.simulate_primal_startup(primal, service_type).await;

            assert!(startup_result.is_ok());
            let startup_info = startup_result.unwrap();
            assert_eq!(startup_info.primal_name, primal);
            assert_eq!(startup_info.service_type, service_type);
            assert!(startup_info.startup_successful);
        }

        env.shutdown().await?;
        Ok(())
    })
    .await
}

/// Test ecosystem shutdown sequence
#[tokio::test]
async fn test_ecosystem_shutdown_sequence() -> Result<(), Box<dyn std::error::Error>> {
    run_with_timeout("ecosystem_shutdown_sequence", async {
        let env = EcosystemTestEnvironment::new().await;
        env.initialize().await?;

        // Test graceful shutdown
        let shutdown_sequence = vec![
            ("squirrel", "intelligence"),
            ("toadstool", "compute"),
            ("songbird", "orchestration"),
            ("nestgate", "storage"),
            ("beardog", "security"),
        ];

        for (primal, service_type) in shutdown_sequence {
            let squirrel = env.squirrel_instance.read().await;
            let shutdown_result = squirrel
                .simulate_primal_shutdown(primal, service_type)
                .await;

            assert!(shutdown_result.is_ok());
            let shutdown_info = shutdown_result.unwrap();
            assert_eq!(shutdown_info.primal_name, primal);
            assert_eq!(shutdown_info.service_type, service_type);
            assert!(shutdown_info.shutdown_successful);
        }

        Ok(())
    })
    .await
}
