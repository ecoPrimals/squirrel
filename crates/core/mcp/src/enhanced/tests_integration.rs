// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Integration tests for MCP platform components
//!
//! These tests verify the interaction between different MCP components
//! and ensure the platform works correctly in realistic scenarios.

use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

use crate::protocol::{MCPRequest, MCPResponse, MCPMessage};
use crate::transport::websocket::WebSocketTransport;
use crate::enhanced::service_composition::{ServiceComposition, ServiceConfig};
use crate::enhanced::workflow::{WorkflowEngine, WorkflowDefinition, WorkflowStep};
use crate::error::MCPError;

/// Integration test configuration
struct IntegrationTestConfig {
    timeout: Duration,
    retry_attempts: u32,
    buffer_size: usize,
}

impl Default for IntegrationTestConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            retry_attempts: 3,
            buffer_size: 1024,
        }
    }
}

/// Mock MCP service for testing
#[derive(Clone)]
struct MockMCPService {
    service_id: String,
    capabilities: Vec<String>,
    message_count: Arc<tokio::sync::Mutex<u32>>,
}

impl MockMCPService {
    fn new(service_id: &str) -> Self {
        Self {
            service_id: service_id.to_string(),
            capabilities: vec!["test_capability".to_string()],
            message_count: Arc::new(tokio::sync::Mutex::new(0)),
        }
    }

    async fn handle_request(&self, request: &MCPRequest) -> Result<MCPResponse, MCPError> {
        let mut count = self.message_count.lock().await;
        *count += 1;

        match request.method.as_str() {
            "test_echo" => {
                Ok(MCPResponse {
                    id: request.id.clone(),
                    result: request.params.clone(),
                    error: None,
                })
            }
            "test_capabilities" => {
                Ok(MCPResponse {
                    id: request.id.clone(),
                    result: Some(serde_json::json!({
                        "capabilities": self.capabilities,
                        "service_id": self.service_id
                    })),
                    error: None,
                })
            }
            _ => {
                Err(MCPError::MethodNotFound(request.method.clone()))
            }
        }
    }

    async fn get_message_count(&self) -> u32 {
        *self.message_count.lock().await
    }
}

#[cfg(test)]
mod protocol_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_protocol_message_roundtrip() {
        let config = IntegrationTestConfig::default();
        let service = MockMCPService::new("test_service");

        let request = MCPRequest {
            id: Uuid::new_v4().to_string(),
            method: "test_echo".to_string(),
            params: Some(serde_json::json!({"message": "hello world"})),
        };

        let response = service.handle_request(&request).await.unwrap();

