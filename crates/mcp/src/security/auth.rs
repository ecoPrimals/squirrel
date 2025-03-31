use async_trait::async_trait;
use crate::error::Result;
use crate::security::types::{AuthCredentials, Token, UserId, Resource, Action};
use crate::security::rbac::RBACManager;
use crate::security::token::TokenManager;
use crate::security::audit::AuditService;
use crate::security::identity::IdentityManager;
use crate::security::crypto::CryptoProvider;
use crate::security::traits::{ResourceTrait, ActionTrait, make_permission_string};
use tracing::{info, warn};
use std::sync::Arc;
use crate::context_manager::Context;
use crate::error::{SecurityError};
use crate::security::manager::{SecurityManager, TypedSecurityManager, CombinedSecurityManager};

/// Security context for authentication and authorization
#[derive(Debug, Clone, Default)]
pub struct SecurityContext {
    /// User ID for the current operation
    pub user_id: Option<String>,
    /// Security token for the current operation
    pub token: Option<String>,
}

/// User context information
#[derive(Debug, Clone)]
pub struct UserContext {
    /// User ID
    pub user_id: UserId,
    /// User roles
    pub roles: Vec<String>,
}

/// Authentication and authorization manager
#[async_trait::async_trait]
pub trait AuthManager: Send + Sync {
    /// Authenticate a user with the provided credentials
    ///
    /// # Arguments
    /// * `credentials` - The credentials to authenticate with
    ///
    /// # Returns
    /// * `Ok(Token)` - The authentication token
    /// * `Err(SecurityError)` - If authentication fails
    async fn authenticate(&self, credentials: &AuthCredentials) -> Result<Token>;
    
    /// Validate a token
    ///
    /// # Arguments
    /// * `token_str` - The token string to validate
    ///
    /// # Returns
    /// * `Ok(Token)` - The validated token
    /// * `Err(SecurityError)` - If validation fails
    async fn validate_token(&self, token_str: &str) -> Result<Token>;
    
    /// Authorize a user for a specific resource and action
    ///
    /// # Arguments
    /// * `token` - The authentication token
    /// * `resource` - The resource to access
    /// * `action` - The action to perform
    /// * `context` - Additional context for authorization
    ///
    /// # Returns
    /// * `Ok(())` - If authorization is successful
    /// * `Err(SecurityError)` - If authorization fails
    async fn authorize(&self, token: &Token, resource: &Resource, action: &Action, _context: Option<&Context>) -> Result<()>;
    
    /// Authorize a user for a set of permissions
    ///
    /// # Arguments
    /// * `token` - The authentication token
    /// * `permissions` - The permissions to check
    ///
    /// # Returns
    /// * `Ok(())` - If authorization is successful
    /// * `Err(SecurityError)` - If authorization fails
    async fn authorize_permissions(&self, token: &Token, permissions: &[String]) -> Result<()>;
}

/// Implementation of the Auth Manager
pub struct AuthManagerImpl {
    /// Identity manager for user authentication
    pub identity_manager: Arc<dyn IdentityManager>,
    /// Cryptographic provider for secure operations
    pub crypto_provider: Arc<dyn CryptoProvider>,
    /// RBAC manager for permission-based access control
    pub rbac_manager: Arc<dyn RBACManager>,
}

impl AuthManagerImpl {
    /// Create a new instance of the auth manager
    pub fn new(
        identity_manager: Arc<dyn IdentityManager>,
        crypto_provider: Arc<dyn CryptoProvider>,
        rbac_manager: Arc<dyn RBACManager>,
    ) -> Self {
        Self {
            identity_manager,
            crypto_provider,
            rbac_manager,
        }
    }
}

#[async_trait]
impl AuthManager for AuthManagerImpl {
    async fn authenticate(&self, credentials: &AuthCredentials) -> Result<Token> {
        // Validate credentials against identity store
        // Convert token::AuthCredentials to identity::Credentials
        let identity_credentials = crate::security::identity::Credentials {
            username: credentials.username.clone(),
            password: credentials.password.clone(),
        };
        
        let user_id = self.identity_manager.verify_credentials(&identity_credentials).await?;
        
        // Create and return a token for the authenticated user
        let roles = self.rbac_manager.get_user_roles(&user_id.0.to_string()).await?;
        
        Ok(Token {
            token: format!("user.{}.roles.{}", user_id.0, roles.join(",")),
            user_id,
            roles: roles.into_iter().collect(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(8)),
            issued_at: chrono::Utc::now(),
        })
    }
    
