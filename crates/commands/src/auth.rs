//! Command authentication system for Squirrel
//!
//! This module provides functionality for authenticating and authorizing 
//! command execution to ensure secure operation.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::CommandError;
use crate::registry::Command;

/// Result type for authentication operations
pub type AuthResult<T> = Result<T, CommandError>;

/// Permission level for command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// No permissions, can't execute any commands
    None,
    
    /// Read-only permissions, can only execute read commands
    ReadOnly,
    
    /// Standard permissions, can execute most commands
    Standard,
    
    /// Administrative permissions, can execute all commands
    Admin,
}

impl fmt::Display for PermissionLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::ReadOnly => write!(f, "ReadOnly"),
            Self::Standard => write!(f, "Standard"),
            Self::Admin => write!(f, "Admin"),
        }
    }
}

/// User identity and permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User identifier
    pub id: String,
    
    /// User name
    pub name: String,
    
    /// User permission level
    pub permission_level: PermissionLevel,
    
    /// Additional metadata about the user
    pub metadata: HashMap<String, String>,
}

impl User {
    /// Creates a new user with the given permission level
    pub fn new(id: impl Into<String>, name: impl Into<String>, permission_level: PermissionLevel) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            permission_level,
            metadata: HashMap::new(),
        }
    }
    
    /// Creates a new user with administrative permissions
    pub fn admin(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(id, name, PermissionLevel::Admin)
    }
    
    /// Creates a new user with standard permissions
    pub fn standard(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(id, name, PermissionLevel::Standard)
    }
    
    /// Creates a new user with read-only permissions
    pub fn readonly(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(id, name, PermissionLevel::ReadOnly)
    }
    
    /// Adds metadata to the user
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Checks if the user has at least the given permission level
    pub fn has_permission(&self, level: PermissionLevel) -> bool {
        matches!(
            (self.permission_level, level),
            (PermissionLevel::Admin, _) |
            (PermissionLevel::Standard, PermissionLevel::Standard | PermissionLevel::ReadOnly | PermissionLevel::None) |
            (PermissionLevel::ReadOnly, PermissionLevel::ReadOnly | PermissionLevel::None) |
            (PermissionLevel::None, PermissionLevel::None)
        )
    }
}

/// Command permission requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandPermission {
    /// Required permission level to execute this command
    pub required_level: PermissionLevel,
}

impl CommandPermission {
    /// Creates a new command permission with the given required level
    pub fn new(required_level: PermissionLevel) -> Self {
        Self { required_level }
    }
    
    /// Creates an admin-only command permission
    pub fn admin_only() -> Self {
        Self::new(PermissionLevel::Admin)
    }
    
    /// Creates a standard command permission
    pub fn standard() -> Self {
        Self::new(PermissionLevel::Standard)
    }
    
    /// Creates a read-only command permission
    pub fn readonly() -> Self {
        Self::new(PermissionLevel::ReadOnly)
    }
}

/// Authentication provider interface
pub trait AuthProvider: Send + Sync + fmt::Debug {
    /// Returns the name of the authentication provider
    fn name(&self) -> &'static str;
    
    /// Authenticates a user based on credentials
    fn authenticate(&self, credentials: &AuthCredentials) -> AuthResult<User>;
    
    /// Checks if a user is authorized to execute a command
    fn authorize(&self, user: &User, command: &dyn Command) -> AuthResult<bool>;
    
    /// Clones the provider into a new Box
    fn clone_box(&self) -> Box<dyn AuthProvider>;
}

/// Authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthCredentials {
    /// No credentials, anonymous access
    None,
    
    /// Basic username/password credentials
    Basic {
        /// Username
        username: String,
        
        /// Password
        password: String,
    },
    
    /// Token-based authentication
    Token(String),
    
    /// API key authentication
    ApiKey(String),
}

/// Authentication manager
#[derive(Debug, Clone)]
pub struct AuthManager {
    /// Current authenticated user
    current_user: Arc<RwLock<Option<User>>>,
    
    /// Authentication providers
    providers: Arc<RwLock<Vec<Box<dyn AuthProvider>>>>,
    
    /// Command permission requirements
    command_permissions: Arc<RwLock<HashMap<String, CommandPermission>>>,
}

impl AuthManager {
    /// Creates a new authentication manager
    pub fn new() -> Self {
        debug!("Creating new AuthManager instance");
        Self {
            current_user: Arc::new(RwLock::new(None)),
            providers: Arc::new(RwLock::new(Vec::new())),
            command_permissions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Adds an authentication provider
    pub fn add_provider(&self, provider: Box<dyn AuthProvider>) -> AuthResult<()> {
        debug!("Adding auth provider: {}", provider.name());
        let mut providers = self.providers.write().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire write lock: {}", e))
        })?;
        providers.push(provider);
        Ok(())
    }
    
