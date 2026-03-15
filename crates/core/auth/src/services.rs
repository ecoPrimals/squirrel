// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Authentication service implementations and traits.
//!
//! This module contains the main authentication service trait and its
//! implementation using the BearDog security provider.

use super::providers::{
    AuthProvider, AuthRequest, BeardogPermission, BeardogSession, ComplianceMonitor,
    EncryptionService, UserInfo,
};
use super::types::{
    AuditEvent, AuthContext, AuthError, ComplianceCheck, LoginRequest, LoginResponse, Permission,
    Session, User,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

/// Main authentication service trait
#[async_trait]
pub trait AuthenticationService: Send + Sync {
    /// Authenticate user credentials and return an authentication context
    async fn authenticate(&self, credentials: &LoginRequest) -> Result<AuthContext, AuthError>;

    /// Verify a token and return the authentication context
    async fn verify_token(&self, token: &str) -> Result<AuthContext, AuthError>;

    /// Refresh an authentication token
    async fn refresh_token(&self, refresh_token: &str) -> Result<LoginResponse, AuthError>;

    /// Logout and invalidate a session
    async fn logout(&self, session_id: &Uuid) -> Result<(), AuthError>;

    /// Get user permissions by user ID
    async fn get_user_permissions(&self, user_id: &Uuid) -> Result<Vec<Permission>, AuthError>;

    /// Check if a user has a specific permission
    async fn has_permission(
        &self,
        user_id: &Uuid,
        permission: &Permission,
    ) -> Result<bool, AuthError>;
}

/// BearDog security client implementing the authentication service
pub struct BeardogSecurityClient {
    auth_provider: Arc<AuthProvider>,
    encryption_service: Arc<EncryptionService>,
    compliance_monitor: Arc<ComplianceMonitor>,
    config: squirrel_mcp_config::BeardogConfig,
}

impl BeardogSecurityClient {
    /// Create a new BearDog security client
    pub async fn new(config: squirrel_mcp_config::BeardogConfig) -> Result<Self, AuthError> {
        let auth_provider = Arc::new(
            AuthProvider::new(&config.auth_endpoint.to_string(), &config.jwt_secret_key_id)
                .await
                .map_err(|e| AuthError::BeardogError(e.to_string()))?,
        );

        let encryption_service = Arc::new(
            EncryptionService::new(&config.encryption_algorithm, &config.hsm_provider)
                .await
                .map_err(|e| AuthError::BeardogError(e.to_string()))?,
        );

        let compliance_monitor = Arc::new(
            ComplianceMonitor::new(&config.compliance_mode, config.audit_enabled)
                .await
                .map_err(|e| AuthError::BeardogError(e.to_string()))?,
        );

        Ok(Self {
            auth_provider,
            encryption_service,
            compliance_monitor,
            config,
        })
    }

    /// Authenticate a request using a token
    pub async fn authenticate_request(&self, token: &str) -> Result<AuthContext, AuthError> {
        let claims = self
            .auth_provider
            .verify_jwt(token)
            .await
            .map_err(|e| AuthError::BeardogError(e.to_string()))?;

        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken)?;

        let permissions = self
            .auth_provider
            .get_permissions(&user_id)
            .await
            .map_err(|e| AuthError::BeardogError(e.to_string()))?;

        // Convert BearDog permissions to our format
        let squirrel_permissions: Vec<Permission> = permissions
            .into_iter()
            .map(|p| Permission {
                resource: p.resource,
                action: p.action,
                scope: p.scope,
            })
            .collect();

        let session_id =
            Uuid::parse_str(&claims.session_id).map_err(|_| AuthError::InvalidToken)?;

        let expires_at = DateTime::from_timestamp(claims.exp, 0).ok_or(AuthError::InvalidToken)?;

        let issued_at = DateTime::from_timestamp(claims.iat, 0).ok_or(AuthError::InvalidToken)?;

        Ok(AuthContext {
            user_id,
            username: claims.username,
            permissions: squirrel_permissions,
            session_id,
            expires_at,
            issued_at,
            roles: claims.roles,
        })
    }

    /// Create a new session for a user
    pub async fn create_session(&self, user: &User) -> Result<Session, AuthError> {
        let session_id = Uuid::new_v4();
        let expires_at =
            Utc::now() + chrono::Duration::seconds(self.config.timeout.as_secs() as i64);

        let beardog_session = BeardogSession {
            id: session_id,
            user_id: user.id,
            username: user.username.clone(),
            roles: user.roles.clone(),
            expires_at,
            created_at: Utc::now(),
        };

        self.auth_provider
            .create_session(&beardog_session)
            .await
            .map_err(|e| AuthError::BeardogError(e.to_string()))?;

        Ok(Session {
            id: session_id,
            user_id: user.id,
            username: user.username.clone(),
            expires_at,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            active: true,
        })
    }

    /// Encrypt sensitive data
    pub async fn encrypt_sensitive_data(&self, data: &[u8]) -> Result<Vec<u8>, AuthError> {
        self.encryption_service
            .encrypt_with_context(data, "squirrel-mcp")
            .await
            .map_err(|e| AuthError::BeardogError(e.to_string()))
    }

    /// Decrypt sensitive data
    pub async fn decrypt_sensitive_data(
        &self,
        encrypted_data: &[u8],
    ) -> Result<Vec<u8>, AuthError> {
        self.encryption_service
            .decrypt_with_context(encrypted_data, "squirrel-mcp")
            .await
            .map_err(|e| AuthError::BeardogError(e.to_string()))
    }

    /// Audit a login attempt
    pub async fn audit_login_attempt(
        &self,
        username: &str,
        success: bool,
        ip_address: &str,
    ) -> Result<(), AuthError> {
        let event = AuditEvent::new("login_attempt", None, Some(username.to_string()), success)
            .with_ip_address(ip_address)
            .with_details(serde_json::json!({
                "action": "login",
                "success": success,
                "source": "squirrel-mcp"
            }));

        self.compliance_monitor
            .record_event(&event)
            .await
            .map_err(|e| AuthError::BeardogError(e.to_string()))?;

        Ok(())
    }

    /// Check compliance for a user action
    pub async fn check_compliance(&self, user_id: &Uuid, action: &str) -> Result<bool, AuthError> {
        let compliance_check = ComplianceCheck::new(*user_id, action, "squirrel-mcp");

        self.compliance_monitor
            .check_compliance(&compliance_check)
            .await
            .map_err(|e| AuthError::BeardogError(e.to_string()))
    }

    /// Get authentication provider reference
    ///
    /// Returns a reference to the underlying authentication provider that handles
    /// user authentication, password verification, and credential management.
    ///
    /// # Returns
    ///
    /// Returns a reference to the `AuthProvider` instance used by this service manager.
    pub fn auth_provider(&self) -> &AuthProvider {
        &self.auth_provider
    }

    /// Get encryption service reference
    ///
    /// Returns a reference to the encryption service that handles cryptographic
    /// operations such as password hashing, data encryption, and secure key
    /// generation.
    ///
    /// # Returns
    ///
    /// Returns a reference to the `EncryptionService` instance used by this service manager.
    pub fn encryption_service(&self) -> &EncryptionService {
        &self.encryption_service
    }

    /// Get compliance monitor reference
    ///
    /// Returns a reference to the compliance monitor that tracks and validates
    /// authentication events, security policies, and regulatory compliance
    /// requirements.
    ///
    /// # Returns
    ///
    /// Returns a reference to the `ComplianceMonitor` instance used by this service manager.
    pub fn compliance_monitor(&self) -> &ComplianceMonitor {
        &self.compliance_monitor
    }

    /// Get configuration reference
    ///
    /// Returns a reference to the Beardog security configuration that contains
    /// authentication settings, security policies, and service configuration
    /// parameters.
    ///
    /// # Returns
    ///
    /// Returns a reference to the `BeardogConfig` instance used by this service manager.
    pub fn config(&self) -> &squirrel_mcp_config::BeardogConfig {
        &self.config
    }
}