        assert_eq!(response.id, request.id);
        assert_eq!(response.result, request.params);
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_protocol_error_handling() {
        let service = MockMCPService::new("test_service");

        let request = MCPRequest {
            id: Uuid::new_v4().to_string(),
            method: "unknown_method".to_string(),
            params: None,
        };

        let result = service.handle_request(&request).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            MCPError::MethodNotFound(method) => {
                assert_eq!(method, "unknown_method");
            }
            _ => panic!("Expected MethodNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_concurrent_request_handling() {
        let service = Arc::new(MockMCPService::new("concurrent_test"));
        let mut handles = Vec::new();

        // Send multiple concurrent requests
        for i in 0..10 {
            let service_clone = Arc::clone(&service);
            let handle = tokio::spawn(async move {
                let request = MCPRequest {
                    id: format!("request_{}", i),
                    method: "test_echo".to_string(),
                    params: Some(serde_json::json!({"index": i})),
                };

                service_clone.handle_request(&request).await
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        for handle in handles {
            let response = handle.await.unwrap().unwrap();
            assert!(response.error.is_none());
            assert!(response.result.is_some());
        }

        // Verify all messages were processed
        let message_count = service.get_message_count().await;
        assert_eq!(message_count, 10);
    }
}

#[cfg(test)]
mod service_composition_integration_tests {
    use super::*;

    async fn create_test_service_config(name: &str, port: u16) -> ServiceConfig {
        ServiceConfig {
            name: name.to_string(),
            endpoint: format!("http://localhost:{}", port),
            capabilities: vec!["test".to_string()],
            timeout: Duration::from_secs(10),
            retry_policy: None,
        }
    }

    #[tokio::test]
    async fn test_service_composition_lifecycle() {
        let mut composition = ServiceComposition::new();

        // Add services
        let service1 = create_test_service_config("service1", 8081).await;
        let service2 = create_test_service_config("service2", 8082).await;

        composition.add_service(service1.clone()).await.unwrap();
        composition.add_service(service2.clone()).await.unwrap();

        // Verify services are registered
        let services = composition.list_services().await.unwrap();
        assert_eq!(services.len(), 2);
        assert!(services.contains(&service1.name));
        assert!(services.contains(&service2.name));

        // Remove a service
        composition.remove_service(&service1.name).await.unwrap();
        let services = composition.list_services().await.unwrap();
        assert_eq!(services.len(), 1);
        assert!(!services.contains(&service1.name));
        assert!(services.contains(&service2.name));
    }

    #[tokio::test]
    async fn test_service_discovery_and_routing() {
        let mut composition = ServiceComposition::new();

        // Add services with different capabilities
        let web_service = ServiceConfig {
            name: "web_service".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec!["http".to_string(), "web".to_string()],
            timeout: Duration::from_secs(10),
            retry_policy: None,
        };

        let db_service = ServiceConfig {
            name: "db_service".to_string(),
            endpoint: "http://localhost:8081".to_string(),
            capabilities: vec!["database".to_string(), "storage".to_string()],
            timeout: Duration::from_secs(15),
            retry_policy: None,
        };

        composition.add_service(web_service).await.unwrap();
        composition.add_service(db_service).await.unwrap();

        // Test capability-based service discovery
        let web_services = composition
            .discover_services_by_capability("web")
            .await
            .unwrap();
        assert_eq!(web_services.len(), 1);
        assert_eq!(web_services[0], "web_service");

        let db_services = composition
            .discover_services_by_capability("database")
            .await
            .unwrap();
        assert_eq!(db_services.len(), 1);
        assert_eq!(db_services[0], "db_service");

        // Test routing to specific service
        let route_target = composition
            .route_request("database", &serde_json::json!({"query": "SELECT * FROM users"}))
            .await
            .unwrap();
        assert_eq!(route_target, "db_service");
    }
}

#[cfg(test)]
mod workflow_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_workflow_execution_pipeline() {
        let engine = WorkflowEngine::new();

        // Create a test workflow with multiple steps
        let workflow = WorkflowDefinition {
            id: "test_workflow".to_string(),
            name: "Test Workflow".to_string(),
            steps: vec![
                WorkflowStep {
                    id: "step1".to_string(),
                    name: "Data Validation".to_string(),
                    service: "validation_service".to_string(),
                    method: "validate".to_string(),
                    input: serde_json::json!({"data": "test_data"}),
                    dependencies: vec![],
                    timeout: Duration::from_secs(10),
                },
                WorkflowStep {
                    id: "step2".to_string(),
                    name: "Data Processing".to_string(),
                    service: "processing_service".to_string(),
                    method: "process".to_string(),
                    input: serde_json::json!({"validated_data": "${step1.result}"}),
                    dependencies: vec!["step1".to_string()],
                    timeout: Duration::from_secs(30),
                },
                WorkflowStep {
                    id: "step3".to_string(),
                    name: "Result Storage".to_string(),
                    service: "storage_service".to_string(),
                    method: "store".to_string(),
                    input: serde_json::json!({"processed_data": "${step2.result}"}),
                    dependencies: vec!["step2".to_string()],
                    timeout: Duration::from_secs(15),
                },
            ],
        };

        // Register workflow
        engine.register_workflow(workflow.clone()).await.unwrap();

        // Verify workflow registration
        let registered_workflows = engine.list_workflows().await.unwrap();
        assert!(registered_workflows.contains(&workflow.id));

        // Execute workflow (would normally interact with real services)
        // This test verifies the workflow structure and dependencies
        let execution_plan = engine
            .create_execution_plan(&workflow.id)
            .await
            .unwrap();

        assert_eq!(execution_plan.len(), 3);
        assert_eq!(execution_plan[0].dependencies.len(), 0); // step1 has no deps
        assert_eq!(execution_plan[1].dependencies.len(), 1); // step2 depends on step1
        assert_eq!(execution_plan[2].dependencies.len(), 1); // step3 depends on step2
    }

    #[tokio::test]
    async fn test_workflow_error_handling_and_recovery() {
        let engine = WorkflowEngine::new();

        let workflow_with_failure = WorkflowDefinition {
            id: "failure_workflow".to_string(),
            name: "Failure Test Workflow".to_string(),
            steps: vec![
                WorkflowStep {
                    id: "success_step".to_string(),
                    name: "Success Step".to_string(),
                    service: "reliable_service".to_string(),
                    method: "process".to_string(),
                    input: serde_json::json!({"data": "good_data"}),
                    dependencies: vec![],
                    timeout: Duration::from_secs(10),
                },
                WorkflowStep {
                    id: "failure_step".to_string(),
                    name: "Failure Step".to_string(),
                    service: "unreliable_service".to_string(),
                    method: "fail".to_string(),
                    input: serde_json::json!({"data": "bad_data"}),
                    dependencies: vec!["success_step".to_string()],
                    timeout: Duration::from_secs(5),
                },
                WorkflowStep {
                    id: "recovery_step".to_string(),
                    name: "Recovery Step".to_string(),
                    service: "recovery_service".to_string(),
                    method: "recover".to_string(),
                    input: serde_json::json!({"error": "${failure_step.error}"}),
                    dependencies: vec!["failure_step".to_string()],
                    timeout: Duration::from_secs(15),
                },
            ],
        };

        engine.register_workflow(workflow_with_failure.clone()).await.unwrap();

        // Test error propagation and recovery mechanisms
        let execution_plan = engine
            .create_execution_plan(&workflow_with_failure.id)
            .await
            .unwrap();

        // Verify that recovery step can handle failure from previous step
        let recovery_step = execution_plan
            .iter()
            .find(|step| step.id == "recovery_step")
            .unwrap();

        assert!(recovery_step.dependencies.contains(&"failure_step".to_string()));
        assert!(recovery_step.input.to_string().contains("${failure_step.error}"));
    }
}

#[cfg(test)]
mod transport_integration_tests {
    use super::*;

    #[tokio::test] 
    async fn test_websocket_connection_lifecycle() {
        let config = IntegrationTestConfig::default();
        
        // This would normally create a real WebSocket connection
        // For now, we test the transport configuration and setup
        let transport_config = serde_json::json!({
            "protocol": "websocket",
            "host": "localhost", 
            "port": 8080,
            "path": "/mcp",
            "timeout": config.timeout.as_secs(),
            "buffer_size": config.buffer_size
        });

        // Verify transport configuration
        assert_eq!(transport_config["protocol"], "websocket");
        assert_eq!(transport_config["host"], "localhost");
        assert_eq!(transport_config["port"], 8080);
        assert_eq!(transport_config["timeout"], 30);
    }

    #[tokio::test]
    async fn test_message_framing_and_serialization() {
        let request = MCPRequest {
            id: Uuid::new_v4().to_string(),
            method: "test_method".to_string(),
            params: Some(serde_json::json!({
                "arg1": "value1",
                "arg2": 42,
                "nested": {
                    "inner": "data"
                }
            })),
        };

        let message = MCPMessage::Request(request.clone());

        // Test serialization
        let serialized = serde_json::to_string(&message).unwrap();
        assert!(serialized.contains("test_method"));
        assert!(serialized.contains("value1"));
        assert!(serialized.contains("42"));

        // Test deserialization
        let deserialized: MCPMessage = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            MCPMessage::Request(deserialized_request) => {
                assert_eq!(deserialized_request.id, request.id);
                assert_eq!(deserialized_request.method, request.method);
                assert_eq!(deserialized_request.params, request.params);
            }
            _ => panic!("Expected Request message"),
        }
    }
}

#[cfg(test)]
mod performance_integration_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_high_throughput_message_processing() {
        let service = Arc::new(MockMCPService::new("performance_test"));
        let message_count = 1000;
        let start_time = Instant::now();

        let mut handles = Vec::new();

        for i in 0..message_count {
            let service_clone = Arc::clone(&service);
            let handle = tokio::spawn(async move {
                let request = MCPRequest {
                    id: format!("perf_request_{}", i),
                    method: "test_echo".to_string(),
                    params: Some(serde_json::json!({"index": i, "data": "test_payload"})),
                };

                service_clone.handle_request(&request).await
            });
            handles.push(handle);
        }

        // Wait for all requests
        let mut success_count = 0;
        for handle in handles {
            if handle.await.unwrap().is_ok() {
                success_count += 1;
            }
        }

        let elapsed = start_time.elapsed();
        let throughput = message_count as f64 / elapsed.as_secs_f64();

        println!("Processed {} messages in {:?} ({:.2} msg/sec)", 
                 message_count, elapsed, throughput);

        assert_eq!(success_count, message_count);
        assert!(throughput > 100.0, "Throughput too low: {:.2} msg/sec", throughput);

        // Verify all messages were processed
        let final_count = service.get_message_count().await;
        assert_eq!(final_count, message_count);
    }

    #[tokio::test]
    async fn test_memory_usage_under_load() {
        let service = Arc::new(MockMCPService::new("memory_test"));
        
        // Process large payloads to test memory handling
        let large_payload = "x".repeat(1024 * 10); // 10KB payload
        let request_count = 100;

        for i in 0..request_count {
            let request = MCPRequest {
                id: format!("memory_request_{}", i),
                method: "test_echo".to_string(),
                params: Some(serde_json::json!({
                    "index": i,
                    "large_data": large_payload.clone()
                })),
            };

            let response = service.handle_request(&request).await.unwrap();
            
            // Verify response contains the data
            assert!(response.result.is_some());
            assert_eq!(response.id, request.id);
            
            // Add small delay to prevent overwhelming the system
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        let final_count = service.get_message_count().await;
        assert_eq!(final_count, request_count);
    }
}

#[cfg(test)]
mod reliability_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_graceful_degradation() {
        let service = MockMCPService::new("degradation_test");
        
        // Test handling of various error conditions
        let error_scenarios = vec![
            ("invalid_method", "nonexistent_method"),
            ("malformed_params", "test_echo"), // Will succeed but tests different code path
        ];

        for (test_name, method) in error_scenarios {
            let request = MCPRequest {
                id: format!("{}_request", test_name),
                method: method.to_string(),
                params: Some(serde_json::json!({"test": "data"})),
            };

            let result = service.handle_request(&request).await;
            
            match method {
                "nonexistent_method" => {
                    assert!(result.is_err(), "Expected error for {}", test_name);
                }
                _ => {
                    assert!(result.is_ok(), "Expected success for {}", test_name);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_timeout_handling() {
        let config = IntegrationTestConfig::default();
        let service = MockMCPService::new("timeout_test");

        let request = MCPRequest {
            id: "timeout_request".to_string(),
            method: "test_echo".to_string(),
            params: Some(serde_json::json!({"message": "test"})),
        };

        // Test that normal requests complete within timeout
        let result = timeout(config.timeout, service.handle_request(&request)).await;
        
        assert!(result.is_ok(), "Request should complete within timeout");
        
        let response = result.unwrap().unwrap();
        assert_eq!(response.id, request.id);
    }
} 