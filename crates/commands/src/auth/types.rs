//! Core authentication and authorization types
//!
//! This module contains the core types used by the authentication and
//! authorization system.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::CommandError;

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
    providers: Arc<RwLock<Vec<Box<dyn crate::auth::AuthProvider>>>>,
    
    /// Command permission requirements
    command_permissions: Arc<RwLock<HashMap<String, CommandPermission>>>,
}

impl AuthManager {
    /// Creates a new authentication manager
    pub fn new() -> Self {
        tracing::debug!("Creating new AuthManager instance");
        Self {
            current_user: Arc::new(RwLock::new(None)),
            providers: Arc::new(RwLock::new(Vec::new())),
            command_permissions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Adds an authentication provider
    pub fn add_provider(&self, provider: Box<dyn crate::auth::AuthProvider>) -> AuthResult<()> {
        tracing::debug!("Adding auth provider: {}", provider.name());
        let mut providers = self.providers.write().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire write lock: {}", e))
        })?;
        providers.push(provider);
        Ok(())
    }
    
    /// Sets permission requirements for a command
    pub fn set_command_permission(&self, command_name: impl Into<String>, permission: CommandPermission) -> AuthResult<()> {
        let command_name = command_name.into();
        tracing::debug!("Setting permission for command '{}': {:?}", command_name, permission);
        
        let mut permissions = self.command_permissions.write().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire write lock: {}", e))
        })?;
        
        permissions.insert(command_name, permission);
        Ok(())
    }
    
    /// Sets the current user
    pub fn set_current_user(&self, user: User) -> AuthResult<()> {
        tracing::debug!("Setting current user: {}", user.name);
        let mut current_user = self.current_user.write().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire write lock: {}", e))
        })?;
        *current_user = Some(user);
        Ok(())
    }
    
    /// Clears the current user (logs out)
    pub fn clear_current_user(&self) -> AuthResult<()> {
        tracing::debug!("Clearing current user");
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
        tracing::debug!("Authenticating user with credentials: {:?}", credentials);
        
        let providers = self.providers.read().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire read lock: {}", e))
        })?;
        
        for provider in providers.iter() {
            match provider.authenticate(credentials) {
                Ok(user) => {
                    self.set_current_user(user.clone())?;
                    return Ok(user);
                }
                Err(_) => continue,
            }
        }
        
        Err(CommandError::AuthenticationError("Authentication failed".to_string()))
    }
    
    /// Authorizes a command execution for the current user
    pub fn authorize(&self, command: &dyn crate::Command) -> AuthResult<bool> {
        let current_user = match self.get_current_user()? {
            Some(user) => user,
            None => return Ok(false),
        };
        
        let permissions = self.command_permissions.read().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire read lock: {}", e))
        })?;
        
        let required_level = permissions
            .get(command.name())
            .map(|p| p.required_level)
            .unwrap_or(PermissionLevel::Standard);
        
        Ok(current_user.has_permission(required_level))
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_levels() {
        assert!(PermissionLevel::Admin > PermissionLevel::Standard);
        assert!(PermissionLevel::Standard > PermissionLevel::ReadOnly);
        assert!(PermissionLevel::ReadOnly > PermissionLevel::None);
    }

    #[test]
    fn test_user_creation() {
        let user = User::admin("admin", "Admin User");
        assert_eq!(user.permission_level, PermissionLevel::Admin);

        let user = User::standard("user", "Standard User");
        assert_eq!(user.permission_level, PermissionLevel::Standard);

        let user = User::readonly("reader", "Read-Only User");
        assert_eq!(user.permission_level, PermissionLevel::ReadOnly);
    }

    #[test]
    fn test_permission_checks() {
        let admin = User::admin("admin", "Admin");
        let standard = User::standard("user", "User");
        let readonly = User::readonly("reader", "Reader");
        let none = User::new("none", "None", PermissionLevel::None);

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

        // ReadOnly can do readonly and below
        assert!(!readonly.has_permission(PermissionLevel::Admin));
        assert!(!readonly.has_permission(PermissionLevel::Standard));
        assert!(readonly.has_permission(PermissionLevel::ReadOnly));
        assert!(readonly.has_permission(PermissionLevel::None));

        // None can do nothing
        assert!(!none.has_permission(PermissionLevel::Admin));
        assert!(!none.has_permission(PermissionLevel::Standard));
        assert!(!none.has_permission(PermissionLevel::ReadOnly));
        assert!(none.has_permission(PermissionLevel::None));
    }
} 