use async_trait::async_trait;
use crate::error::Result;
use crate::security::types::{Credentials, Token, UserId, Resource, Action, SecurityLevel, EncryptionFormat};
use crate::security::auth::SecurityContext;
use crate::security::crypto::CryptoProvider;
use crate::security::token::TokenManager;
use crate::security::rbac::RBACManager;
use crate::security::identity::IdentityManager;
use crate::security::audit::AuditService;
use crate::security::traits::{ResourceTrait, ActionTrait, make_permission_string};
use std::sync::Arc;
use tracing::{info, warn, error, instrument};
use crate::error::MCPError;
use crate::context_manager::Context;

/// Security manager trait for unified security operations
#[async_trait::async_trait]
pub trait SecurityManager: Send + Sync {
    /// Authenticate a user with the provided credentials
    async fn authenticate(&self, credentials: &Credentials) -> Result<Token>;
    
    /// Authorize a user for a specific resource and action
    async fn authorize<R, A>(&self, token: &Token, resource: &R, action: &A, context: Option<&Context>) -> Result<()>
    where
        R: ResourceTrait + Send + Sync,
        A: ActionTrait + Send + Sync;
    
    /// Validate a token string
    async fn validate_token(&self, token_str: &str) -> Result<Token>;
    
    /// Encrypt data with the specified key
    async fn encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>>;
    
    /// Decrypt data with the specified key
    async fn decrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>>;
    
    /// Get the security manager version
    fn version(&self) -> &str;
}

/// Security Manager implementation
pub struct SecurityManagerImpl {
    crypto_provider: Arc<dyn CryptoProvider>,
    token_manager: Arc<dyn TokenManager>,
    identity_manager: Arc<dyn IdentityManager>,
    rbac_manager: Arc<dyn RBACManager>,
    audit_service: Arc<dyn AuditService>,
    version: String,
}

impl SecurityManagerImpl {
    /// Create a new security manager with the specified components
    pub fn new(
        crypto_provider: Arc<dyn CryptoProvider>,
        token_manager: Arc<dyn TokenManager>,
        identity_manager: Arc<dyn IdentityManager>,
        rbac_manager: Arc<dyn RBACManager>,
        audit_service: Arc<dyn AuditService>,
    ) -> Self {
        Self {
            crypto_provider,
            token_manager,
            identity_manager,
            rbac_manager,
            audit_service,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    
    /// Get the encryption key for a security context
    async fn get_encryption_key_for_context(&self, _context: &SecurityContext) -> Result<Vec<u8>> {
        // In a real implementation, retrieve a key based on the context
        // This is a placeholder that returns a fixed key
        let key = vec![0u8; 32]; // 256-bit key
        Ok(key)
    }
    
    /// Get the encryption format for a security context
    fn get_encryption_format_for_context(&self, _context: &SecurityContext) -> EncryptionFormat {
        // In a real implementation, determine the format based on the context
        // This is a placeholder that returns a fixed format
        EncryptionFormat::Aes256Gcm
    }
}

#[async_trait::async_trait]
impl SecurityManager for SecurityManagerImpl {
    #[instrument(skip(self, credentials), fields(username = %credentials.username))]
    async fn authenticate(&self, credentials: &Credentials) -> Result<Token> {
        // Convert token::Credentials to identity::Credentials
        let identity_credentials = crate::security::identity::Credentials {
            username: credentials.username.clone(),
            password: credentials.password.clone(),
        };
        
        let user_id = self.identity_manager.verify_credentials(&identity_credentials).await?;
        let roles = self.rbac_manager.get_user_roles(&user_id.0.to_string()).await?;
        let token = self.token_manager.generate_token(&user_id, &roles).await?;
        self.audit_service.log_authentication_success(&user_id).await;
        Ok(token)
    }

    #[instrument(skip(self, token_str))]
    async fn validate_token(&self, token_str: &str) -> Result<Token> {
        self.token_manager.validate_token(token_str).await
    }

    #[instrument(skip(self, token, resource, action, context))]
    async fn authorize<R, A>(&self, token: &Token, resource: &R, action: &A, context: Option<&Context>) -> Result<()>
    where
        R: ResourceTrait + Send + Sync,
        A: ActionTrait + Send + Sync
    {
        let permission_str = make_permission_string(action, resource);
        
        let has_permission = self.rbac_manager.has_permission(&token.user_id.0.to_string(), &permission_str, context).await?;
        
        if has_permission {
            self.audit_service.log_authorization_success(&token.user_id, &format!("{}", resource), &format!("{}", action)).await;
            Ok(())
        } else {
            self.audit_service.log_authorization_failure(&token.user_id, &format!("{}", resource), &format!("{}", action), "Permission denied").await;
            Err(crate::error::SecurityError::AuthorizationFailed(
                format!("User {} lacks permission {} for resource {}", token.user_id, permission_str, resource)
            ).into())
        }
    }

    #[instrument(skip(self, data, key))]
    async fn encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        self.crypto_provider.encrypt(data, key, self.get_encryption_format_for_context(&SecurityContext::default())).await
    }

    #[instrument(skip(self, data, key))]
    async fn decrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        self.crypto_provider.decrypt(data, key, self.get_encryption_format_for_context(&SecurityContext::default())).await
    }

    fn version(&self) -> &str {
        &self.version
    }
} 