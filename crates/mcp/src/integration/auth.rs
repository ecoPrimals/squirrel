//! Authentication and authorization manager for the integration layer.

use std::sync::Arc;
use crate::security::{
    Resource,
    Action,
    Token,
    manager::{SecurityManagerImpl, SecurityManager}
};
use crate::security::token::AuthCredentials;
use crate::error::Result;
use tracing::info;

/// Authentication and authorization manager for the integration layer.
///
/// Handles user authentication and authorization checks against
/// required permissions.
#[derive(Clone)]
pub struct AuthManager {
    security_manager: Arc<SecurityManagerImpl>,
}

impl AuthManager {
    /// Create a new authentication manager
    pub fn new(security_manager: Arc<SecurityManagerImpl>) -> Self {
        Self { security_manager }
    }

    /// Authenticate a user based on credentials
    pub async fn authenticate(&self, credentials: &AuthCredentials) -> Result<Token> {
        info!("Authenticating user: {}", credentials.username);
        self.security_manager.authenticate(credentials).await
    }

    /// Validate a token string and return the parsed token
    pub async fn validate_token(&self, token_str: &str) -> Result<Token> {
        self.security_manager.validate_token(token_str).await
    }

    /// Authorize a user for a specific set of permissions
    #[tracing::instrument(skip(self, token, required_permissions), fields(user_id = %token.user_id.0))]
    pub async fn authorize(&self, token: &Token, required_permissions: &[Permission]) -> Result<()> {
        for permission in required_permissions {
            let resource = Resource { 
                id: permission.resource.clone(), 
                attributes: None 
            };
            let action = Action::new(&permission.action);
            
            // Pass security context if available
            let context = None; // Optional context
            
            self.security_manager
                .authorize_concrete(token, &resource, &action, context)
                .await?;
        }
        
        info!("Authorization successful for user: {}", token.user_id.0);
        Ok(())
    }
}

/// User representation in the integration layer
#[derive(Debug, Clone)]
pub struct User {
    /// User unique identifier
    pub id: String,
    /// Authentication token, if the user is authenticated
    pub token: Option<Token>,
    /// User's roles, if available
    pub roles: Vec<String>,
}

impl User {
    /// Create a new user with the given ID
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            token: None,
            roles: Vec::new(),
        }
    }
    
    /// Create a new user from a token
    pub fn from_token(token: Token) -> Self {
        Self {
            id: token.user_id.0.to_string(),
            roles: token.roles.clone(),
            token: Some(token),
        }
    }
    
    /// Check if the user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }
}

// Add a local Permission type definition to use in this module
#[derive(Debug, Clone)]
pub struct Permission {
    pub resource: String,
    pub action: String,
} 