    /// Sets permission requirements for a command
    pub fn set_command_permission(&self, command_name: impl Into<String>, permission: CommandPermission) -> AuthResult<()> {
        let command_name = command_name.into();
        debug!("Setting permission for command '{}': {:?}", command_name, permission);
        
        let mut permissions = self.command_permissions.write().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire write lock: {}", e))
        })?;
        
        permissions.insert(command_name, permission);
        Ok(())
    }
    
    /// Sets the current user
    pub fn set_current_user(&self, user: User) -> AuthResult<()> {
        debug!("Setting current user: {}", user.name);
        let mut current_user = self.current_user.write().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire write lock: {}", e))
        })?;
        *current_user = Some(user);
        Ok(())
    }
    
    /// Clears the current user (logs out)
    pub fn clear_current_user(&self) -> AuthResult<()> {
        debug!("Clearing current user");
        let mut current_user = self.current_user.write().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire write lock: {}", e))
        })?;
        *current_user = None;
        Ok(())
    }
    
    /// Gets the current user
    pub fn get_current_user(&self) -> AuthResult<Option<User>> {
        let current_user = self.current_user.read().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire read lock: {}", e))
        })?;
        Ok(current_user.clone())
    }
    
    /// Authenticates a user with the given credentials
    pub fn authenticate(&self, credentials: &AuthCredentials) -> AuthResult<User> {
        debug!("Authenticating user with credentials: {:?}", credentials);
        
        let providers = self.providers.read().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire read lock: {}", e))
        })?;
        
        if providers.is_empty() {
            return Err(CommandError::ValidationError("No authentication providers registered".to_string()));
        }
        
        for provider in providers.iter() {
            match provider.authenticate(credentials) {
                Ok(user) => {
                    // Set the current user
                    self.set_current_user(user.clone())?;
                    return Ok(user);
                }
                Err(_) => continue,
            }
        }
        
        Err(CommandError::ValidationError("Authentication failed".to_string()))
    }
    
    /// Checks if the current user is authorized to execute a command
    pub fn authorize(&self, command: &dyn Command) -> AuthResult<bool> {
        let command_name = command.name();
        debug!("Authorizing command execution: {}", command_name);
        
        // Get the current user
        let current_user = match self.get_current_user()? {
            Some(user) => user,
            None => {
                debug!("No user authenticated, denying command execution");
                return Ok(false);
            }
        };
        
        // Get the command permission
        let permissions = self.command_permissions.read().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire read lock: {}", e))
        })?;
        
        let permission = match permissions.get(command_name) {
            Some(permission) => permission,
            None => {
                // If no permission is specified, default to standard
                debug!("No permission specified for command '{}', using standard", command_name);
                return Ok(current_user.has_permission(PermissionLevel::Standard));
            }
        };
        
        // Check if the user has the required permission
        let authorized = current_user.has_permission(permission.required_level);
        debug!(
            "Authorization result for '{}': {} (required: {:?}, user: {:?})",
            command_name, authorized, permission.required_level, current_user.permission_level
        );
        
        Ok(authorized)
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Basic authentication provider
#[derive(Debug, Clone)]
pub struct BasicAuthProvider {
    /// Users registered with this provider
    users: Arc<RwLock<HashMap<String, User>>>,
    
    /// User credentials (username -> password)
    credentials: Arc<RwLock<HashMap<String, String>>>,
}

impl BasicAuthProvider {
    /// Creates a new basic auth provider
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            credentials: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Adds a user to the provider
    pub fn add_user(&self, user: User, password: impl Into<String>) -> AuthResult<()> {
        let username = user.name.clone();
        let password = password.into();
        
        let mut users = self.users.write().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire write lock: {}", e))
        })?;
        
        let mut credentials = self.credentials.write().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire write lock: {}", e))
        })?;
        
        users.insert(username.clone(), user);
        credentials.insert(username, password);
        
        Ok(())
    }
}

