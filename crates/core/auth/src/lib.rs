//! Authentication and Security Framework
//!
//! This crate provides a comprehensive authentication and security framework
//! for the Squirrel MCP system, with BearDog integration for enterprise security.
//!
//! ## Architecture Overview
//!
//! The authentication system is organized into several key components:
//!
//! - **Types**: Core authentication types, errors, and data structures
//! - **Providers**: BearDog security provider integration with auth, encryption, and compliance
//! - **Services**: Authentication service trait and implementations
//! - **Modules**: JWT, sessions, middleware, and bearer token support
//!
//! ## Usage Example
//!
//! ```rust
//! use squirrel_auth::{AuthenticationService, BeardogSecurityClient, LoginRequest};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create authentication service
//! let config = squirrel_mcp_config::BeardogConfig::default();
//! let auth_service = BeardogSecurityClient::new(config).await?;
//!
//! // Authenticate user
//! let login_request = LoginRequest::new("username", "password");
//! let auth_context = auth_service.authenticate(&login_request).await?;
//!
//! // Check permissions
//! let has_permission = auth_service.has_permission(
//!     &auth_context.user_id,
//!     &Permission::new("mcp", "read")
//! ).await?;
//! # Ok(())
//! # }
//! ```

// Existing modules
pub mod bearer_token;
pub mod jwt;
pub mod middleware;
pub mod session;

// New organized modules
pub mod providers;
pub mod services;
pub mod types;

// Re-export core types from types module
pub use types::{
    AuditEvent, AuthContext, AuthError, ComplianceCheck, LoginRequest, LoginResponse, Permission,
    Session, TokenResponse, User,
};

// Re-export providers from providers module
pub use providers::{
    AuthProvider, AuthRequest, BeardogAuthContext, BeardogPermission, BeardogSession,
    ComplianceMonitor, EncryptionService, UserInfo,
};

// Re-export services from services module
pub use services::{AuthenticationService, BeardogSecurityClient};

// Re-export from existing modules for backward compatibility
pub use bearer_token::BearerTokenValidator;
pub use jwt::{JwtClaims, JwtTokenManager};
pub use middleware::SecurityMiddleware;
pub use session::{Session as SessionModule, SessionManager};

// Backward compatibility aliases
pub use services::BeardogSecurityClient as BeardogClient;
pub use types::AuthContext as Context;
pub use types::AuthError as Error;

/// Module for authentication helpers and utilities
pub mod helpers {
    pub use crate::services::helpers::*;
}

/// Configuration utilities for authentication setup
pub mod config {

    /// Default configuration for BearDog integration
    pub fn default_beardog_config() -> squirrel_mcp_config::BeardogConfig {
        squirrel_mcp_config::BeardogConfig::default()
    }

    /// Create test configuration for development
    pub fn test_config() -> squirrel_mcp_config::BeardogConfig {
        squirrel_mcp_config::BeardogConfig {
            auth_endpoint: "http://localhost:8080".to_string(),
            encryption_algorithm: "AES-256-GCM".to_string(),
            hsm_provider: "SoftHSM".to_string(),
            compliance_mode: "development".to_string(),
            audit_enabled: true,
            jwt_secret_key_id: "test-key".to_string(),
            timeout: std::time::Duration::from_secs(3600),
        }
    }
}

/// Prelude module for common imports
pub mod prelude {
    pub use crate::helpers::*;
    pub use crate::{
        AuditEvent, AuthContext, AuthError, AuthenticationService, BeardogSecurityClient,
        LoginRequest, LoginResponse, Permission, Session, User,
    };
}

#[cfg(test)]
mod integration_tests {
    use super::helpers::*;
    use super::*;

    #[test]
    fn test_module_organization() {
        // Test that we can create core types
        let user = create_test_user("test_user", "test@example.com");
        let context = create_test_auth_context("test_user", vec!["user".to_string()]);
        let login_request = create_test_login_request("username", "password");

        assert_eq!(user.username, "test_user");
        assert_eq!(context.username, "test_user");
        assert_eq!(login_request.username, "username");
    }

    #[test]
    fn test_permission_system() {
        let permission = Permission::new("mcp", "read");
        let scoped_permission = Permission::with_scope("mcp", "write", "admin");

        assert_eq!(permission.resource, "mcp");
        assert_eq!(permission.action, "read");
        assert!(permission.scope.is_none());

        assert_eq!(scoped_permission.resource, "mcp");
        assert_eq!(scoped_permission.action, "write");
        assert_eq!(scoped_permission.scope, Some("admin".to_string()));
    }

