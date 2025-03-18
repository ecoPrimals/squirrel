// Test utilities for dependency injection and test mocks
//! 
//! This module provides utilities for testing with dependency injection.
//! It includes mock implementations of core dependencies and factory functions
//! to create test harnesses.

use std::sync::Arc;
use std::error::Error;
use std::fmt;
use tokio::sync::RwLock;
use serde_json::Value;

use crate::context::{ContextManager, ContextTracker, ContextConfig};
use crate::context_adapter::{ContextAdapter, ContextAdapterConfig};
use crate::mcp::protocol::{ProtocolAdapter, ProtocolConfig};
use crate::mcp::sync::{SyncConfig, MCPSync};
use crate::error::SquirrelError;
use crate::mcp::security::{SecurityManager, Credentials, SecurityConfig};
use crate::mcp::types::{SecurityLevel, EncryptionFormat};

/// Error type for test failures
#[derive(Debug)]
pub struct TestError {
    pub message: String,
}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Test error: {}", self.message)
    }
}

impl Error for TestError {}

impl From<String> for TestError {
    fn from(message: String) -> Self {
        Self { message }
    }
}

impl From<&str> for TestError {
    fn from(message: &str) -> Self {
        Self { message: message.to_owned() }
    }
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
                security_level: SecurityLevel::None,
                encryption_format: EncryptionFormat::None,
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
pub struct TestData;

impl TestData {
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