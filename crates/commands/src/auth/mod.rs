//! Authentication and authorization system for Squirrel
//!
//! This module provides functionality for authenticating and authorizing
//! command execution to ensure secure operation.

pub mod audit;
pub mod password;
pub mod provider;
pub mod types;

pub use audit::{AuditEvent, AuditEventType, AuditLogger};
pub use password::{PasswordError, PasswordManager, PasswordResult};
pub use provider::{AuthProvider, BasicAuthProvider};
pub use types::{AuthCredentials, AuthResult, CommandPermission, PermissionLevel, User};

// Re-export commonly used types
pub use crate::{Command, CommandError};

use std::sync::Arc;
use tokio::sync::RwLock;

/// Authentication manager that handles user authentication and authorization
#[derive(Debug, Clone)]
pub struct AuthManager {
    providers: Arc<RwLock<Vec<Box<dyn AuthProvider>>>>,
    audit_logger: Arc<AuditLogger>,
}

impl AuthManager {
    /// Creates a new authentication manager
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(Vec::new())),
            audit_logger: Arc::new(AuditLogger::new()),
        }
    }

    /// Creates a new authentication manager with the given provider
    pub fn with_provider(provider: Box<dyn AuthProvider>) -> Self {
        Self {
            providers: Arc::new(RwLock::new(vec![provider])),
            audit_logger: Arc::new(AuditLogger::new()),
        }
    }

    /// Creates a new authentication manager with a basic provider
    pub fn with_basic_provider() -> Self {
        Self::with_provider(Box::new(BasicAuthProvider::new()))
    }

    /// Gets a reference to the audit logger
    pub fn audit_logger(&self) -> &AuditLogger {
        &self.audit_logger
    }

    /// Authenticates a user with the given credentials
    pub async fn authenticate(&self, credentials: &AuthCredentials) -> AuthResult<User> {
        let provider = self.providers.write().await;
        let first_provider = provider.first().ok_or_else(|| {
            CommandError::RegistryError("No authentication provider available".to_string())
        })?;

        match first_provider.authenticate(credentials) {
            Ok(user) => {
                self.audit_logger.log_authentication_success(&user).await;
                Ok(user)
            }
            Err(e) => {
                self.audit_logger.log_authentication_failure(credentials, &e.to_string()).await;
                Err(e)
            }
        }
    }

    /// Authorizes a user to execute a command
    pub async fn authorize(&self, user: &User, command: &dyn Command) -> AuthResult<bool> {
        let provider = self.providers.write().await;
        let first_provider = provider.first().ok_or_else(|| {
            CommandError::RegistryError("No authentication provider available".to_string())
        })?;

        match first_provider.authorize(user, command) {
            Ok(true) => {
                self.audit_logger.log_authorization_success(user, command).await;
                Ok(true)
            }
            Ok(false) => {
                self.audit_logger.log_authorization_failure(user, command, "Not authorized").await;
                Ok(false)
            }
            Err(e) => {
                self.audit_logger.log_authorization_failure(user, command, &e.to_string()).await;
                Err(e)
            }
        }
    }

    /// Creates a new user
    pub async fn create_user(&self, user: User) -> AuthResult<()> {
        let provider = self.providers.write().await;
        let first_provider = provider.first().ok_or_else(|| {
            CommandError::RegistryError("No authentication provider available".to_string())
        })?;

        match first_provider.create_user(user.clone()) {
            Ok(()) => {
                self.audit_logger.log_user_creation(&user).await;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Updates an existing user
    pub async fn update_user(&self, user: User) -> AuthResult<()> {
        let provider = self.providers.write().await;
        let first_provider = provider.first().ok_or_else(|| {
            CommandError::RegistryError("No authentication provider available".to_string())
        })?;

        match first_provider.update_user(user.clone()) {
            Ok(()) => {
                self.audit_logger.log_user_modification(&user, "Updated user information").await;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Deletes a user
    pub async fn delete_user(&self, username: &str) -> AuthResult<()> {
        let provider = self.providers.write().await;
        let first_provider = provider.first().ok_or_else(|| {
            CommandError::RegistryError("No authentication provider available".to_string())
        })?;

        match first_provider.delete_user(username) {
            Ok(()) => {
                self.audit_logger.log_user_deletion(username).await;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Changes a user's permission level
    pub async fn change_permission_level(&self, user: &User, new_level: PermissionLevel) -> AuthResult<()> {
        let provider = self.providers.write().await;
        let first_provider = provider.first().ok_or_else(|| {
            CommandError::RegistryError("No authentication provider available".to_string())
        })?;

        let old_level = user.permission_level;
        match first_provider.change_permission_level(&user.name, new_level) {
            Ok(()) => {
                self.audit_logger.log_permission_change(user, old_level, new_level).await;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Gets a user by username
    pub async fn get_user(&self, username: &str) -> AuthResult<User> {
        let provider = self.providers.read().await;
        let first_provider = provider.first().ok_or_else(|| {
            CommandError::RegistryError("No authentication provider available".to_string())
        })?;
        first_provider.get_user(username)
    }

    /// Lists all users
    pub async fn list_users(&self) -> AuthResult<Vec<User>> {
        let provider = self.providers.read().await;
        let first_provider = provider.first().ok_or_else(|| {
            CommandError::RegistryError("No authentication provider available".to_string())
        })?;
        first_provider.list_users()
    }

    /// Gets the basic auth provider
    pub async fn get_basic_provider(&self) -> AuthResult<BasicAuthProvider> {
        let provider = self.providers.read().await;
        let first_provider = provider.first().ok_or_else(|| {
            CommandError::RegistryError("No authentication provider available".to_string())
        })?;
        
        // Try to downcast and clone
        if let Some(basic_provider) = first_provider.as_any().downcast_ref::<BasicAuthProvider>() {
            Ok(basic_provider.clone())
        } else {
            Err(CommandError::RegistryError("Provider is not a BasicAuthProvider".to_string()))
        }
    }

    /// Adds a user with a password (convenience method for BasicAuthProvider)
    pub async fn add_user_with_password(&self, user: User, password: impl AsRef<[u8]>) -> AuthResult<()> {
        let provider = self.get_basic_provider().await?;
        let result = provider.add_user(user.clone(), password);
        if result.is_ok() {
            self.audit_logger.log_user_creation(&user).await;
        }
        result
    }

    /// Adds a provider to the authentication manager
    pub async fn add_provider(&self, provider: Box<dyn AuthProvider>) -> AuthResult<()> {
        let mut providers = self.providers.write().await;
        providers.push(provider);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::User;
    
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
    
    #[tokio::test]
    async fn test_auth_manager() {
        let auth_manager = AuthManager::with_basic_provider();
        let username = "testuser";
        let user = User::standard(username, username);
        auth_manager.add_user_with_password(user.clone(), "password123").await.unwrap();

        // Test authentication
        let credentials = AuthCredentials::Basic {
            username: username.to_string(),
            password: "password123".to_string(),
        };

        let authenticated_user = auth_manager.authenticate(&credentials).await.unwrap();
        assert_eq!(authenticated_user.name, username);

        // Test authorization
        let command = TestCommand;
        auth_manager
            .authorize(&authenticated_user, &command)
            .await
            .unwrap();

        // Test permission level change
        auth_manager
            .change_permission_level(&authenticated_user, PermissionLevel::Admin)
            .await
            .unwrap();

        let updated_user = auth_manager.get_user(username).await.unwrap();
        assert_eq!(updated_user.permission_level, PermissionLevel::Admin);

        // Test user deletion
        auth_manager.delete_user(username).await.unwrap();

        // Verify audit logs
        let audit_logs = auth_manager.audit_logger().get_events().await;
        assert!(audit_logs.len() >= 5); // Creation, auth success, authorization success, permission change, deletion
    }
} 