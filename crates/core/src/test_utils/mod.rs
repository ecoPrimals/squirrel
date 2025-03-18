// Test utilities for dependency injection and test mocks
//! 
//! This module provides utilities for testing with dependency injection.
//! It includes mock implementations of core dependencies and factory functions
//! to create test harnesses.

#[cfg(test)]
use std::sync::Arc;
#[cfg(test)]
use std::error::Error;
use tokio::sync::RwLock;
use serde_json::Value;
use std::path::PathBuf;
use tempfile::TempDir;
use crate::error::Result;

use crate::context::{ContextManager, ContextTracker, ContextConfig};
use crate::context_adapter::{ContextAdapter, ContextAdapterConfig};
use crate::mcp::protocol::{ProtocolAdapter, ProtocolConfig};
use crate::error::SquirrelError;
use crate::mcp::security::{SecurityManager, Credentials, SecurityConfig};
use crate::mcp::types::{SecurityLevel, EncryptionFormat};

// Integration tests module
pub mod integration_tests;
pub use integration_tests::IntegrationTestContext;

/// Test data for testing persistence.
#[derive(Debug, Clone, PartialEq)]
pub struct TestData {
    /// The name of the test data.
    pub name: String,
    /// The value of the test data.
    pub value: String,
}

/// Error type for test utilities.
#[derive(Debug, thiserror::Error)]
pub enum TestError {
    /// Invalid test data.
    #[error("Invalid test data: {0}")]
    InvalidData(String),
    /// Error during test initialization.
    #[error("Test initialization error: {0}")]
    InitError(String),
    /// Error during test execution.
    #[error("Test execution error: {0}")]
    ExecError(String),
}

/// Mock context adapter for testing
#[derive(Debug)]
pub struct MockContextAdapter {
    pub is_initialized: bool,
    pub config: ContextAdapterConfig,
    pub state: RwLock<Value>,
}

impl MockContextAdapter {
    pub fn new() -> Self {
        Self {
            is_initialized: false,
            config: ContextAdapterConfig::default(),
            state: RwLock::new(Value::Null),
        }
    }
}

impl ContextAdapter for MockContextAdapter {
    fn initialize(&mut self) -> Result<(), SquirrelError> {
        self.is_initialized = true;
        Ok(())
    }

    fn initialize_with_config(&mut self, config: ContextAdapterConfig) -> Result<(), SquirrelError> {
        self.config = config;
        self.is_initialized = true;
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    async fn set_state(&self, state: Value) -> Result<(), SquirrelError> {
        let mut writable = self.state.write().await;
        *writable = state;
        Ok(())
    }

    async fn get_state(&self) -> Result<Value, SquirrelError> {
        let readable = self.state.read().await;
        Ok(readable.clone())
    }

    async fn get_config(&self) -> Result<ContextAdapterConfig, SquirrelError> {
        Ok(self.config.clone())
    }
}

/// Mock protocol adapter for testing
#[derive(Debug)]
pub struct MockProtocolAdapter {
    pub is_initialized: bool,
    pub config: ProtocolConfig,
    pub state: RwLock<Value>,
}

impl MockProtocolAdapter {
    pub fn new() -> Self {
        Self {
            is_initialized: false,
            config: ProtocolConfig::default(),
            state: RwLock::new(Value::Null),
        }
    }
}

impl ProtocolAdapter for MockProtocolAdapter {
    async fn initialize(&mut self) -> Result<(), SquirrelError> {
        self.is_initialized = true;
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    async fn get_state(&self) -> Result<Value, SquirrelError> {
        let readable = self.state.read().await;
        Ok(readable.clone())
    }

    async fn set_state(&self, state: Value) -> Result<(), SquirrelError> {
        let mut writable = self.state.write().await;
        *writable = state;
        Ok(())
    }

    async fn get_config(&self) -> Result<ProtocolConfig, SquirrelError> {
        Ok(self.config.clone())
    }

    async fn set_config(&mut self, config: ProtocolConfig) -> Result<(), SquirrelError> {
        self.config = config;
        Ok(())
    }
}

/// Mock security manager for testing
#[derive(Debug)]
pub struct MockSecurityManager {
    pub is_initialized: bool,
    pub auth_result: bool,
    pub config: SecurityConfig,
}

impl MockSecurityManager {
    pub fn new() -> Self {
        Self {
            is_initialized: false,
            auth_result: true, // Default to successful authentication
            config: SecurityConfig {
                min_security_level: SecurityLevel::Medium,
                encryption_format: EncryptionFormat::Aes256Gcm,
                token_validity: 3600,
                max_auth_attempts: 5,
                default_roles: Vec::new(),
            },
        }
    }
    
    pub fn with_auth_result(mut self, result: bool) -> Self {
        self.auth_result = result;
        self
    }
}

impl SecurityManager for MockSecurityManager {
    fn initialize(&mut self) -> Result<(), SquirrelError> {
        self.is_initialized = true;
        Ok(())
    }
    
