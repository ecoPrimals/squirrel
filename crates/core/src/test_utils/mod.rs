//! Test utilities for dependency injection and test mocks
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
use std::collections::HashSet;
use chrono::{Utc};
use uuid::Uuid;

// Fix the imports
use crate::context::manager::ContextManager;
use crate::context::ContextTracker;
use crate::app::context::ContextConfig;
use crate::context_adapter::{ContextAdapterConfig};
use crate::mcp::protocol::{ProtocolConfig};
use crate::mcp::security::{SecurityManager, Credentials, SecurityConfig, Session, Permission, Role};
use crate::mcp::types::SecurityLevel;
use crate::mcp::types::EncryptionFormat;
use crate::mcp::error::Result as MCPResult;
use crate::mcp::context_manager::Context;

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
    /// Flag indicating if the adapter has been initialized
    pub is_initialized: bool,
    /// Configuration for the context adapter
    pub config: ContextAdapterConfig,
    /// Thread-safe storage for the adapter state
    pub state: RwLock<Value>,
}

impl MockContextAdapter {
    /// Creates a new, uninitialized mock context adapter
    pub fn new() -> Self {
        Self {
            is_initialized: false,
            config: ContextAdapterConfig::default(),
            state: RwLock::new(Value::Null),
        }
    }
    
    /// Initializes the adapter
    pub fn initialize(&mut self) -> Result<()> {
        self.is_initialized = true;
        Ok(())
    }

    /// Initializes the adapter with a specific configuration
    pub fn initialize_with_config(&mut self, config: ContextAdapterConfig) -> Result<()> {
        self.config = config;
        self.is_initialized = true;
        Ok(())
    }

    /// Checks if the adapter has been initialized
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    /// Sets the adapter state
    pub async fn set_state(&self, state: Value) -> Result<()> {
        let mut writable = self.state.write().await;
        *writable = state;
        Ok(())
    }

    /// Gets the current adapter state
    pub async fn get_state(&self) -> Result<Value> {
        let readable = self.state.read().await;
        Ok(readable.clone())
    }

    /// Gets the current adapter configuration
    pub async fn get_config(&self) -> Result<ContextAdapterConfig> {
        Ok(self.config.clone())
    }
}

/// Mock protocol adapter for testing
#[derive(Debug)]
pub struct MockProtocolAdapter {
    /// Flag indicating if the adapter has been initialized
    pub is_initialized: bool,
    /// Configuration for the protocol adapter
    pub config: ProtocolConfig,
    /// Thread-safe storage for the adapter state
    pub state: RwLock<Value>,
}

impl MockProtocolAdapter {
    /// Creates a new, uninitialized mock protocol adapter
    pub fn new() -> Self {
        Self {
            is_initialized: false,
            config: ProtocolConfig::default(),
            state: RwLock::new(Value::Null),
        }
    }
    
    /// Initializes the adapter
    pub async fn initialize(&mut self) -> Result<()> {
        self.is_initialized = true;
        Ok(())
    }

    /// Checks if the adapter has been initialized
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    /// Gets the current adapter state
    pub async fn get_state(&self) -> Result<Value> {
        let readable = self.state.read().await;
        Ok(readable.clone())
    }

    /// Sets the adapter state
    pub async fn set_state(&self, state: Value) -> Result<()> {
        let mut writable = self.state.write().await;
        *writable = state;
        Ok(())
    }

    /// Gets the current adapter configuration
    pub async fn get_config(&self) -> Result<ProtocolConfig> {
        Ok(self.config.clone())
    }

    /// Sets the adapter configuration
    pub async fn set_config(&mut self, config: ProtocolConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }
}

/// Mock security manager for testing
#[derive(Debug)]
pub struct MockSecurityManager {
    /// Flag indicating if the security manager is initialized
    pub is_initialized: bool,
    /// The auth result token that will be returned by authenticate()
    pub auth_result: String,
    /// Security manager configuration
    pub config: SecurityConfig,
}