    async fn authorize(&self, token: &Token, resource: &Resource, action: &Action, _context: Option<&Context>) -> Result<()> {
        // Create permission string from resource and action
        let permission = format!("{}:{}", action.action, resource.id);
        
        // Authorize using the permission
        self.authorize_permissions(token, &[permission]).await
    }
    
    async fn authorize_permissions(&self, token: &Token, permissions: &[String]) -> Result<()> {
        // Check each permission
        for permission in permissions {
            let has_permission = self.rbac_manager.has_permission(
                &token.user_id.0.to_string(),
                permission,
                None // No context for now
            ).await?;
            
            if !has_permission {
                return Err(crate::error::SecurityError::AuthorizationFailed(
                    format!("User {} lacks permission: {}", token.user_id, permission)
                ).into());
            }
        }
        
        Ok(())
    }
    
    async fn validate_token(&self, token_str: &str) -> Result<Token> {
        // This is just a placeholder implementation
        // In a real system, this would use a TokenManager to validate
        Err(SecurityError::Unsupported("Token validation not implemented in AuthManagerImpl".to_string()).into())
    }
}

/// Default Auth Manager that wraps the implementation
pub struct DefaultAuthManager {
    /// Inner implementation
    inner: AuthManagerImpl,
    /// Token manager for token validation
    token_manager: Arc<dyn TokenManager>,
}

impl DefaultAuthManager {
    /// Create a new instance of the default auth manager
    pub fn new(
        identity_manager: Arc<dyn IdentityManager>,
        crypto_provider: Arc<dyn CryptoProvider>,
        rbac_manager: Arc<dyn RBACManager>,
        token_manager: Arc<dyn TokenManager>,
    ) -> Self {
        Self {
            inner: AuthManagerImpl::new(identity_manager, crypto_provider, rbac_manager),
            token_manager,
        }
    }
}

#[async_trait]
impl AuthManager for DefaultAuthManager {
    async fn authenticate(&self, credentials: &AuthCredentials) -> Result<Token> {
        self.inner.authenticate(credentials).await
    }
    
    async fn authorize(&self, token: &Token, resource: &Resource, action: &Action, context: Option<&Context>) -> Result<()> {
        self.inner.authorize(token, resource, action, context).await
    }
    
    async fn authorize_permissions(&self, token: &Token, permissions: &[String]) -> Result<()> {
        self.inner.authorize_permissions(token, permissions).await
    }
    
    async fn validate_token(&self, token_str: &str) -> Result<Token> {
        self.token_manager.validate_token(token_str).await
    }
}

#[async_trait]
impl TypedSecurityManager for DefaultAuthManager {
    async fn authorize<R, A>(&self, token: &Token, resource: &R, action: &A, context: Option<&Context>) -> Result<()>
    where
        R: ResourceTrait + Send + Sync,
        A: ActionTrait + Send + Sync 
    {
        // Create permission string using the helper function
        let permission_str = make_permission_string(action, resource);
        
        // Authorize using the permission string
        self.authorize_permissions(token, &[permission_str]).await
    }
}

#[async_trait]
impl SecurityManager for DefaultAuthManager {
    async fn authenticate(&self, credentials: &AuthCredentials) -> Result<Token> {
        self.inner.authenticate(credentials).await
    }
    
    async fn authorize_concrete(&self, token: &Token, resource: &Resource, action: &Action, context: Option<&Context>) -> Result<()> {
        self.inner.authorize(token, resource, action, context).await
    }
    
    async fn validate_token(&self, token_str: &str) -> Result<Token> {
        self.token_manager.validate_token(token_str).await
    }
    
    async fn encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        self.inner.crypto_provider.encrypt(data, key, crate::security::types::EncryptionFormat::Aes256Gcm).await
    }
    
    async fn decrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        self.inner.crypto_provider.decrypt(data, key, crate::security::types::EncryptionFormat::Aes256Gcm).await
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
}

// Implement CombinedSecurityManager for DefaultAuthManager
#[async_trait]
impl CombinedSecurityManager for DefaultAuthManager {
    // No additional methods needed as it's just a marker trait
} 