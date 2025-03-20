use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use squirrel_core::error::Result;
use uuid::Uuid;

/// Mock permission for security testing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MockPermission {
    /// Permission name
    pub name: String,
    /// Permission resource
    pub resource: String,
    /// Permission action
    pub action: String,
}

impl MockPermission {
    /// Create a new mock permission
    pub fn new(name: &str, resource: &str, action: &str) -> Self {
        Self {
            name: name.to_string(),
            resource: resource.to_string(),
            action: action.to_string(),
        }
    }
}

/// Mock role for security testing
#[derive(Debug, Clone)]
pub struct MockRole {
    /// Role ID
    pub id: String,
    /// Role name
    pub name: String,
    /// Role permissions
    pub permissions: Vec<MockPermission>,
}

impl MockRole {
    /// Create a new mock role
    pub fn new(name: &str, permissions: Vec<MockPermission>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            permissions,
        }
    }
}

/// Mock token for security testing
#[derive(Debug, Clone)]
pub struct MockToken {
    /// Token value
    pub value: String,
    /// User ID
    pub user_id: String,
    /// Expiry time in seconds since epoch
    pub expires_at: u64,
}

impl MockToken {
    /// Create a new mock token
    pub fn new(user_id: &str, expires_in_seconds: u64) -> Self {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            value: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            expires_at: current_time + expires_in_seconds,
        }
    }
    
    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        current_time >= self.expires_at
    }
}

/// Mock security implementation for testing
#[derive(Debug, Default)]
pub struct MockSecurity {
    /// Whether the mock is initialized
    pub initialized: bool,
    /// Users stored in the mock
    pub users: HashMap<String, String>, // user_id -> password_hash
    /// Roles stored in the mock
    pub roles: Vec<MockRole>,
    /// Active tokens
    pub tokens: HashMap<String, MockToken>, // token_value -> token
}

impl MockSecurity {
    /// Create a new mock security
    pub fn new() -> Self {
        Self {
            initialized: false,
            users: HashMap::new(),
            roles: Vec::new(),
            tokens: HashMap::new(),
        }
    }
    
    /// Initialize the mock security
    pub fn initialize(&mut self) -> Result<()> {
        self.initialized = true;
        Ok(())
    }
    
    /// Check if the mock security is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Add a user to the mock
    pub fn add_user(&mut self, user_id: &str, password: &str) {
        // Simple password hash for testing
        let password_hash = format!("hash_{}", password);
        self.users.insert(user_id.to_string(), password_hash);
    }
    
    /// Add a role to the mock
    pub fn add_role(&mut self, role: MockRole) {
        self.roles.push(role);
    }
    
    /// Authenticate a user and return a token
    pub fn authenticate(&mut self, user_id: &str, password: &str) -> Result<MockToken> {
        // Check if user exists
        let password_hash = self.users.get(user_id)
            .ok_or_else(|| squirrel_core::error::SquirrelError::Security("User not found".into()))?;
        
        // Check password
        if password_hash != &format!("hash_{}", password) {
            return Err(squirrel_core::error::SquirrelError::Security("Invalid password".into()));
        }
        
        // Create token
        let token = MockToken::new(user_id, 3600); // 1 hour expiry
        self.tokens.insert(token.value.clone(), token.clone());
        
        Ok(token)
    }
    
    /// Validate a token
    pub fn validate_token(&self, token_value: &str) -> Result<&MockToken> {
        // Check if token exists
        let token = self.tokens.get(token_value)
            .ok_or_else(|| squirrel_core::error::SquirrelError::Security("Invalid token".into()))?;
        
        // Check if token is expired
        if token.is_expired() {
            return Err(squirrel_core::error::SquirrelError::Security("Token expired".into()));
        }
        
        Ok(token)
    }
}

/// Create a new mock security with shared ownership
pub fn create_mock_security() -> Arc<RwLock<MockSecurity>> {
    Arc::new(RwLock::new(MockSecurity::new()))
}

/// Create an initialized mock security with shared ownership
pub fn create_initialized_mock_security() -> Result<Arc<RwLock<MockSecurity>>> {
    let security = Arc::new(RwLock::new(MockSecurity::new()));
    {
        let mut sec = security.try_write().unwrap();
        sec.initialize()?;
    }
    Ok(security)
} 