    #[test]
    fn test_user_management() {
        let mut user = User::new("test_user", "test@example.com");

        user.add_role("user");
        user.add_role("editor");
        assert!(user.has_role("user"));
        assert!(user.has_role("editor"));
        assert!(!user.has_role("admin"));

        let permission = Permission::new("mcp", "read");
        user.add_permission(permission.clone());
        assert!(user.has_permission(&permission));

        user.mark_login();
        assert!(user.last_login.is_some());
        assert_eq!(user.login_attempts, 0);
    }

    #[test]
    fn test_session_management() {
        let user_id = uuid::Uuid::new_v4();
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
        let mut session = Session::new(user_id, "test_user", expires_at);

        assert!(!session.is_expired());
        assert!(session.is_active());

        session.mark_accessed();
        assert!(session.last_accessed > session.created_at);

        session.invalidate();
        assert!(!session.is_active());
    }

    #[test]
    fn test_auth_context_features() {
        let mut context =
            create_test_auth_context("test_user", vec!["user".to_string(), "editor".to_string()]);

        assert!(context.has_role("user"));
        assert!(context.has_role("editor"));
        assert!(!context.has_role("admin"));
        assert!(!context.is_expired());

        let permission = Permission::new("mcp", "read");
        context.permissions.push(permission.clone());
        assert!(context.has_permission(&permission));
    }

    #[test]
    fn test_audit_events() {
        let user_id = uuid::Uuid::new_v4();
        let event = AuditEvent::new("login", Some(user_id), Some("test_user".to_string()), true)
            .with_ip_address("192.168.1.1")
            .with_details(serde_json::json!({
                "source": "test",
                "action": "login"
            }));

        assert_eq!(event.event_type, "login");
        assert_eq!(event.user_id, Some(user_id));
        assert_eq!(event.username, Some("test_user".to_string()));
        assert!(event.success);
        assert_eq!(event.ip_address, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_compliance_checks() {
        let user_id = uuid::Uuid::new_v4();
        let check = ComplianceCheck::new(user_id, "read", "mcp");

        assert_eq!(check.user_id, user_id);
        assert_eq!(check.action, "read");
        assert_eq!(check.resource, "mcp");
    }

    #[test]
    fn test_error_types() {
        let errors = vec![
            AuthError::InvalidCredentials,
            AuthError::TokenExpired,
            AuthError::InvalidToken,
            AuthError::InsufficientPermissions,
            AuthError::UserNotFound,
            AuthError::SessionExpired,
            AuthError::AuthenticationFailed("test".to_string()),
            AuthError::BeardogError("test".to_string()),
        ];

        for error in errors {
            assert!(!error.to_string().is_empty());
        }
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that old aliases still work
        let _error: Error = AuthError::InvalidCredentials;
        let context = create_test_auth_context("test", vec![]);
        let _context_alias: Context = context;

        // Just verify they compile and work
        assert!(true);
    }

    #[test]
    fn test_config_utilities() {
        let default_config = config::default_beardog_config();
        let test_config = config::test_config();

        assert!(!default_config.auth_endpoint.is_empty());
        assert_eq!(test_config.compliance_mode, "development");
        assert!(test_config.audit_enabled);
    }

    #[test]
    fn test_helper_functions() {
        let context = create_test_auth_context("admin", vec!["admin".to_string()]);

        assert!(is_admin(&context));
        assert_eq!(get_user_id(&context), context.user_id);
        assert_eq!(get_session_id(&context), context.session_id);
        assert!(validate_auth_context(&context).is_ok());

        let context_with_perms = {
            let mut ctx = create_test_auth_context("user", vec!["user".to_string()]);
            ctx.permissions.push(Permission::new("mcp", "read"));
            ctx.permissions.push(Permission::new("mcp", "write"));
            ctx
        };

        assert!(can_read_resource(&context_with_perms, "mcp"));
        assert!(can_write_resource(&context_with_perms, "mcp"));
        assert!(!can_read_resource(&context_with_perms, "admin"));
    }

    #[test]
    fn test_role_checking_helpers() {
        let context =
            create_test_auth_context("user", vec!["user".to_string(), "editor".to_string()]);

        assert!(has_any_role(&context, &["user", "admin"]));
        assert!(has_all_roles(&context, &["user", "editor"]));
        assert!(!has_all_roles(&context, &["user", "admin"]));
        assert!(!has_any_role(&context, &["admin", "superuser"]));
    }
}