    fn authenticate(&self, _credentials: &Credentials) -> Result<bool, SquirrelError> {
        Ok(self.auth_result)
    }
    
    fn get_config(&self) -> Result<SecurityConfig, SquirrelError> {
        Ok(self.config.clone())
    }
    
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SquirrelError> {
        // Just return the original data in the mock
        Ok(data.to_vec())
    }
    
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, SquirrelError> {
        // Just return the original data in the mock
        Ok(data.to_vec())
    }
}

/// Factory for creating test components with dependency injection
pub struct TestFactory;

impl TestFactory {
    /// Create a fully mocked test environment with all dependencies
    pub fn create_test_environment() -> Result<TestEnvironment, Box<dyn Error>> {
        let context_adapter = Arc::new(RwLock::new(MockContextAdapter::new()));
        let protocol_adapter = Arc::new(RwLock::new(MockProtocolAdapter::new()));
        
        let context_manager = Arc::new(ContextManager::new());
        let context_tracker = ContextTracker::new(context_manager.clone());
        
        Ok(TestEnvironment {
            context_adapter,
            protocol_adapter,
            context_manager,
            context_tracker,
        })
    }
}

/// Test environment containing mocked dependencies
pub struct TestEnvironment {
    pub context_adapter: Arc<RwLock<MockContextAdapter>>,
    pub protocol_adapter: Arc<RwLock<MockProtocolAdapter>>,
    pub context_manager: Arc<ContextManager>,
    pub context_tracker: ContextTracker,
}

/// Test data generator for common test scenarios
pub struct TestDataGenerator;

impl TestDataGenerator {
    /// Create a simple JSON test state
    pub fn create_test_state() -> Value {
        serde_json::json!({
            "test": true,
            "value": "test data",
            "number": 42,
            "nested": {
                "field": "nested value",
                "array": [1, 2, 3]
            }
        })
    }
    
    /// Create a simple context configuration
    pub fn create_test_context_config() -> ContextConfig {
        ContextConfig {
            persistence_enabled: true,
            auto_save: true,
            history_size: 10,
        }
    }
    
    /// Create a simple adapter configuration
    pub fn create_test_adapter_config() -> ContextAdapterConfig {
        ContextAdapterConfig {
            persistence_path: Some("/tmp/test".to_string()),
            auto_save_interval: Some(60),
        }
    }
}

/// Security test utilities
pub mod security {
    use crate::mcp::security::{RBACManager, Permission, Action, SecurityConfig, Credentials};
    use crate::mcp::types::{SecurityLevel, EncryptionFormat};
    
    /// Creates a test RBAC manager with predefined roles and permissions
    pub fn create_test_rbac_manager() -> RBACManager {
        let mut rbac = RBACManager::new();
        
        // Create base permissions
        let read_perm = Permission {
            id: "perm-read".to_string(),
            name: "Read".to_string(),
            resource: "Document".to_string(),
            action: Action::Read,
        };
        
        let write_perm = Permission {
            id: "perm-write".to_string(),
            name: "Write".to_string(),
            resource: "Document".to_string(),
            action: Action::Write,
        };
        
        let admin_perm = Permission {
            id: "perm-admin".to_string(),
            name: "Admin".to_string(),
            resource: "System".to_string(),
            action: Action::Admin,
        };
        
        // Create roles
        rbac.create_role("reader", "Reader", vec![read_perm.clone()]);
        
        let editor_id = rbac.create_role_with_parent(
            "editor", 
            "Editor", 
            vec![write_perm.clone()], 
            vec![String::from("reader")]
        );
        
        rbac.create_role_with_parent(
            "admin", 
            "Admin", 
            vec![admin_perm.clone()], 
            vec![editor_id]
        );
        
        rbac
    }
    
    /// Creates a test security config
    pub fn create_test_security_config() -> SecurityConfig {
        SecurityConfig {
            min_security_level: SecurityLevel::Medium,
            encryption_format: EncryptionFormat::Aes256Gcm,
            token_validity: 3600,
            max_auth_attempts: 5,
            default_roles: Vec::new(),
        }
    }
    
    /// Creates test credentials
    pub fn create_test_credentials(client_id: &str, client_secret: &str) -> Credentials {
        Credentials {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            security_level: SecurityLevel::Medium,
            requested_roles: None,
        }
    }
}

/// Creates a temporary test environment.
pub fn create_test_env() -> Result<TempDir> {
    let temp_dir = tempfile::tempdir()
        .map_err(|e| crate::error::SquirrelError::other(format!("Failed to create temp dir: {}", e)))?;
    Ok(temp_dir)
}

/// Creates a test file in the given directory.
pub fn create_test_file(dir: &TempDir, name: &str, content: &str) -> Result<PathBuf> {
    let path = dir.path().join(name);
    std::fs::write(&path, content)
        .map_err(|e| crate::error::SquirrelError::other(format!("Failed to write test file: {}", e)))?;
    Ok(path)
} 