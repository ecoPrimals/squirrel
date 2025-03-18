#[cfg(test)]
use std::sync::Arc;
#[cfg(test)]
use chrono::Utc;
#[cfg(test)]
use uuid::Uuid;
#[cfg(test)]
use tempfile::tempdir;

use crate::error::Result;
use crate::mcp::context_manager::Context;
use crate::mcp::sync::{
    MCPSync, SyncConfig, StateOperation, StateSyncManager,
    create_mcp_sync, create_mcp_sync_with_deps
};
use crate::mcp::persistence::{MCPPersistence, PersistenceConfig};
use crate::mcp::monitoring::MCPMonitor;

mod sync_tests;
mod state_tests;

// Common test utilities for MCP sync tests

/// Creates a test context for use in tests
fn create_test_context() -> Context {
    Context {
        id: Uuid::new_v4(),
        name: "test_context".to_string(),
        data: serde_json::json!({"test": true}),
        metadata: None,
        parent_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        expires_at: None,
    }
}

/// Creates a default sync config for testing
fn create_test_config() -> SyncConfig {
    SyncConfig {
        sync_interval: 1,
        max_retries: 3,
        timeout_ms: 1000,
        cleanup_older_than_days: 30,
    }
}

/// Creates a test MCPSync instance with dependency injection
async fn create_test_mcp_sync() -> (MCPSync, Arc<MCPPersistence>, Arc<MCPMonitor>, Arc<StateSyncManager>) {
    // ARRANGE: Create dependencies with DI pattern
    let config = create_test_config();
    let persistence = Arc::new(MCPPersistence::new(PersistenceConfig::default()));
    let monitor = Arc::new(MCPMonitor::new().await.unwrap());
    let state_manager = Arc::new(StateSyncManager::new());
    
    // Create and initialize sync instance
    let mut sync = MCPSync::new(
        config,
        persistence.clone(),
        monitor.clone(),
        state_manager.clone()
    );
    
    // Return both the service and its dependencies for verification
    (sync, persistence, monitor, state_manager)
} 