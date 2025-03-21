//! Authentication provider implementations
//!
//! This module contains the authentication provider trait and its implementations.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};
use std::any::Any;

use crate::auth::{AuthCredentials, AuthResult, PasswordManager, User, PermissionLevel};
use crate::{Command, CommandError};

/// Authentication provider interface
pub trait AuthProvider: Send + Sync + fmt::Debug {
    /// Returns the name of the authentication provider
    fn name(&self) -> &'static str;
    
    /// Authenticates a user based on credentials
    fn authenticate(&self, credentials: &AuthCredentials) -> AuthResult<User>;
    
    /// Checks if a user is authorized to execute a command
    fn authorize(&self, user: &User, command: &dyn Command) -> AuthResult<bool>;
    
    /// Creates a new user
    fn create_user(&self, user: User) -> AuthResult<()>;
    
    /// Updates an existing user
    fn update_user(&self, user: User) -> AuthResult<()>;
    
    /// Deletes a user
    fn delete_user(&self, username: &str) -> AuthResult<()>;
    
    /// Gets a user by username
    fn get_user(&self, username: &str) -> AuthResult<User>;
    
    /// Lists all users
    fn list_users(&self) -> AuthResult<Vec<User>>;
    
    /// Changes a user's permission level
    fn change_permission_level(&self, username: &str, new_level: PermissionLevel) -> AuthResult<()>;
    
    /// Clones the provider into a new Box
    fn clone_box(&self) -> Box<dyn AuthProvider>;
    
    /// Converts to Any to enable downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Basic authentication provider that uses username/password credentials
#[derive(Debug, Clone)]
pub struct BasicAuthProvider {
    /// Users registered with this provider
    users: Arc<RwLock<HashMap<String, User>>>,
    
    /// User credentials (username -> password hash)
    credentials: Arc<RwLock<HashMap<String, String>>>,
    
    /// Password manager for secure password operations
    password_manager: Arc<PasswordManager>,
}

impl BasicAuthProvider {
    /// Creates a new basic authentication provider
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            credentials: Arc::new(RwLock::new(HashMap::new())),
            password_manager: Arc::new(PasswordManager::new()),
        }
    }
    
    /// Adds a user with the given password
    pub fn add_user(&self, user: User, password: impl AsRef<[u8]>) -> AuthResult<()> {
        let password_hash = self.password_manager.hash_password(password).map_err(|e| {
            CommandError::AuthenticationError(format!("Failed to hash password: {}", e))
        })?;

        let mut users = self.users.write().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire write lock: {}", e))
        })?;
        let mut credentials = self.credentials.write().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire write lock: {}", e))
        })?;

        users.insert(user.name.clone(), user.clone());
        credentials.insert(user.name.clone(), password_hash);
        Ok(())
    }
    
    /// Verifies a user's password
    pub fn verify_password(&self, username: &str, password: impl AsRef<[u8]>) -> AuthResult<bool> {
        let credentials = self.credentials.read().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire read lock: {}", e))
        })?;

        if let Some(hash) = credentials.get(username) {
            self.password_manager.verify_password(password, hash).map_err(|e| {
                CommandError::AuthenticationError(format!("Failed to verify password: {}", e))
            })
        } else {
            Ok(false)
        }
    }
}