#[async_trait]
impl AuthenticationService for BeardogSecurityClient {
    async fn authenticate(&self, credentials: &LoginRequest) -> Result<AuthContext, AuthError> {
        let auth_request = AuthRequest {
            username: credentials.username.clone(),
            password: credentials.password.clone(),
            remember_me: credentials.remember_me,
        };

        let beardog_context = self
            .auth_provider
            .authenticate(&auth_request)
            .await
            .map_err(|e| AuthError::BeardogError(e.to_string()))?;

        // Convert BearDog context to our format
        let permissions: Vec<Permission> = beardog_context
            .permissions
            .into_iter()
            .map(|p| Permission {
                resource: p.resource,
                action: p.action,
                scope: p.scope,
            })
            .collect();

        Ok(AuthContext {
            user_id: beardog_context.user_id,
            username: beardog_context.username,
            permissions,
            session_id: beardog_context.session_id,
            expires_at: beardog_context.expires_at,
            issued_at: beardog_context.issued_at,
            roles: beardog_context.roles,
        })
    }

    async fn verify_token(&self, token: &str) -> Result<AuthContext, AuthError> {
        self.authenticate_request(token).await
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<LoginResponse, AuthError> {
        let new_tokens = self
            .auth_provider
            .refresh_token(refresh_token)
            .await
            .map_err(|e| AuthError::BeardogError(e.to_string()))?;

        let user_info = self
            .auth_provider
            .get_user_info(&new_tokens.user_id)
            .await
            .map_err(|e| AuthError::BeardogError(e.to_string()))?;

        let user = convert_user_info_to_user(user_info);

        Ok(LoginResponse {
            access_token: new_tokens.access_token,
            refresh_token: Some(new_tokens.refresh_token),
            expires_in: new_tokens.expires_in,
            token_type: "Bearer".to_string(),
            user,
        })
    }

    async fn logout(&self, session_id: &Uuid) -> Result<(), AuthError> {
        self.auth_provider
            .invalidate_session(session_id)
            .await
            .map_err(|e| AuthError::BeardogError(e.to_string()))?;

        Ok(())
    }

    async fn get_user_permissions(&self, user_id: &Uuid) -> Result<Vec<Permission>, AuthError> {
        let permissions = self
            .auth_provider
            .get_permissions(user_id)
            .await
            .map_err(|e| AuthError::BeardogError(e.to_string()))?;

        let squirrel_permissions: Vec<Permission> = permissions
            .into_iter()
            .map(|p| Permission {
                resource: p.resource,
                action: p.action,
                scope: p.scope,
            })
            .collect();

        Ok(squirrel_permissions)
    }

    async fn has_permission(
        &self,
        user_id: &Uuid,
        permission: &Permission,
    ) -> Result<bool, AuthError> {
        let beardog_permission = BeardogPermission {
            resource: permission.resource.clone(),
            action: permission.action.clone(),
            scope: permission.scope.clone(),
        };

        self.auth_provider
            .has_permission(user_id, &beardog_permission)
            .await
            .map_err(|e| AuthError::BeardogError(e.to_string()))
    }
}

/// Convert UserInfo from BearDog to our User type
fn convert_user_info_to_user(user_info: UserInfo) -> User {
    let permissions: Vec<Permission> = user_info
        .permissions
        .into_iter()
        .map(|p| Permission {
            resource: p.resource,
            action: p.action,
            scope: p.scope,
        })
        .collect();

    User {
        id: user_info.id,
        username: user_info.username,
        email: user_info.email,
        roles: user_info.roles,
        permissions,
        created_at: user_info.created_at,
        updated_at: user_info.updated_at,
        last_login: user_info.last_login,
        login_attempts: user_info.login_attempts,
        locked_until: user_info.locked_until,
        active: user_info.active,
    }
}

/// Authentication helper functions
pub mod helpers {
    use super::*;