impl MockSecurityManager {
    /// Creates a new mock security manager with default settings
    pub fn new() -> Self {
        Self {
            is_initialized: false,
            auth_result: "mock_token".to_string(), // Default mock token
            config: SecurityConfig {
                min_security_level: SecurityLevel::Standard,
                encryption_format: EncryptionFormat::Aes256Gcm,
                token_validity: 3600,
                max_auth_attempts: 5,
                default_roles: Vec::new(),
            },
        }
    }
    
    /// Sets a custom authentication result token
    pub fn with_auth_result(mut self, result: String) -> Self {
        self.auth_result = result;
        self
    }
}

#[async_trait::async_trait]
impl SecurityManager for MockSecurityManager {
    async fn authenticate(&self, _credentials: &Credentials) -> MCPResult<String> {
        Ok(self.auth_result.clone())
    }
    
    async fn authorize(&self, _token: &str, _required_level: SecurityLevel, _required_permission: Option<&Permission>) -> MCPResult<Session> {
        // Return a mock session
        Ok(Session {
            id: "mock_session".to_string(),
            token: "mock_token".to_string(),
            client_id: "mock_client".to_string(),
            security_level: SecurityLevel::Standard,
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        })
    }
    
    async fn encrypt(&self, _session_id: &str, data: &[u8]) -> MCPResult<Vec<u8>> {
        // Just return the original data in the mock
        Ok(data.to_vec())
    }
    
    async fn decrypt(&self, _session_id: &str, data: &[u8]) -> MCPResult<Vec<u8>> {
        // Just return the original data in the mock
        Ok(data.to_vec())
    }
    
    async fn has_permission(&self, _user_id: &str, _permission: &Permission) -> bool {
        // Mock implementation: always return true
        true
    }
    
    async fn get_user_permissions(&self, _user_id: &str) -> HashSet<Permission> {
        // Return empty permissions set
        HashSet::new()
    }
    
    async fn assign_role(&self, _user_id: String, _role_id: String) -> MCPResult<()> {
        // Mock implementation, just return success
        Ok(())
    }
    
    async fn assign_role_by_name(&self, _user_id: String, _role_name: String) -> MCPResult<()> {
        // Mock implementation, just return success
        Ok(())
    }
    
    async fn create_role(
        &self, 
        name: String,
        description: Option<String>,
        permissions: HashSet<Permission>,
        _parent_roles: HashSet<String>,
    ) -> MCPResult<Role> {
        // Return a mock role
        Ok(Role {
            id: format!("mock_role_{}", name),
            name,
            description,
            permissions,
            parent_roles: HashSet::new(),
        })
    }
    
    async fn create_role_with_id(
        &self, 
        id: String,
        name: String,
        description: Option<String>,
        permissions: HashSet<Permission>,
        _parent_roles: HashSet<String>,
    ) -> MCPResult<Role> {
        // Return a role with the specified ID
        Ok(Role {
            id,
            name,
            description,
            permissions,
            parent_roles: HashSet::new(),
        })
    }
    
    async fn get_role_by_id(&self, id: &str) -> Option<Role> {
        // Return a mock role
        Some(Role {
            id: id.to_string(),
            name: "Mock Role".to_string(),
            description: Some("A mock role for testing".to_string()),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
        })
    }
    
    async fn get_role_by_name(&self, name: &str) -> Option<Role> {
        // Return a mock role
        Some(Role {
            id: "mock_role".to_string(),
            name: name.to_string(),
            description: Some("A mock role for testing".to_string()),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
        })
    }
    
    async fn cleanup_expired_sessions(&self) -> MCPResult<()> {
        // Mock implementation, just return success
        Ok(())
    }
}

/// Factory for creating test components with dependency injection
pub struct TestFactory;

