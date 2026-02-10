// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Enhanced MCP Simple Test
//!
//! Comprehensive tests for biomeOS integration functionality using the actual API.

use squirrel::biomeos_integration::*;
use squirrel::error::PrimalError;

type Result<T> = std::result::Result<T, PrimalError>;

/// Test AI intelligence functionality
#[tokio::test]
async fn test_ai_intelligence() -> Result<()> {
    let ai_intelligence = AiIntelligence::new();

    // Test basic functionality
    assert_eq!(ai_intelligence.active_predictions, 0);
    assert_eq!(ai_intelligence.automation_tasks, 0);

    // Test ecosystem analysis
    let analysis_result = ai_intelligence.generate_ecosystem_report().await?;
    assert!(!analysis_result.recommendations.is_empty());

    Ok(())
}

/// Test MCP integration functionality
#[tokio::test]
async fn test_mcp_integration() -> Result<()> {
    let mut mcp_integration = McpIntegration::new();

    // Test coordination session creation
    let session_id = mcp_integration
        .create_coordination_session(vec!["test-primal".to_string()], "test-session".to_string())
        .await?;

    assert!(!session_id.is_empty());

    Ok(())
}

/// Test context state management
#[tokio::test]
async fn test_context_state_management() -> Result<()> {
    let mut context_state = ContextState::new();

    // Test session creation
    context_state
        .create_session_context(
            "test-session-001".to_string(),
            Some("user-123".to_string()),
            "test_context".to_string(),
        )
        .await?;

    // Verify session was created
    assert_eq!(context_state.get_active_sessions(), 1);

    Ok(())
}

/// Test service registration structure and validation
#[tokio::test]
async fn test_service_registration_structure() -> Result<()> {
    // Create service registration
    let registration = EcosystemServiceRegistration::default();

    // Validate registration structure
    assert_eq!(registration.primal_type, "squirrel");
    assert_eq!(registration.service_id, "primal-squirrel-ai-default");
    assert!(!registration.capabilities.ai_capabilities.is_empty());
    assert!(!registration.capabilities.mcp_capabilities.is_empty());
    assert!(!registration.capabilities.context_capabilities.is_empty());

    // Validate specific AI capabilities
    assert!(registration
        .capabilities
        .ai_capabilities
        .contains(&"ecosystem_intelligence".to_string()));
    assert!(registration
        .capabilities
        .ai_capabilities
        .contains(&"predictive_analytics".to_string()));

    Ok(())
}

/// Test health status reporting
#[tokio::test]
async fn test_health_status_reporting() -> Result<()> {
    // Create health status with ALL required fields
    let health_status = HealthStatus {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        ai_engine_status: "operational".to_string(),
        mcp_server_status: "active".to_string(),
        context_manager_status: "running".to_string(),
        agent_deployment_status: "active".to_string(), // Required field
        active_sessions: 10,
        ai_requests_processed: 150,
        context_states_managed: 25,
        deployed_agents: 5, // Required field
    };

    // Validate health status
    assert_eq!(health_status.status, "healthy");
    assert_eq!(health_status.active_sessions, 10);
    assert_eq!(health_status.ai_requests_processed, 150);
    assert_eq!(health_status.context_states_managed, 25);
    assert_eq!(health_status.deployed_agents, 5);
    assert_eq!(health_status.ai_engine_status, "operational");
    assert_eq!(health_status.mcp_server_status, "active");
    assert_eq!(health_status.context_manager_status, "running");
    assert_eq!(health_status.agent_deployment_status, "active");

    Ok(())
}

/// Test error handling and validation
#[tokio::test]
async fn test_error_handling() -> Result<()> {
    // Test different error types
    let network_error = PrimalError::Network("Connection timeout".to_string());
    let auth_error = PrimalError::Authentication("Invalid JWT token".to_string());
    let internal_error = PrimalError::Internal("Configuration missing".to_string());

    // Validate error messages contain our text
    assert!(network_error.to_string().contains("Connection timeout"));
    assert!(auth_error.to_string().contains("Invalid JWT token"));
    assert!(internal_error.to_string().contains("Configuration missing"));

    Ok(())
}

/// Test biomeOS integration creation
#[tokio::test]
async fn test_biomeos_integration_creation() -> Result<()> {
    let integration = SquirrelBiomeOSIntegration::new("test-biome".to_string());

    assert_eq!(integration.biome_id, "test-biome");
    assert!(integration.service_id.starts_with("primal-squirrel-ai-"));
    assert_eq!(integration.health_status.status, "initializing");
    assert_eq!(integration.health_status.ai_engine_status, "starting");
    assert_eq!(integration.health_status.mcp_server_status, "starting");
    assert_eq!(integration.health_status.context_manager_status, "starting");

    Ok(())
}

/// Test utilities for biomeOS operations
pub mod test_utils {
    use super::*;

    /// Create a test biomeOS integration instance
    pub fn create_test_integration() -> SquirrelBiomeOSIntegration {
        SquirrelBiomeOSIntegration::new("test-biome".to_string())
    }

    /// Create test health status with all required fields
    pub fn create_test_health_status() -> HealthStatus {
        HealthStatus {
            status: "healthy".to_string(),
            timestamp: chrono::Utc::now(),
            ai_engine_status: "operational".to_string(),
            mcp_server_status: "active".to_string(),
            context_manager_status: "running".to_string(),
            agent_deployment_status: "active".to_string(),
            active_sessions: 5,
            ai_requests_processed: 100,
            context_states_managed: 20,
            deployed_agents: 3,
        }
    }
}