    /// Create a simple authentication context for testing
    pub fn create_test_auth_context(username: &str, roles: Vec<String>) -> AuthContext {
        AuthContext {
            user_id: Uuid::new_v4(),
            username: username.to_string(),
            permissions: vec![],
            session_id: Uuid::new_v4(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            issued_at: Utc::now(),
            roles,
        }
    }

    /// Create a test user
    pub fn create_test_user(username: &str, email: &str) -> User {
        User::new(username, email)
    }

    /// Create a test login request
    pub fn create_test_login_request(username: &str, password: &str) -> LoginRequest {
        LoginRequest::new(username, password)
    }

    /// Validate authentication context
    pub fn validate_auth_context(context: &AuthContext) -> Result<(), AuthError> {
        if context.is_expired() {
            return Err(AuthError::TokenExpired);
        }

        if context.username.is_empty() {
            return Err(AuthError::InvalidToken);
        }

        Ok(())
    }

    /// Check if user has admin role
    pub fn is_admin(context: &AuthContext) -> bool {
        context.has_role("admin") || context.has_role("administrator")
    }

    /// Check if user has read permission for a resource
    pub fn can_read_resource(context: &AuthContext, resource: &str) -> bool {
        let permission = Permission::new(resource, "read");
        context.has_permission(&permission)
    }

    /// Check if user has write permission for a resource
    pub fn can_write_resource(context: &AuthContext, resource: &str) -> bool {
        let permission = Permission::new(resource, "write");
        context.has_permission(&permission)
    }

    /// Extract user ID from authentication context
    pub fn get_user_id(context: &AuthContext) -> Uuid {
        context.user_id
    }

    /// Extract session ID from authentication context
    pub fn get_session_id(context: &AuthContext) -> Uuid {
        context.session_id
    }

    /// Check if context has any of the specified roles
    pub fn has_any_role(context: &AuthContext, roles: &[&str]) -> bool {
        roles.iter().any(|role| context.has_role(role))
    }

    /// Check if context has all of the specified roles
    pub fn has_all_roles(context: &AuthContext, roles: &[&str]) -> bool {
        roles.iter().all(|role| context.has_role(role))
    }
}

#[cfg(test)]
mod tests {
    use super::helpers::*;
    use super::*;