impl Default for BasicAuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthProvider for BasicAuthProvider {
    fn name(&self) -> &'static str {
        "BasicAuthProvider"
    }
    
    fn authenticate(&self, credentials: &AuthCredentials) -> AuthResult<User> {
        match credentials {
            AuthCredentials::Basic { username, password } => {
                let credentials_map = self.credentials.read().map_err(|e| {
                    CommandError::RegistryError(format!("Failed to acquire read lock: {}", e))
                })?;
                
                let stored_password = credentials_map.get(username).ok_or_else(|| {
                    CommandError::ValidationError(format!("User not found: {}", username))
                })?;
                
                if password != stored_password {
                    return Err(CommandError::ValidationError("Invalid password".to_string()));
                }
                
                let users = self.users.read().map_err(|e| {
                    CommandError::RegistryError(format!("Failed to acquire read lock: {}", e))
                })?;
                
                let user = users.get(username).ok_or_else(|| {
                    CommandError::ValidationError(format!("User not found: {}", username))
                })?;
                
                Ok(user.clone())
            }
            _ => Err(CommandError::ValidationError("Unsupported authentication method".to_string())),
        }
    }
    
    fn authorize(&self, user: &User, _command: &dyn Command) -> AuthResult<bool> {
        // For basic auth, we just check if the user exists and has the required permission level
        // In a real implementation, this might involve more complex checks
        
        // Default to standard permission level for all commands
        let required_level = PermissionLevel::Standard;
        
        Ok(user.has_permission(required_level))
    }
    
    fn clone_box(&self) -> Box<dyn AuthProvider> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use crate::registry::CommandResult;
    
    // Test command implementation
    struct TestCommand;
    
    impl Command for TestCommand {
        fn name(&self) -> &str {
            "test-command"
        }
        
        fn description(&self) -> &str {
            "A test command"
        }
        
        fn execute(&self, _args: &[String]) -> CommandResult<String> {
            Ok("Test command executed".to_string())
        }
        
        fn parser(&self) -> clap::Command {
            clap::Command::new("test-command")
                .about("A test command")
        }
        
        fn clone_box(&self) -> Box<dyn Command> {
            Box::new(TestCommand)
        }
    }
    
    #[test]
    fn test_permission_levels() {
        assert!(PermissionLevel::Admin > PermissionLevel::Standard);
        assert!(PermissionLevel::Standard > PermissionLevel::ReadOnly);
        assert!(PermissionLevel::ReadOnly > PermissionLevel::None);
    }
    
    #[test]
    fn test_user_creation() {
        let admin = User::admin("admin-id", "Admin User");
        assert_eq!(admin.permission_level, PermissionLevel::Admin);
        
        let standard = User::standard("user-id", "Standard User");
        assert_eq!(standard.permission_level, PermissionLevel::Standard);
        
        let readonly = User::readonly("readonly-id", "ReadOnly User");
        assert_eq!(readonly.permission_level, PermissionLevel::ReadOnly);
    }
    
    #[test]
    fn test_permission_checks() {
        let admin = User::admin("admin-id", "Admin User");
        let standard = User::standard("user-id", "Standard User");
        let readonly = User::readonly("readonly-id", "ReadOnly User");
        
        // Admin can do everything
        assert!(admin.has_permission(PermissionLevel::Admin));
        assert!(admin.has_permission(PermissionLevel::Standard));
        assert!(admin.has_permission(PermissionLevel::ReadOnly));
        assert!(admin.has_permission(PermissionLevel::None));
        
        // Standard can do standard and below
        assert!(!standard.has_permission(PermissionLevel::Admin));
        assert!(standard.has_permission(PermissionLevel::Standard));
        assert!(standard.has_permission(PermissionLevel::ReadOnly));
        assert!(standard.has_permission(PermissionLevel::None));
        
        // ReadOnly can do readonly and none
        assert!(!readonly.has_permission(PermissionLevel::Admin));
        assert!(!readonly.has_permission(PermissionLevel::Standard));
        assert!(readonly.has_permission(PermissionLevel::ReadOnly));
        assert!(readonly.has_permission(PermissionLevel::None));
    }
    
    #[test]
    fn test_auth_manager() -> Result<(), Box<dyn Error>> {
        let auth_manager = AuthManager::new();
        
        // Add a basic auth provider
        let provider = BasicAuthProvider::new();
        
        // Add some users
        provider.add_user(User::admin("admin-id", "admin"), "admin-pass")?;
        provider.add_user(User::standard("user-id", "user"), "user-pass")?;
        provider.add_user(User::readonly("readonly-id", "readonly"), "readonly-pass")?;
        
        auth_manager.add_provider(Box::new(provider))?;
        
        // Set permissions for a command
        auth_manager.set_command_permission("test-command", CommandPermission::standard())?;
        
        // Test authentication
        let admin = auth_manager.authenticate(&AuthCredentials::Basic {
            username: "admin".to_string(),
            password: "admin-pass".to_string(),
        })?;
        
        assert_eq!(admin.permission_level, PermissionLevel::Admin);
        
        // Test authorization
        let command = TestCommand;
        assert!(auth_manager.authorize(&command)?);
        
        // Change to read-only user
        auth_manager.authenticate(&AuthCredentials::Basic {
            username: "readonly".to_string(),
            password: "readonly-pass".to_string(),
        })?;
        
        // ReadOnly user can't execute standard commands
        assert!(!auth_manager.authorize(&command)?);
        
        Ok(())
    }
} 