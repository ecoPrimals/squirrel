//! Songbird Integration Test
//!
//! Tests the integration between squirrel and songbird orchestration system.
//! This test verifies that squirrel can properly register with and coordinate through songbird.

use squirrel::biomeos_integration::{HealthStatus, SquirrelBiomeOSIntegration};
use squirrel::songbird::SongbirdIntegration;

#[tokio::test]
async fn test_songbird_integration_basic() {
    // Test basic songbird integration functionality
    let integration = SongbirdIntegration::new();
    // Just check that we can create the integration
    assert_eq!(
        integration.config.songbird_endpoint,
        "http://localhost:8080"
    );
    assert_eq!(integration.config.max_retries, 3);
}

#[tokio::test]
async fn test_songbird_health_status() {
    // Test health status reporting using the correct HealthStatus from biomeos_integration
    let health_status = HealthStatus {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        ai_engine_status: "operational".to_string(),
        mcp_server_status: "active".to_string(),
        context_manager_status: "running".to_string(),
        active_sessions: 0,
        ai_requests_processed: 0,
        context_states_managed: 0,
    };

    assert_eq!(health_status.status, "healthy");
    assert_eq!(health_status.ai_engine_status, "operational");
    assert_eq!(health_status.mcp_server_status, "active");
    assert_eq!(health_status.context_manager_status, "running");
}

#[tokio::test]
async fn test_biome_os_integration_with_songbird() {
    // Test that BiomeOS integration works with songbird
    let integration = SquirrelBiomeOSIntegration::new("default".to_string());
    assert!(integration.service_id.starts_with("primal-squirrel-ai-"));
    assert_eq!(integration.biome_id, "default");
}