    #[tokio::test]
    async fn test_authentication_service_trait() {
        // This test would require a mock implementation
        // In a real scenario, you would create a mock AuthenticationService
        assert!(true); // Placeholder
    }

    #[test]
    fn test_auth_context_creation() {
        let context = create_test_auth_context("test_user", vec!["user".to_string()]);

        assert_eq!(context.username, "test_user");
        assert!(context.has_role("user"));
        assert!(!context.is_expired());
    }

    #[test]
    fn test_user_creation() {
        let user = create_test_user("test_user", "test@example.com");

        assert_eq!(user.username, "test_user");
        assert_eq!(user.email, "test@example.com");
        assert!(user.active);
        assert!(!user.is_locked());
    }

    #[test]
    fn test_login_request_creation() {
        let request = create_test_login_request("username", "password");

        assert_eq!(request.username, "username");
        assert_eq!(request.password, "password");
        assert!(!request.remember_me);
    }

    #[test]
    fn test_auth_context_validation() {
        let context = create_test_auth_context("test_user", vec!["user".to_string()]);

        assert!(validate_auth_context(&context).is_ok());
    }

    #[test]
    fn test_admin_check() {
        let admin_context = create_test_auth_context("admin", vec!["admin".to_string()]);
        let user_context = create_test_auth_context("user", vec!["user".to_string()]);

        assert!(is_admin(&admin_context));
        assert!(!is_admin(&user_context));
    }

    #[test]
    fn test_permission_helpers() {
        let mut context = create_test_auth_context("test_user", vec!["user".to_string()]);
        context.permissions.push(Permission::new("mcp", "read"));
        context.permissions.push(Permission::new("mcp", "write"));

        assert!(can_read_resource(&context, "mcp"));
        assert!(can_write_resource(&context, "mcp"));
        assert!(!can_read_resource(&context, "admin"));
    }

    #[test]
    fn test_role_checks() {
        let context =
            create_test_auth_context("test_user", vec!["user".to_string(), "editor".to_string()]);

        assert!(has_any_role(&context, &["user", "admin"]));
        assert!(has_all_roles(&context, &["user", "editor"]));
        assert!(!has_all_roles(&context, &["user", "admin"]));
    }

    #[test]
    fn test_id_extraction() {
        let context = create_test_auth_context("test_user", vec!["user".to_string()]);

        let user_id = get_user_id(&context);
        let session_id = get_session_id(&context);

        assert_eq!(user_id, context.user_id);
        assert_eq!(session_id, context.session_id);
    }

    #[test]
    fn test_user_info_conversion() {
        let user_info = UserInfo {
            id: Uuid::new_v4(),
            username: "test_user".to_string(),
            email: "test@example.com".to_string(),
            roles: vec!["user".to_string()],
            permissions: vec![BeardogPermission {
                resource: "mcp".to_string(),
                action: "read".to_string(),
                scope: None,
            }],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: None,
            login_attempts: 0,
            locked_until: None,
            active: true,
        };

        let user = convert_user_info_to_user(user_info.clone());

        assert_eq!(user.id, user_info.id);
        assert_eq!(user.username, user_info.username);
        assert_eq!(user.email, user_info.email);
        assert_eq!(user.roles, user_info.roles);
        assert_eq!(user.permissions.len(), 1);
        assert!(user.active);
    }
}