impl TestFactory {
    /// Create a fully mocked test environment with all dependencies
    pub fn create_test_environment() -> std::result::Result<TestEnvironment, Box<dyn Error>> {
        let context_adapter = Arc::new(RwLock::new(MockContextAdapter::new()));
        let protocol_adapter = Arc::new(RwLock::new(MockProtocolAdapter::new()));
        
        let context_manager = Arc::new(ContextManager::new());
        let context_tracker = ContextTracker::new();
        
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
    /// Mock context adapter for testing context functionality
    pub context_adapter: Arc<RwLock<MockContextAdapter>>,
    /// Mock protocol adapter for testing MCP protocol functionality
    pub protocol_adapter: Arc<RwLock<MockProtocolAdapter>>,
    /// Context manager instance for testing context operations
    pub context_manager: Arc<ContextManager>,
    /// Context tracker for testing context state management
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
    
    /// Create a test adapter configuration
    pub fn create_test_adapter_config() -> ContextAdapterConfig {
        ContextAdapterConfig {
            max_contexts: 100,
            ttl_seconds: 3600,
            enable_auto_cleanup: true,
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
            action: Action::Update,
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
            min_security_level: SecurityLevel::Standard,
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
            security_level: SecurityLevel::Standard,
            requested_roles: None,
        }
    }

    /// Creates a basic RBAC setup with reader, editor, and admin roles
    pub fn setup_basic_rbac() -> RBACManager {
        let mut rbac = RBACManager::new();
        let read_perm = Permission {
            id: "perm-read".to_string(),
            name: "Read".to_string(),
            resource: "System".to_string(),
            action: Action::Read,
        };
        
        let write_perm = Permission {
            id: "perm-write".to_string(),
            name: "Write".to_string(),
            resource: "System".to_string(),
            action: Action::Update,
        };
        
        let admin_perm = Permission {
            id: "perm-admin".to_string(),
            name: "Admin".to_string(),
            resource: "System".to_string(),
            action: Action::Admin,
        };
        
        // Create HashSets for permissions
        let mut read_perms = HashSet::new();
        read_perms.insert(read_perm.clone());
        
        let mut editor_perms = HashSet::new();
        editor_perms.insert(read_perm.clone());
        editor_perms.insert(write_perm.clone());
        
        let mut admin_perms = HashSet::new();
        admin_perms.insert(admin_perm.clone());
        
        // Create empty parent roles HashSet
        let empty_parents = HashSet::new();
        
        // Create roles
        let reader_role = rbac.create_role(
            "reader".to_string(), 
            Some("Reader".to_string()), 
            read_perms,
            empty_parents.clone()
        ).unwrap();
        
        // Create a HashSet for reader parent
        let mut reader_parent = HashSet::new();
        reader_parent.insert(reader_role.id.clone());
        
        let editor_role = rbac.create_role(
            "editor".to_string(), 
            Some("Editor".to_string()), 
            editor_perms,
            reader_parent
        ).unwrap();
        
        // Create a HashSet for editor parent
        let mut editor_parent = HashSet::new();
        editor_parent.insert(editor_role.id.clone());
        
        rbac.create_role(
            "admin".to_string(), 
            Some("Admin".to_string()), 
            admin_perms,
            editor_parent
        ).unwrap();
        
        rbac
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

/// Creates mock security options for testing security features
#[allow(unused)]
pub fn create_mock_security_options() -> MCPSecurityOptions {
    MCPSecurityOptions {
        min_security_level: SecurityLevel::Standard,
        enable_encryption: false,
        encryption_format: None,
        allow_anonymous: true,
        auth_required: false,
    }
}

/// Creates mock context attributes for testing context creation
pub fn create_mock_context_attributes() -> MCPContextAttributes {
    MCPContextAttributes {
        name: "Test Context".to_string(),
        security_level: SecurityLevel::Standard,
        ttl: None,
        creation_time: Utc::now(),
        last_updated: Utc::now(),
    }
}

/// Creates a mock security manager for testing security features
pub fn mock_security_manager() -> MockSecurityManager {
    MockSecurityManager::new()
}

/// Mock user for testing user-related functionality
/// 
/// Represents a test user entity for verifying authentication, authorization,
/// and user management operations. Contains all essential user attributes
/// needed for comprehensive testing of user-related security features.
#[derive(Debug, Clone)]
pub struct MockUser {
    /// Unique identifier for the user
    pub id: String,
    /// Display name of the user
    pub name: String,
    /// Email address of the user
    pub email: String,
    /// Security level assigned to the user
    pub security_level: SecurityLevel,
    /// Roles assigned to the user
    pub roles: Vec<String>,
    /// Flag indicating if the user account is active
    pub is_active: bool,
    /// Time of the user's last login
    pub last_login: Option<chrono::DateTime<Utc>>,
}

/// Creates a mock user role with the specified name and description
pub fn create_mock_user_role(name: &str, description: Option<String>) -> MCPRole {
    MCPRole {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.to_string(),
        description,
        permissions: vec![],
        parent_roles: vec![],
    }
}

/// Creates a standard test role for testing RBAC functionality
pub fn create_test_role() -> Role {
    Role {
        id: uuid::Uuid::new_v4().to_string(),
        name: "test-role".to_string(),
        description: Some("A mock role for testing".to_string()),
        permissions: HashSet::new(),
        parent_roles: HashSet::new(),
    }
}

/// Creates an admin test role for testing administrative functions
pub fn create_test_admin_role() -> Role {
    Role {
        id: uuid::Uuid::new_v4().to_string(),
        name: "admin".to_string(),
        description: Some("A mock role for testing".to_string()),
        permissions: HashSet::new(),
        parent_roles: HashSet::new(),
    }
}

/// Creates a test permission for the specified resource
pub fn create_test_permission(resource: &str) -> Permission {
    Permission {
        id: Uuid::new_v4().to_string(),
        name: format!("{}-permission", resource),
        resource: resource.to_string(),
        action: Action::Update,
    }
}

/// Creates a test context with a new context tracker
pub fn create_context_with_tracking() -> (Context, ContextTracker) {
    let context = create_test_context();
    let context_tracker = ContextTracker::new();
    
    (context, context_tracker)
}

/// Creates a test configuration for the context adapter
pub fn create_test_context_adapter_config() -> ContextAdapterConfig {
    ContextAdapterConfig {
        max_contexts: 100,
        ttl_seconds: 3600,
        enable_auto_cleanup: true,
    }
}

/// Security options for MCP protocol
/// 
/// Configuration options that control the security behavior of the MCP protocol,
/// including authentication requirements, encryption settings, and access controls.
#[derive(Debug, Clone)]
pub struct MCPSecurityOptions {
    /// Minimum security level required for operations
    pub min_security_level: SecurityLevel,
    /// Flag indicating if encryption is enabled
    pub enable_encryption: bool,
    /// Format for encryption if enabled
    pub encryption_format: Option<EncryptionFormat>,
    /// Flag indicating if anonymous access is allowed
    pub allow_anonymous: bool,
    /// Flag indicating if authentication is required
    pub auth_required: bool,
}

/// Context attributes for MCP contexts
/// 
/// Defines metadata and configuration settings for MCP context instances,
/// including security settings, naming, and lifecycle information.
/// These attributes are used for testing context creation and management.
#[derive(Debug, Clone)]
pub struct MCPContextAttributes {
    /// Name of the context
    pub name: String,
    /// Security level for the context
    pub security_level: SecurityLevel,
    /// Time-to-live in seconds (optional)
    pub ttl: Option<i64>,
    /// Time when the context was created
    pub creation_time: chrono::DateTime<Utc>,
    /// Time when the context was last updated
    pub last_updated: chrono::DateTime<Utc>,
}

/// Role definition for MCP
/// 
/// Represents a user role in the Machine Context Protocol (MCP) system,
/// defining permissions and hierarchical relationships for access control.
/// Used in testing RBAC functionality.
#[derive(Debug, Clone)]
pub struct MCPRole {
    /// Unique identifier for the role
    pub id: String,
    /// Name of the role
    pub name: String,
    /// Optional description of the role
    pub description: Option<String>,
    /// List of permissions assigned to the role
    pub permissions: Vec<String>,
    /// List of parent role IDs
    pub parent_roles: Vec<String>,
}

/// Creates a test context with default values
pub fn create_test_context() -> Context {
    Context {
        id: Uuid::new_v4(),
        name: "test-context".to_string(),
        data: serde_json::json!({}),
        metadata: None,
        parent_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        expires_at: None,
    }
} 