//! Authentication and authorization system for Squirrel
//!
//! This module provides functionality for authenticating and authorizing
//! command execution to ensure secure operation.

pub mod audit;
pub mod password;
pub mod provider;
pub mod roles;
pub mod types;

pub use audit::{AuditEvent, AuditEventType, AuditLogger};
pub use password::{PasswordError, PasswordManager, PasswordResult};
pub use provider::{AuthProvider, BasicAuthProvider};
pub use roles::{Permission, Role, RoleManager, create_standard_permissions, create_standard_roles};
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
    role_manager: Arc<RoleManager>,
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthManager {
    /// Creates a new authentication manager
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(Vec::new())),
            audit_logger: Arc::new(AuditLogger::new()),
            role_manager: Arc::new(RoleManager::new()),
        }
    }

    /// Creates a new authentication manager with the given provider
    pub fn with_provider(provider: Box<dyn AuthProvider>) -> Self {
        Self {
            providers: Arc::new(RwLock::new(vec![provider])),
            audit_logger: Arc::new(AuditLogger::new()),
            role_manager: Arc::new(RoleManager::new()),
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

    /// Gets a reference to the role manager
    pub fn role_manager(&self) -> &RoleManager {
        &self.role_manager
    }

    /// Initializes the RBAC system with standard roles and permissions
    pub async fn initialize_rbac(&self) -> AuthResult<()> {
        // Create standard permissions
        let permissions = create_standard_permissions();
        for permission in &permissions {
            self.role_manager.create_permission(permission.clone()).await?;
        }

        // Create standard roles
        let roles = create_standard_roles(&permissions);
        for role in &roles {
            self.role_manager.create_role(role.clone()).await?;
        }

        Ok(())
    }

    /// Authenticates a user with the given credentials
    pub async fn authenticate(&self, credentials: &AuthCredentials) -> AuthResult<User> {
        let provider = self.providers.write().await;
        let first_provider = provider.first().ok_or_else(|| {
            CommandError::RegistryError("No authentication provider available".to_string())
        })?;

        println!("DEBUG: Authenticating with provider: {}", first_provider.name());
        
        if let AuthCredentials::Basic { username, password } = credentials {
            println!("DEBUG: Username: {}, Password length: {}", username, password.len());
        }
        
        // Clone credentials for audit logging in case of failure
        let cloned_credentials = credentials.clone();
        
        // Call the synchronous authenticate method
        let result = first_provider.authenticate(credentials);
        
        println!("DEBUG: Authentication result: {:?}", result.is_ok());
        
        // Handle the result in the async context
        match result {
            Ok(user) => {
                println!("DEBUG: Authentication successful for user: {}", user.name);
                self.audit_logger.log_authentication_success(&user).await;
                Ok(user)
            }
            Err(e) => {
                println!("DEBUG: Authentication failed: {}", e);
                self.audit_logger.log_authentication_failure(&cloned_credentials, &e.to_string()).await;
                Err(e)
            }
        }
    }

    /// Authorizes a user to execute a command using role-based permissions
    pub async fn authorize(&self, user: &User, command: &dyn Command) -> AuthResult<bool> {
        println!("AuthManager.authorize: User {} (level {:?}) with command {}", 
            user.name, user.permission_level, command.name());
        
        self.audit_logger.log_authorization_attempt(user, command).await;

        // Check role-based authorization first
        let rbac_result = self.role_manager.authorize_command(&user.id, command).await?;
        println!("AuthManager.authorize: RBAC result: {}", rbac_result);
        
        if rbac_result {
            // User has role-based permission
            println!("AuthManager.authorize: User has role-based permission");
            self.audit_logger.log_authorization_success(user, command).await;
            return Ok(true);
        }

        // If there are command permissions defined but user doesn't have the roles needed,
        // don't fall back to legacy permission system
        let command_permissions = self.role_manager.get_command_permissions(command.name()).await?;
        println!("AuthManager.authorize: Command permissions count: {}", command_permissions.len());
        
        if !command_permissions.is_empty() {
            // Command has specific RBAC permissions defined, but user doesn't have them
            println!("AuthManager.authorize: Command has specific RBAC permissions, but user doesn't have them");
            self.audit_logger.log_authorization_failure(user, command, "No required role permissions").await;
            return Ok(false);
        }

        // Fall back to permission level-based authorization
        println!("AuthManager.authorize: Falling back to permission level-based authorization");
        
        let provider = self.providers.read().await;
        let first_provider = provider.first().ok_or_else(|| {
            CommandError::RegistryError("No authentication provider available".to_string())
        })?;

        println!("AuthManager.authorize: Calling provider.authorize");
        match first_provider.authorize(user, command) {
            Ok(true) => {
                println!("AuthManager.authorize: Provider authorized user");
                self.audit_logger.log_authorization_success(user, command).await;
                Ok(true)
            }
            Ok(false) => {
                println!("AuthManager.authorize: Provider denied authorization");
                self.audit_logger.log_authorization_failure(user, command, "Not authorized").await;
                Ok(false)
            }
            Err(e) => {
                println!("AuthManager.authorize: Provider returned error: {}", e);
                self.audit_logger.log_authorization_failure(user, command, &e.to_string()).await;
                Err(e)
            }
        }
    }

    /// Assigns a role to a user
    pub async fn assign_role_to_user(&self, user: &User, role_id: &str) -> AuthResult<()> {
        // Get the role to verify it exists
        let role = self.role_manager.get_role(role_id).await?;
        
        // Assign the role
        let result = self.role_manager.assign_role_to_user(&user.id, role_id).await;
        
        if result.is_ok() {
            // Log the role assignment
            self.audit_logger
                .log_user_modification(user, &format!("Assigned role: {}", role.name))
                .await;
        }
        
        result
    }

    /// Revokes a role from a user
    pub async fn revoke_role_from_user(&self, user: &User, role_id: &str) -> AuthResult<()> {
        // Get the role to verify it exists and get its name
        let role = self.role_manager.get_role(role_id).await?;
        
        // Revoke the role
        let result = self.role_manager.revoke_role_from_user(&user.id, role_id).await;
        
        if result.is_ok() {
            // Log the role revocation
            self.audit_logger
                .log_user_modification(user, &format!("Revoked role: {}", role.name))
                .await;
        }
        
        result
    }

    /// Gets all roles assigned to a user
    pub async fn get_user_roles(&self, user: &User) -> AuthResult<Vec<Role>> {
        let role_ids = self.role_manager.get_user_roles(&user.id).await?;
        let mut roles = Vec::new();
        
        for role_id in role_ids {
            if let Ok(role) = self.role_manager.get_role(&role_id).await {
                roles.push(role);
            }
        }
        
        Ok(roles)
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
    use std::collections::HashSet;
    
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

    #[tokio::test]
    async fn test_rbac() {
        let auth_manager = AuthManager::with_basic_provider();
        
        // Initialize RBAC system
        auth_manager.initialize_rbac().await.unwrap();
        
        // Create a test user
        let username = "rbac_test";
        let user = User::standard(username, username);
        auth_manager.add_user_with_password(user.clone(), "password123").await.unwrap();
        
        // Test command
        let command = TestCommand;
        
        // Get execute permission
        let permissions = auth_manager.role_manager().list_permissions().await.unwrap();
        let execute_perm = permissions
            .iter()
            .find(|p| p.resource == "command" && p.action == "execute")
            .unwrap();
        
        // Set command permissions
        let mut command_perms = HashSet::new();
        command_perms.insert(execute_perm.id.clone());
        auth_manager
            .role_manager()
            .set_command_permissions(command.name(), command_perms)
            .await
            .unwrap();
        
        // Get user role
        let roles = auth_manager.role_manager().list_roles().await.unwrap();
        let user_role = roles.iter().find(|r| r.name == "User").unwrap();
        
        // Assign role to user
        auth_manager
            .assign_role_to_user(&user, &user_role.id)
            .await
            .unwrap();
        
        // Test authorization with role
        let auth_result = auth_manager.authorize(&user, &command).await.unwrap();
        assert!(auth_result, "User should be authorized by role");
        
        // Revoke role
        auth_manager
            .revoke_role_from_user(&user, &user_role.id)
            .await
            .unwrap();
        
        // Test authorization without role
        let auth_result = auth_manager.authorize(&user, &command).await.unwrap();
        assert!(!auth_result, "User should not be authorized after role revocation");
    }
} 