impl Default for BasicAuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthProvider for BasicAuthProvider {
    fn name(&self) -> &'static str {
        "basic"
    }
    
    fn authenticate(&self, credentials: &AuthCredentials) -> AuthResult<User> {
        match credentials {
            AuthCredentials::Basic { username, password } => {
                let users = self.users.read().map_err(|e| {
                    CommandError::RegistryError(format!("Failed to acquire read lock: {}", e))
                })?;

                if let Some(user) = users.get(username) {
                    if self.verify_password(username, password)? {
                        Ok(user.clone())
                    } else {
                        Err(CommandError::AuthenticationError("Invalid password".to_string()))
                    }
                } else {
                    Err(CommandError::AuthenticationError("User not found".to_string()))
                }
            }
            _ => Err(CommandError::AuthenticationError(
                "Unsupported authentication method".to_string(),
            )),
        }
    }
    
    fn authorize(&self, user: &User, _command: &dyn Command) -> AuthResult<bool> {
        let users = self.users.read().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire read lock: {}", e))
        })?;
        Ok(users.contains_key(&user.name))
    }
    
    fn create_user(&self, user: User) -> AuthResult<()> {
        let mut users = self.users.write().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire write lock: {}", e))
        })?;

        if users.contains_key(&user.name) {
            return Err(CommandError::AuthenticationError("User already exists".to_string()));
        }

        users.insert(user.name.clone(), user);
        Ok(())
    }
    
    fn update_user(&self, user: User) -> AuthResult<()> {
        let mut users = self.users.write().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire write lock: {}", e))
        })?;

        if !users.contains_key(&user.name) {
            return Err(CommandError::AuthenticationError("User not found".to_string()));
        }

        users.insert(user.name.clone(), user);
        Ok(())
    }
    
    fn delete_user(&self, username: &str) -> AuthResult<()> {
        let mut users = self.users.write().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire write lock: {}", e))
        })?;

        if !users.contains_key(username) {
            return Err(CommandError::AuthenticationError("User not found".to_string()));
        }

        users.remove(username);
        Ok(())
    }
    
    fn get_user(&self, username: &str) -> AuthResult<User> {
        let users = self.users.read().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire read lock: {}", e))
        })?;

        users.get(username)
            .cloned()
            .ok_or_else(|| CommandError::AuthenticationError("User not found".to_string()))
    }
    
    fn list_users(&self) -> AuthResult<Vec<User>> {
        let users = self.users.read().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(users.values().cloned().collect())
    }
    
    fn change_permission_level(&self, username: &str, new_level: PermissionLevel) -> AuthResult<()> {
        let mut users = self.users.write().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire write lock: {}", e))
        })?;

        if let Some(user) = users.get_mut(username) {
            user.permission_level = new_level;
            Ok(())
        } else {
            Err(CommandError::AuthenticationError("User not found".to_string()))
        }
    }
    
    fn clone_box(&self) -> Box<dyn AuthProvider> {
        Box::new(Self {
            users: self.users.clone(),
            credentials: self.credentials.clone(),
            password_manager: self.password_manager.clone(),
        })
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::PermissionLevel;

    struct TestCommand;

    impl Command for TestCommand {
        fn name(&self) -> &str {
            "test"
        }

        fn description(&self) -> &str {
            "Test command"
        }

        fn execute(&self, _args: &[String]) -> crate::CommandResult<String> {
            Ok("test".to_string())
        }

        fn parser(&self) -> clap::Command {
            clap::Command::new("test")
        }

        fn clone_box(&self) -> Box<dyn Command> {
            Box::new(Self)
        }
    }

    #[test]
    fn test_basic_auth_provider() {
        let provider = BasicAuthProvider::new();
        let username = "testuser";
        let password = "password123";

        // Add a test user
        let user = User::standard(username, username);
        provider.add_user(user.clone(), password).unwrap();

        // Test successful authentication
        let credentials = AuthCredentials::Basic {
            username: username.to_string(),
            password: password.to_string(),
        };
        let authenticated_user = provider.authenticate(&credentials).unwrap();
        assert_eq!(authenticated_user.name, username);
        assert_eq!(authenticated_user.permission_level, PermissionLevel::Standard);

        // Test failed authentication
        let wrong_credentials = AuthCredentials::Basic {
            username: username.to_string(),
            password: "wrongpassword".to_string(),
        };
        assert!(provider.authenticate(&wrong_credentials).is_err());

        // Test authorization
        let command = TestCommand;
        assert!(provider.authorize(&authenticated_user, &command).is_ok());
    }
} 