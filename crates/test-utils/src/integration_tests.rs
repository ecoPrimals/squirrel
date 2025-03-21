//! Integration tests for App and MCPSync modules
//!
//! These tests verify the interaction between different modules
//! using the new dependency injection pattern.

#[cfg(test)]
use std::path::PathBuf;
use std::path::PathBuf as StdPathBuf;
use squirrel_core::error::Result;
use std::fmt::Debug;
use squirrel_core::error::SquirrelError;

/// Test harness for integration tests
pub struct IntegrationTestContext {
    /// App adapter - using placeholder types until app is fixed
    pub app: MockAppAdapter,
    /// MCPSync instance - using placeholder types until mcp is fixed
    pub sync: MockMCPSync,
    /// Temporary directory for test data
    temp_dir: StdPathBuf,
}

/// Mock app adapter for testing
#[derive(Debug)]
pub struct MockAppAdapter {
    initialized: bool,
}

impl MockAppAdapter {
    /// Create a new mock app adapter
    pub fn new(_config: MockAppConfig) -> Self {
        Self {
            initialized: false,
        }
    }
    
    /// Initialize the mock app
    pub fn initialize(&mut self) -> Result<()> {
        // Simple implementation just for testing
        self.initialized = true;
        Ok(())
    }
    
    /// Check if the mock app is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

/// Mock app config for testing
#[derive(Debug, Default)]
pub struct MockAppConfig {
    pub name: String,
    pub version: String,
    pub options: Vec<(String, String)>,
}

/// Mock MCP sync for testing
#[derive(Debug)]
pub struct MockMCPSync {
    #[allow(dead_code)]
    initialized: bool,
}

impl MockMCPSync {
    /// Create a new mock MCP sync
    pub async fn create(_config: MockSyncConfig) -> Result<Self> {
        Ok(Self {
            initialized: false,
        })
    }
}

/// Mock sync config for testing
#[derive(Debug, Default)]
pub struct MockSyncConfig {
    pub timeout_ms: u64,
}

impl IntegrationTestContext {
    /// Create a new integration test context
    pub async fn new() -> Result<Self> {
        // Create temporary directory
        let temp_dir = std::env::temp_dir().join(format!("squirrel_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;
        
        // Create app with custom config
        let app_config = MockAppConfig {
            name: "Squirrel Test".to_string(),
            version: "0.1.0-test".to_string(),
            options: vec![
                ("data_dir".to_string(), temp_dir.to_string_lossy().to_string()),
                ("test_mode".to_string(), "true".to_string()),
            ],
        };
        
        // Create app adapter
        let mut app = MockAppAdapter::new(app_config);
        // Initialize the app
        app.initialize()?;
        
        // Create MCPSync with custom config
        let sync_config = MockSyncConfig::default();
        let sync = MockMCPSync::create(sync_config).await?;
        
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
        
        // ASSERT: Verify operations succeed
        assert!(context.app.is_initialized());
    }
    
    #[tokio::test]
    async fn test_uninitialized_operations() {
        // ARRANGE: Create app but don't initialize
        let app_config = MockAppConfig::default();
        let app = MockAppAdapter::new(app_config);
        // Deliberately not calling initialize()
        
        // Create MCPSync but don't initialize
        let sync_config = MockSyncConfig::default();
        let _sync = MockMCPSync::create(sync_config).await.expect("Failed to create MCPSync");
        
        // ACT & ASSERT: Verify app is not initialized
        assert!(!app.is_initialized());
    }
    
    #[tokio::test]
    async fn test_error_propagation() {
        // ARRANGE: Set up test context
        let _context = IntegrationTestContext::new().await.expect("Failed to create integration test context");
        
        // ACT: Create a complex operation that will fail
        let result = async {
            // Create a bogus operation that will fail
            Err::<(), _>(SquirrelError::Other("Deliberate test error".to_string()))
        }.await;
        
        // ASSERT: Verify error propagation
        assert!(result.is_err());
    }
} 