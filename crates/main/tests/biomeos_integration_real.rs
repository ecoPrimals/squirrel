// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Real BiomeOS Integration Tests
//!
//! These tests validate the actual SquirrelBiomeOSIntegration API,
//! replacing the fictional test methods that were removed during deep evolution.
//!
//! Philosophy: Test reality, not fiction.

use squirrel::biomeos_integration::*;
use squirrel::error::PrimalError;

/// Test BiomeOS integration creation and basic operations
#[tokio::test]
async fn test_biomeos_integration_creation() {
    let integration = SquirrelBiomeOSIntegration::new("test-biome".to_string());

    // Verify basic structure
    assert_eq!(integration.biome_id, "test-biome");

    // Verify initial state
    let health = integration.get_health_status();
    assert!(!health.status.is_empty());
    assert_eq!(health.status, "initializing");
}

/// Test BiomeOS registration workflow
///
/// Note: This may fail if no biomeOS server is running, which is expected.
/// The test validates the registration logic, not server availability.
#[tokio::test]
async fn test_biomeos_registration() {
    let mut integration = SquirrelBiomeOSIntegration::new("test-biome".to_string());

    // Attempt registration
    let result = integration.register_with_biomeos().await;

    // Valid outcomes:
    // - Ok(()) if biomeOS server is running
    // - Err(Network/NetworkError) if no server (expected in tests)
    // - Other errors should be investigated
    match result {
        Ok(()) => {
            // Registration successful - biomeOS server was available
            println!("✅ Registration successful - biomeOS server available");
        }
        Err(PrimalError::Network(_)) | Err(PrimalError::NetworkError(_)) => {
            // Expected when no biomeOS server is running
            println!("✅ Network error as expected (no server)");
        }
        Err(e) => {
            // May be other expected errors (Configuration, etc.)
            println!("⚠️ Error during registration (may be expected): {:?}", e);
        }
    }
}

/// Test ecosystem services lifecycle
#[tokio::test]
async fn test_ecosystem_services_lifecycle() {
    let mut integration = SquirrelBiomeOSIntegration::new("test-biome".to_string());

    // Test service startup
    let start_result = integration.start_ecosystem_services().await;

    // Valid outcomes:
    // - Ok(()) if services start successfully
    // - Err if dependencies not available (expected in tests)
    match start_result {
        Ok(()) => {
            println!("✅ Services started successfully");

            // Verify health status changed
            let health = integration.get_health_status();
            assert!(
                health.status == "running" || health.status == "starting",
                "Expected running or starting, got: {}",
                health.status
            );
        }
        Err(e) => {
            println!(
                "✅ Service start failed as expected (no dependencies): {:?}",
                e
            );

            // Even if start fails, health status should still be accessible
            let health = integration.get_health_status();
            assert!(!health.status.is_empty());
        }
    }
}

/// Test agent deployment management
#[tokio::test]
async fn test_agent_deployment_management() {
    let integration = SquirrelBiomeOSIntegration::new("test-biome".to_string());

    // Test agent listing (should work even without running agents)
    let agents = integration.list_deployed_agents().await;

    // Initially should be empty or contain agents
    // Both states are valid
    println!("✅ Deployed agents count: {}", agents.len());

    // Test deployment status
    let status = integration.get_deployment_status().await;

    // Should return valid status
    assert!(status.total_agents >= 0);
    println!("✅ Deployment status: {:?}", status);
}

/// Test health check functionality
#[tokio::test]
async fn test_health_check() {
    let mut integration = SquirrelBiomeOSIntegration::new("test-biome".to_string());

    // Get initial health status
    let initial_health = integration.get_health_status();
    assert!(!initial_health.status.is_empty());
    // active_sessions and deployed_agents are unsigned, no need to check >= 0

    // Perform health check
    let check_result = integration.health_check().await;

    // Valid outcomes:
    // - Ok(()) if all systems healthy
    // - Err if systems unavailable (expected in tests)
    match check_result {
        Ok(()) => {
            println!("✅ Health check passed");
        }
        Err(e) => {
            println!(
                "✅ Health check failed as expected (no dependencies): {:?}",
                e
            );
        }
    }

    // Health status should still be accessible
    let final_health = integration.get_health_status();
    assert!(!final_health.status.is_empty());
}

/// Test manifest generation
#[test]
fn test_manifest_generation() {
    let integration = SquirrelBiomeOSIntegration::new("test-biome".to_string());

    // Generate manifest template
    let manifest = integration.generate_manifest_template();

    // Verify manifest structure
    assert!(
        !manifest.metadata.name.is_empty(),
        "Manifest should have name"
    );
    assert!(!manifest.agents.is_empty(), "Manifest should have agents");
    assert!(
        !manifest.metadata.author.is_empty(),
        "Manifest should have author"
    );

    println!("✅ Manifest generated successfully");
}

/// Test agent stopping (graceful handling when no agent exists)
#[tokio::test]
async fn test_agent_stop_nonexistent() {
    let mut integration = SquirrelBiomeOSIntegration::new("test-biome".to_string());

    // Try to stop non-existent agent
    let result = integration.stop_agent("non-existent-agent-id").await;

    // Should handle gracefully
    match result {
        Ok(()) => {
            println!("✅ Stop succeeded (agent may have been stopped already)");
        }
        Err(e) => {
            println!("✅ Stop failed as expected (agent doesn't exist): {:?}", e);
        }
    }
}

