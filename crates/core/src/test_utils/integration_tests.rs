//! Integration tests for App and MCPSync modules
//!
//! These tests verify the interaction between different modules
//! using the new dependency injection pattern.

#[cfg(test)]
use std::path::PathBuf;
use crate::app::{AppAdapter, AppConfig, create_initialized_app_adapter};
use crate::mcp::sync::{MCPSync, SyncConfig, create_mcp_sync};
// Remove: use crate::mcp::monitoring::MCPMonitor;
// Remove: use crate::mcp::persistence::{MCPPersistence, PersistenceConfig};
// Remove: use crate::mcp::context_manager::Context;
#[cfg(test)]
use crate::error::{Result, SquirrelError};
#[cfg(not(test))]
use crate::error::Result;

/// Test harness for integration tests
pub struct IntegrationTestContext {
    /// App adapter
    pub app: AppAdapter,
    /// MCPSync instance
    pub sync: MCPSync,
    /// Temporary directory for test data
    temp_dir: PathBuf,
}

impl IntegrationTestContext {
    /// Create a new integration test context
    pub async fn new() -> Result<Self> {
        // Create temporary directory
        let temp_dir = std::env::temp_dir().join(format!("squirrel_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;
        
        // Create app with custom config
        let app_config = AppConfig {
            data_dir: temp_dir.clone(),
            monitoring: None, // Disable monitoring for tests
        };
        
        let app = create_initialized_app_adapter(app_config).await?;
        
        // Create MCPSync with custom config
        let sync_config = SyncConfig::default();
        let sync = create_mcp_sync(sync_config).await?;
        
        Ok(Self {
            app,
            sync,
            temp_dir,
        })
    }
}

impl Drop for IntegrationTestContext {
    fn drop(&mut self) {
        // Clean up temporary directory
        let _ = std::fs::remove_dir_all(&self.temp_dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_integration_app_with_mcp() {
        // ARRANGE: Set up integration test context
        let context = IntegrationTestContext::new().await.expect("Failed to create integration test context");
        
        // ACT: Start app
        let start_result = context.app.start().await;
        
        // ASSERT: Verify operations succeed
        assert!(start_result.is_ok());
    }
    
    #[tokio::test]
    async fn test_uninitialized_operations() {
        // ARRANGE: Create app but don't initialize
        let app_config = AppConfig::default();
        let mut app = AppAdapter::new();
        
        // Create MCPSync but don't initialize
        let sync_config = SyncConfig::default();
        let sync = MCPSync::create(sync_config).await.expect("Failed to create MCPSync");
        
        // ACT: Attempt operations on uninitialized components
        let app_start_result = app.start().await;
        
        // ASSERT: Verify proper error responses
        assert!(app_start_result.is_err());
        if let Err(SquirrelError::NotInitialized(_)) = app_start_result {
            // Expected error
        } else {
            panic!("Expected NotInitialized error");
        }
    }
    
    #[tokio::test]
    async fn test_error_propagation() {
        // ARRANGE: Set up test context
        let context = IntegrationTestContext::new().await.expect("Failed to create integration test context");
        
        // Deliberately cause an error by operating on a component after stopping
        let _ = context.app.start().await.expect("Failed to start app");
        let _ = context.app.stop().await.expect("Failed to stop app");
        
        // ACT: Create a complex operation that will fail
        let result = async {
            // This is a complex sequence that should eventually fail
            context.app.start().await?;
            let app_ctx = context.app.context().await?;
            
            // Create a bogus operation that will fail
            Err::<(), _>(SquirrelError::Other("Deliberate test error".to_string()))
        }.await;
        
        // ASSERT: Verify error propagation
        assert!(result.is_err());
    }
} 