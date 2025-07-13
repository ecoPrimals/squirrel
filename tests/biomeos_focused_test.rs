//! Focused BiomeOS Integration Tests
//!
//! Tests for our actual implemented biomeOS integration functionality.

use squirrel::biomeos_integration::*;
use squirrel::error::PrimalError;
use std::collections::HashMap;

/// Test biomeOS integration creation and basic functionality
#[tokio::test]
async fn test_biomeos_integration_basic() -> Result<(), Box<dyn std::error::Error>> {
    // Test integration creation
    let integration = SquirrelBiomeOSIntegration::new("test-biome".to_string());
    
    // Verify basic properties
    assert!(!integration.service_id.is_empty());
    assert_eq!(integration.biome_id, "test-biome");
    assert_eq!(integration.health_status.status, "initializing");
    
    Ok(())
}

/// Test AI intelligence module
#[tokio::test]
async fn test_ai_intelligence_functionality() -> Result<(), Box<dyn std::error::Error>> {
    let mut ai_intelligence = AiIntelligence::new();
    
    // Test initialization
    ai_intelligence.initialize().await?;
    
    // Test ecosystem analysis
    let _analysis = ai_intelligence.analyze_ecosystem().await?;
    // Note: analyze_ecosystem returns (), so we just verify it doesn't error
    
    // Test ecosystem report generation
    let report = ai_intelligence.generate_ecosystem_report().await?;
    assert!(!report.recommendations.is_empty());
    
    Ok(())
}

/// Test context state management
#[tokio::test]
async fn test_context_state_management() -> Result<(), Box<dyn std::error::Error>> {
    let mut context_state = ContextState::new();
    
    // Test initialization
    context_state.initialize().await?;
    
    // Test session context creation
    context_state.create_session_context(
        "test-session-001".to_string(),
        Some("user-123".to_string()),
        "test_context".to_string()
    ).await?;
    
    // Verify session was created
    assert_eq!(context_state.get_active_sessions(), 1);
    
    // Test context updates
    let mut updates = HashMap::new();
    updates.insert("test_key".to_string(), serde_json::json!("test_value"));
    context_state.update_session_context("test-session-001", updates).await?;
    
    Ok(())
}

/// Test MCP integration
#[tokio::test]
async fn test_mcp_integration_functionality() -> Result<(), Box<dyn std::error::Error>> {
    let mut mcp_integration = McpIntegration::new();
    
    // Test initialization
    mcp_integration.initialize().await?;
    
    // Test coordination session creation (with correct parameters)
    let participants = vec!["beardog".to_string(), "toadstool".to_string()];
    let session_id = mcp_integration.create_coordination_session(participants, "security_check".to_string()).await?;
    assert!(!session_id.is_empty());
    
    // Test coordination request handling
    let mut coordination_data = HashMap::new();
    coordination_data.insert("security_level".to_string(), serde_json::json!("high"));
    
    let request = McpCoordinationRequest {
        coordination_id: "coord-001".to_string(),
        coordination_type: "security_check".to_string(),
        participants: vec!["beardog".to_string()],
        coordination_data,
    };
    
    let response = mcp_integration.handle_coordination_request(request).await?;
    assert_eq!(response.coordination_id, "coord-001");
    
    Ok(())
}

/// Test ecosystem client basic functionality
#[tokio::test]
async fn test_ecosystem_client_basic() -> Result<(), Box<dyn std::error::Error>> {
    // Test client creation
    let client = EcosystemClient::new();
    assert_eq!(client.songbird_url, "http://localhost:8080");
    assert_eq!(client.retry_count, 3);
    
    // Test client with custom config
    let auth_config = AuthenticationConfig::default();
    let custom_client = EcosystemClient::with_config("http://localhost:9090".to_string(), auth_config);
    assert_eq!(custom_client.songbird_url, "http://localhost:9090");
    
    Ok(())
}

/// Test service registration structure
#[tokio::test]
async fn test_service_registration_structure() -> Result<(), Box<dyn std::error::Error>> {
    // Test default service registration
    let registration = EcosystemServiceRegistration::default();
    assert_eq!(registration.primal_type, "squirrel");
    assert!(!registration.service_id.is_empty());
    
    // Test capabilities structure
    let capabilities = EcosystemCapabilities::default();
    assert!(!capabilities.ai_capabilities.is_empty());
    assert!(!capabilities.mcp_capabilities.is_empty());
    assert!(!capabilities.context_capabilities.is_empty());
    
    Ok(())
}

