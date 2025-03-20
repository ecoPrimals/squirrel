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
    /// Reset the version counter for testing
    pub fn reset_version_counter() {
        use std::sync::atomic::{AtomicU64, Ordering};
        
        thread_local! {
            static VERSION_COUNTER: AtomicU64 = AtomicU64::new(0);
        }
        
        VERSION_COUNTER.with(|counter| {
            counter.store(0, Ordering::SeqCst);
        });
    }

    /// Get the current version counter value for debugging purposes
    pub fn current_version_debug() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        
        thread_local! {
            static VERSION_COUNTER: AtomicU64 = AtomicU64::new(0);
        }
        
        VERSION_COUNTER.with(|counter| {
            counter.load(Ordering::SeqCst)
        })
    }

    /// Records an operation directly without requiring a full Context object
    /// This is a convenience method for testing purposes
    pub async fn record_operation(&self, context_id: Uuid, operation: StateOperation, data: &serde_json::Value) -> crate::mcp::sync::state::StateChange {
        use crate::mcp::sync::state::StateChange;
        
        // Get current version
        let current_version = self.get_current_version().await.unwrap_or(0);
        // Increment version by 1
        let new_version = current_version + 1;
        
        // Create a change with the incremented version
        let change = StateChange {
            id: Uuid::new_v4(),
            context_id,
            operation,
            data: data.clone(),
            timestamp: Utc::now(),
            version: new_version,
        };
        
        // Apply the change to set the internal version
        self.apply_change(change.clone()).await.expect("Failed to apply change");
        
        change
    }
    
    /// Get the current version number
    pub async fn current_version(&self) -> u64 {
        // Use the public get_current_version method
        self.get_current_version().await.unwrap_or(0)
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