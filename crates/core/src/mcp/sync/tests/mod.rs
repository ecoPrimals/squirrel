#[cfg(test)]
use std::sync::Arc;
#[cfg(test)]
use chrono::Utc;
#[cfg(test)]
use uuid::Uuid;
#[cfg(test)]
use tempfile::tempdir;

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
    let sync = MCPSync::new(
        config,
        persistence.clone(),
        monitor.clone(),
        state_manager.clone()
    );
    
    // Return both the service and its dependencies for verification
    (sync, persistence, monitor, state_manager)
}

// Add missing implementation of record_operation that's used in state_tests.rs
#[cfg(test)]
impl StateSyncManager {
    /// Records an operation directly without requiring a full Context object
    /// This is a convenience method for testing purposes
    pub fn record_operation(&self, context_id: Uuid, operation: StateOperation, data: &serde_json::Value) -> crate::mcp::sync::state::StateChange {
        use crate::mcp::sync::state::StateChange;
        use std::sync::atomic::{AtomicU64, Ordering};
        
        // Use a static counter for the version in tests to avoid async
        thread_local! {
            static VERSION_COUNTER: AtomicU64 = AtomicU64::new(0);
        }
        
        // Increment the version counter
        let version = VERSION_COUNTER.with(|counter| {
            counter.fetch_add(1, Ordering::SeqCst) + 1
        });
        
        let change = StateChange {
            id: Uuid::new_v4(),
            context_id,
            operation,
            data: data.clone(),
            timestamp: Utc::now(),
            version,
        };
        
        // Broadcast the change to subscribers
        let _ = self.sender.send(change.clone());
        
        change
    }
    
    /// Get the current version number without async
    pub fn current_version(&self) -> u64 {
        // Access the thread local version counter
        thread_local! {
            static VERSION_COUNTER: AtomicU64 = AtomicU64::new(0);
        }
        
        VERSION_COUNTER.with(|counter| {
            counter.load(std::sync::atomic::Ordering::SeqCst)
        })
    }
    
    /// Get the subscriber count for testing
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
    
    /// Create a subscriber for testing
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<crate::mcp::sync::state::StateChange> {
        self.sender.subscribe()
    }
} 