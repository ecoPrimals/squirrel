//! Comprehensive tests for Songbird integration

use super::*;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path, header};

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_mock_songbird() -> MockServer {
        MockServer::start().await
    }

    async fn setup_integration(mock_server: &MockServer) -> SongbirdIntegration {
        std::env::set_var("SONGBIRD_ENDPOINT", mock_server.uri());
        std::env::set_var("SONGBIRD_AUTH_TOKEN", "test-token");
        
        SongbirdIntegration::new()
    }

    #[tokio::test]
    async fn test_successful_health_check() {
        let mock_server = setup_mock_songbird().await;
        
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let integration = setup_integration(&mock_server).await;
        
        let result = integration.test_connection().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_failed_health_check() {
        let mock_server = setup_mock_songbird().await;
        
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let integration = setup_integration(&mock_server).await;
        
        let result = integration.test_connection().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_successful_service_registration() {
        let mock_server = setup_mock_songbird().await;
        
        Mock::given(method("POST"))
            .and(path("/api/v1/services/register"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let integration = setup_integration(&mock_server).await;
        
        let result = integration.register_with_songbird().await;
        assert!(result.is_ok());
        
        // Verify registration status
        let state = integration.orchestration_state.read().await;
        assert!(state.registered);
    }

    #[tokio::test]
    async fn test_failed_service_registration() {
        let mock_server = setup_mock_songbird().await;
        
        Mock::given(method("POST"))
            .and(path("/api/v1/services/register"))
            .respond_with(ResponseTemplate::new(401))
            .expect(1)
            .mount(&mock_server)
            .await;

        let integration = setup_integration(&mock_server).await;
        
        let result = integration.register_with_songbird().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_service_discovery() {
        let mock_server = setup_mock_songbird().await;
        
        let mock_services = vec![
            json!({
                "service_id": "toadstool-1",
                "primal_type": "toadstool",
                "endpoint": "http://toadstool:8080",
                "capabilities": ["compute", "processing"],
                "health": "healthy",
                "metadata": {"region": "us-west", "zone": "az1"}
            }),
            json!({
                "service_id": "nestgate-1",
                "primal_type": "nestgate",
                "endpoint": "http://nestgate:8080",
                "capabilities": ["storage", "persistence"],
                "health": "healthy",
                "metadata": {"region": "us-west", "zone": "az1"}
            })
        ];

        Mock::given(method("GET"))
            .and(path("/api/v1/services"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_services))
            .expect(1)
            .mount(&mock_server)
            .await;

        let integration = setup_integration(&mock_server).await;
        
        let result = integration.discover_services().await;
        assert!(result.is_ok());
        
        let services = result.unwrap();
        assert_eq!(services.len(), 2);
        assert_eq!(services[0].service_id, "toadstool-1");
        assert_eq!(services[1].service_id, "nestgate-1");
    }

    #[tokio::test]
    async fn test_coordination_request() {
        let mock_server = setup_mock_songbird().await;
        
        let expected_response = json!({
            "session_id": "coord-123",
            "status": "active",
            "participants": ["squirrel", "toadstool"],
            "result": null
        });

        Mock::given(method("POST"))
            .and(path("/api/v1/coordination"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(expected_response))
            .expect(1)
            .mount(&mock_server)
            .await;

        let integration = setup_integration(&mock_server).await;
        
        let result = integration.coordinate(
            "resource_optimization",
            vec!["squirrel".to_string(), "toadstool".to_string()]
        ).await;
        
        assert!(result.is_ok());
        let session_id = result.unwrap();
        assert_eq!(session_id, "coord-123");
        
        // Verify session is stored locally
        let state = integration.orchestration_state.read().await;
        assert!(state.active_coordinations.contains_key(&session_id));
    }

    #[tokio::test]
    async fn test_heartbeat_sending() {
        let mock_server = setup_mock_songbird().await;
        
        Mock::given(method("POST"))
            .and(path("/api/v1/heartbeat"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let mut integration = setup_integration(&mock_server).await;
        
        let result = integration.send_heartbeat().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_unregistration() {
        let mock_server = setup_mock_songbird().await;
        
        Mock::given(method("POST"))
            .and(path("/api/v1/services/squirrel-test/unregister"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let integration = setup_integration(&mock_server).await;
        
        // Set a test service ID
        {
            let mut state = integration.orchestration_state.write().await;
            state.service_id = "squirrel-test".to_string();
        }
        
        let result = integration.unregister_from_songbird().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_full_integration_lifecycle() {
        let mock_server = setup_mock_songbird().await;
        
        // Health check
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;
        
        // Registration
        Mock::given(method("POST"))
            .and(path("/api/v1/services/register"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;
        
        // Heartbeat
        Mock::given(method("POST"))
            .and(path("/api/v1/heartbeat"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;
        
        // Unregistration
        Mock::given(method("POST"))
            .and(path("/api/v1/services"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let mut integration = setup_integration(&mock_server).await;
        
        // Initialize
        let init_result = integration.initialize().await;
        assert!(init_result.is_ok());
        assert_eq!(integration.health_status.status, "running");
        
        // Send heartbeat
        let heartbeat_result = integration.send_heartbeat().await;
        assert!(heartbeat_result.is_ok());
        
        // Shutdown
        let shutdown_result = integration.shutdown().await;
        assert!(shutdown_result.is_ok());
        assert_eq!(integration.health_status.status, "shutdown");
    }

    #[tokio::test]
    async fn test_error_handling_network_timeout() {
        let mock_server = setup_mock_songbird().await;
        
        // Mock a slow response that will timeout
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(10)))
            .expect(1)
            .mount(&mock_server)
            .await;

        let integration = setup_integration(&mock_server).await;
        
        let result = integration.test_connection().await;
        assert!(result.is_err());
        
        if let Err(PrimalError::Network(msg)) = result {
            assert!(msg.contains("Failed to connect to Songbird"));
        } else {
            panic!("Expected Network error");
        }
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        let config = SongbirdConfig::default();
        
        // Test default values
        assert_eq!(config.heartbeat_interval, Duration::from_secs(30));
        assert_eq!(config.coordination_timeout, Duration::from_secs(60));
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.service_name, "squirrel-mcp");
    }

    #[tokio::test]
    async fn test_service_metadata_generation() {
        let integration = SongbirdIntegration::new();
        let metadata = integration.get_service_metadata();
        
        assert!(metadata.contains_key("version"));
        assert!(metadata.contains_key("region"));
        assert!(metadata.contains_key("zone"));
        assert_eq!(metadata.get("version"), Some(&"2.2.0".to_string()));
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let mock_server = setup_mock_songbird().await;
        
        // Mock multiple service calls
        Mock::given(method("GET"))
            .and(path("/api/v1/services"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .expect(3)
            .mount(&mock_server)
            .await;

        let integration = Arc::new(setup_integration(&mock_server).await);
        
        // Launch multiple concurrent discovery calls
        let mut handles = vec![];
        for _ in 0..3 {
            let integration_clone = integration.clone();
            handles.push(tokio::spawn(async move {
                integration_clone.discover_services().await
            }));
        }
        
        // Wait for all operations to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_health_status_updates() {
        let mut integration = SongbirdIntegration::new();
        
        // Initial health status
        assert_eq!(integration.health_status.status, "initializing");
        
        // Update health
        integration.update_health().await.unwrap();
        
        // Verify health status is updated
        assert_eq!(integration.health_status.active_sessions, 0);
        assert_eq!(integration.health_status.resource_utilization, 0.5);
        
        // Add a coordination session
        {
            let mut state = integration.orchestration_state.write().await;
            state.active_coordinations.insert("test-session".to_string(), CoordinationSession {
                session_id: "test-session".to_string(),
                participants: vec!["squirrel".to_string()],
                session_type: "test".to_string(),
                status: "active".to_string(),
                created_at: Utc::now(),
                last_activity: Utc::now(),
            });
        }
        
        // Update health again
        integration.update_health().await.unwrap();
        
        // Verify session count is updated
        assert_eq!(integration.health_status.active_sessions, 1);
    }

    #[tokio::test]
    async fn test_authentication_header_inclusion() {
        let mock_server = setup_mock_songbird().await;
        
        Mock::given(method("GET"))
            .and(path("/api/v1/services"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .expect(1)
            .mount(&mock_server)
            .await;

        let integration = setup_integration(&mock_server).await;
        
        let result = integration.discover_services().await;
        assert!(result.is_ok());
    }
} 