/// Test health status structure
#[tokio::test]
async fn test_health_status_structure() -> Result<(), Box<dyn std::error::Error>> {
    // Test health status creation
    let health_status = HealthStatus {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        ai_engine_status: "operational".to_string(),
        mcp_server_status: "active".to_string(),
        context_manager_status: "running".to_string(),
        active_sessions: 5,
        ai_requests_processed: 1000,
        context_states_managed: 50,
    };
    
    // Verify health status fields
    assert_eq!(health_status.status, "healthy");
    assert_eq!(health_status.active_sessions, 5);
    assert_eq!(health_status.ai_requests_processed, 1000);
    assert_eq!(health_status.context_states_managed, 50);
    
    Ok(())
}

/// Test context state request/response handling
#[tokio::test]
async fn test_context_state_requests() -> Result<(), Box<dyn std::error::Error>> {
    let context_state = ContextState::new();
    
    // Test context state request
    let request = ContextStateRequest {
        session_id: "test-session".to_string(),
        request_type: "get_context".to_string(),
        context_data: None,
        query: Some("test query".to_string()),
    };
    
    let response = context_state.handle_state_request(request).await?;
    assert_eq!(response.session_id, "test-session");
    // Note: context_state may be empty for a new session, which is expected
    
    Ok(())
}

/// Test intelligence request/response structures
#[tokio::test]
async fn test_intelligence_requests() -> Result<(), Box<dyn std::error::Error>> {
    // Test intelligence request structure
    let request = IntelligenceRequest {
        request_id: "intel-001".to_string(),
        request_type: "ecosystem_analysis".to_string(),
        target_component: Some("beardog".to_string()),
        parameters: HashMap::new(),
        context: None,
    };
    
    assert_eq!(request.request_id, "intel-001");
    assert_eq!(request.request_type, "ecosystem_analysis");
    
    // Test intelligence response structure
    let response = IntelligenceResponse {
        request_id: "intel-001".to_string(),
        response_type: "analysis_complete".to_string(),
        recommendations: vec!["Optimize security protocols".to_string()],
        predictions: vec![],
        optimizations: vec![],
        confidence: 0.95,
        metadata: HashMap::new(),
    };
    
    assert_eq!(response.request_id, "intel-001");
    assert_eq!(response.confidence, 0.95);
    assert!(!response.recommendations.is_empty());
    
    Ok(())
}

/// Test error handling
#[tokio::test]
async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    // Test different error types
    let network_error = PrimalError::Network("Connection failed".to_string());
    let auth_error = PrimalError::Authentication("Unauthorized".to_string());
    let internal_error = PrimalError::Internal("Internal error".to_string());
    
    // Verify error messages
    assert!(network_error.to_string().contains("Connection failed"));
    assert!(auth_error.to_string().contains("Unauthorized"));
    assert!(internal_error.to_string().contains("Internal error"));
    
    Ok(())
}

// Test utilities for creating test data
mod test_utils {
    use super::*;

    pub fn create_test_integration() -> SquirrelBiomeOSIntegration {
        SquirrelBiomeOSIntegration::new("test-biome".to_string())
    }

    pub fn create_test_auth_config() -> AuthenticationConfig {
        AuthenticationConfig {
            auth_type: "ecosystem_jwt".to_string(),
            token: Some("test-token".to_string()),
            client_id: Some("test-client".to_string()),
            client_secret: Some("test-secret".to_string()),
            trust_domain: "biome.local".to_string(),
        }
    }

    pub fn create_test_health_status() -> HealthStatus {
        HealthStatus {
            status: "healthy".to_string(),
            timestamp: chrono::Utc::now(),
            ai_engine_status: "operational".to_string(),
            mcp_server_status: "active".to_string(),
            context_manager_status: "running".to_string(),
            active_sessions: 5,
            ai_requests_processed: 1000,
            context_states_managed: 50,
        }
    }
} 