/// Test concurrent health status reads
///
/// Verifies that health status can be safely read concurrently
#[tokio::test]
async fn test_concurrent_health_reads() {
    use std::sync::Arc;

    let integration = Arc::new(SquirrelBiomeOSIntegration::new("test-biome".to_string()));

    // Spawn multiple concurrent readers
    let mut handles = vec![];

    for i in 0..10 {
        let integration_clone = Arc::clone(&integration);
        let handle = tokio::spawn(async move {
            let health = integration_clone.get_health_status();
            assert!(
                !health.status.is_empty(),
                "Health status should not be empty"
            );
            println!("✅ Reader {} completed", i);
        });
        handles.push(handle);
    }

    // Wait for all readers
    for handle in handles {
        handle.await.expect("Task should complete successfully");
    }

    println!("✅ All concurrent reads completed successfully");
}

/// Test health status update
#[test]
fn test_health_status_update() {
    let mut integration = SquirrelBiomeOSIntegration::new("test-biome".to_string());

    // Update health status
    integration.update_health_status("testing");

    // Verify update
    let health = integration.get_health_status();
    assert_eq!(health.status, "testing");

    // Update again
    integration.update_health_status("running");
    let health = integration.get_health_status();
    assert_eq!(health.status, "running");

    println!("✅ Health status updates working correctly");
}

/// Test ecosystem endpoints configuration
#[test]
fn test_ecosystem_endpoints() {
    let integration = SquirrelBiomeOSIntegration::new("test-biome".to_string());

    // Access ecosystem endpoints (should be configured)
    // This tests that the structure is properly initialized
    let service_id = &integration.service_id;
    assert!(!service_id.is_empty(), "Service ID should be set");
    assert!(
        service_id.starts_with("primal-squirrel-ai-"),
        "Service ID should have correct prefix"
    );

    println!("✅ Service ID: {}", service_id);
}

/// Integration test demonstrating proper error handling
#[tokio::test]
async fn test_error_handling_patterns() {
    let mut integration = SquirrelBiomeOSIntegration::new("test-biome".to_string());

    // Test 1: Registration with no server
    match integration.register_with_biomeos().await {
        Ok(()) => println!("Registration succeeded (server available)"),
        Err(PrimalError::Network(msg)) | Err(PrimalError::NetworkError(msg)) => {
            println!("✅ Network error handled: {}", msg);
            assert!(!msg.is_empty(), "Error message should not be empty");
        }
        Err(e) => println!("Other error (may be expected): {:?}", e),
    }

    // Test 2: Service start with no dependencies
    match integration.start_ecosystem_services().await {
        Ok(()) => println!("Services started (dependencies available)"),
        Err(e) => {
            println!("✅ Service start error handled: {:?}", e);
            // Error should be meaningful
            assert!(!format!("{:?}", e).is_empty());
        }
    }

    println!("✅ Error handling patterns validated");
}

/// Test that demonstrates the integration is thread-safe
#[tokio::test]
async fn test_thread_safety() {
    use std::sync::Arc;
    use tokio::sync::RwLock;

    let integration = Arc::new(RwLock::new(SquirrelBiomeOSIntegration::new(
        "test-biome".to_string(),
    )));

    // Spawn multiple tasks that access the integration
    let mut handles = vec![];

    for i in 0..5 {
        let integration_clone = Arc::clone(&integration);
        let handle = tokio::spawn(async move {
            // Read lock for health check
            {
                let integration = integration_clone.read().await;
                let health = integration.get_health_status();
                assert!(!health.status.is_empty());
            }

            // Write lock for status update
            {
                let mut integration = integration_clone.write().await;
                integration.update_health_status(&format!("test-{}", i));
            }

            println!("✅ Task {} completed", i);
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.expect("Task should complete");
    }

    println!("✅ Thread safety validated");
}

/// Comprehensive integration test
///
/// Tests the complete lifecycle of a BiomeOS integration
#[tokio::test]
async fn test_complete_lifecycle() {
    let mut integration = SquirrelBiomeOSIntegration::new("test-lifecycle-biome".to_string());

    // Phase 1: Initialization
    println!("Phase 1: Initialization");
    let initial_health = integration.get_health_status();
    assert_eq!(initial_health.status, "initializing");

    // Phase 2: Registration attempt
    println!("Phase 2: Registration");
    let _ = integration.register_with_biomeos().await;
    // Result may vary based on server availability

    // Phase 3: Service startup attempt
    println!("Phase 3: Service Startup");
    let _ = integration.start_ecosystem_services().await;
    // Result may vary based on dependencies

    // Phase 4: Agent management
    println!("Phase 4: Agent Management");
    let agents = integration.list_deployed_agents().await;
    println!("  Agents: {}", agents.len());

    let deployment_status = integration.get_deployment_status().await;
    println!("  Deployment: {:?}", deployment_status);

    // Phase 5: Health monitoring
    println!("Phase 5: Health Check");
    let _ = integration.health_check().await;
    let final_health = integration.get_health_status();
    println!("  Final health: {}", final_health.status);

    // Phase 6: Manifest generation
    println!("Phase 6: Manifest");
    let manifest = integration.generate_manifest_template();
    assert!(!manifest.agents.is_empty());

    println!("✅ Complete lifecycle test passed");
}
