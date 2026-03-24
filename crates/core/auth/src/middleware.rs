// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use crate::{AuthContext, AuthError, AuthenticationService, JwtTokenManager};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub struct SecurityMiddleware {
    auth_service: Arc<dyn AuthenticationService>,
    jwt_manager: Arc<JwtTokenManager>,
    require_auth: bool,
}

impl SecurityMiddleware {
    pub fn new(
        auth_service: Arc<dyn AuthenticationService>,
        jwt_manager: Arc<JwtTokenManager>,
        require_auth: bool,
    ) -> Self {
        Self {
            auth_service,
            jwt_manager,
            require_auth,
        }
    }

    pub async fn verify_request(&self, request: &AuthRequest) -> Result<AuthContext, AuthError> {
        debug!("Verifying request authentication");

        // If authentication is not required, return a default context
        if !self.require_auth {
            debug!("Authentication not required, creating anonymous context");
            return Ok(AuthContext {
                user_id: Uuid::nil(),
                username: "anonymous".to_string(),
                permissions: vec![],
                session_id: Uuid::nil(),
                expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
                issued_at: chrono::Utc::now(),
                roles: vec!["anonymous".to_string()],
            });
        }

        // Extract token from request
        let token = match &request.authorization {
            Some(auth_header) => {
                debug!("Authorization header found");
                self.jwt_manager.extract_token_from_header(auth_header)?
            }
            None => {
                warn!("No authorization header provided");
                return Err(AuthError::InvalidCredentials);
            }
        };

        // Verify token
        debug!("Verifying JWT token");
        let auth_context = self.auth_service.verify_token(token).await?;

        // Log successful authentication
        info!(
            user_id = %auth_context.user_id,
            username = %auth_context.username,
            session_id = %auth_context.session_id,
            "User authenticated successfully"
        );

        Ok(auth_context)
    }

    pub async fn check_permission(
        &self,
        user_id: &Uuid,
        required_permission: &crate::Permission,
    ) -> Result<bool, AuthError> {
        debug!(
            user_id = %user_id,
            resource = %required_permission.resource,
            action = %required_permission.action,
            "Checking user permission"
        );

        let has_permission = self
            .auth_service
            .has_permission(user_id, required_permission)
            .await?;

        if has_permission {
            debug!("Permission granted");
        } else {
            warn!(
                user_id = %user_id,
                resource = %required_permission.resource,
                action = %required_permission.action,
                "Permission denied"
            );
        }

        Ok(has_permission)
    }

    pub async fn verify_request_with_permission(
        &self,
        request: &AuthRequest,
        required_permission: &crate::Permission,
    ) -> Result<AuthContext, AuthError> {
        // First verify the request
        let auth_context = self.verify_request(request).await?;

        // If anonymous user and permission required, deny
        if auth_context.user_id == Uuid::nil() && self.require_auth {
            return Err(AuthError::InsufficientPermissions);
        }

        // Check permission if required
        if self.require_auth {
            let has_permission = self
                .check_permission(&auth_context.user_id, required_permission)
                .await?;
            if !has_permission {
                return Err(AuthError::InsufficientPermissions);
            }
        }

        Ok(auth_context)
    }

    pub fn extract_client_ip(&self, request: &AuthRequest) -> Option<String> {
        // Try to get IP from various headers
        if let Some(forwarded) = &request.x_forwarded_for {
            if let Some(ip) = forwarded.split(',').next() {
                return Some(ip.trim().to_string());
            }
        }

        if let Some(real_ip) = &request.x_real_ip {
            return Some(real_ip.clone());
        }

        request.remote_addr.clone()
    }

    pub async fn audit_request(
        &self,
        request: &AuthRequest,
        auth_context: Option<&AuthContext>,
        success: bool,
    ) -> Result<(), AuthError> {
        let client_ip = self.extract_client_ip(request);
        let username = auth_context
            .map(|ctx| ctx.username.as_str())
            .unwrap_or("unknown");

        info!(
            method = %request.method,
            path = %request.path,
            client_ip = ?client_ip,
            username = %username,
            success = %success,
            "Request audited"
        );

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AuthRequest {
    pub method: String,
    pub path: String,
    pub authorization: Option<String>,
    pub x_forwarded_for: Option<String>,
    pub x_real_ip: Option<String>,
    pub remote_addr: Option<String>,
    pub user_agent: Option<String>,
    pub content_type: Option<String>,
}

impl AuthRequest {
    pub fn new(method: String, path: String) -> Self {
        Self {
            method,
            path,
            authorization: None,
            x_forwarded_for: None,
            x_real_ip: None,
            remote_addr: None,
            user_agent: None,
            content_type: None,
        }
    }

    pub fn with_authorization(mut self, authorization: String) -> Self {
        self.authorization = Some(authorization);
        self
    }

    pub fn with_x_forwarded_for(mut self, x_forwarded_for: String) -> Self {
        self.x_forwarded_for = Some(x_forwarded_for);
        self
    }

    pub fn with_x_real_ip(mut self, x_real_ip: String) -> Self {
        self.x_real_ip = Some(x_real_ip);
        self
    }

    pub fn with_remote_addr(mut self, remote_addr: String) -> Self {
        self.remote_addr = Some(remote_addr);
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    pub fn with_content_type(mut self, content_type: String) -> Self {
        self.content_type = Some(content_type);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BeardogSecurityClient, Permission};
    use std::sync::Arc;
    use uuid::Uuid;

    struct MockAuthService;

    #[async_trait::async_trait]
    impl AuthenticationService for MockAuthService {
        async fn authenticate(
            &self,
            _credentials: &crate::LoginRequest,
        ) -> Result<AuthContext, AuthError> {
            Ok(AuthContext {
                user_id: Uuid::new_v4(),
                username: "mock_user".to_string(),
                permissions: vec![Permission::new("test", "read")],
                session_id: Uuid::new_v4(),
                expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
                issued_at: chrono::Utc::now(),
                roles: vec!["user".to_string()],
            })
        }

        async fn verify_token(&self, _token: &str) -> Result<AuthContext, AuthError> {
            Ok(AuthContext {
                user_id: Uuid::new_v4(),
                username: "test_user".to_string(),
                permissions: vec![],
                session_id: Uuid::new_v4(),
                expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
                issued_at: chrono::Utc::now(),
                roles: vec!["user".to_string()],
            })
        }

        async fn refresh_token(
            &self,
            _refresh_token: &str,
        ) -> Result<crate::LoginResponse, AuthError> {
            let user = crate::User::new("mock_user", "mock@example.com");
            Ok(crate::LoginResponse {
                access_token: "mock_access_token".to_string(),
                refresh_token: Some("mock_refresh_token".to_string()),
                token_type: "Bearer".to_string(),
                expires_in: 3600,
                user,
            })
        }

        async fn logout(&self, _session_id: &Uuid) -> Result<(), AuthError> {
            // Mock implementation - just return success
            Ok(())
        }

        async fn get_user_permissions(
            &self,
            _user_id: &Uuid,
        ) -> Result<Vec<Permission>, AuthError> {
            Ok(vec![])
        }

        async fn has_permission(
            &self,
            _user_id: &Uuid,
            _permission: &Permission,
        ) -> Result<bool, AuthError> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_verify_request_with_valid_token() {
        let auth_service = Arc::new(MockAuthService);
        let jwt_manager = Arc::new(JwtTokenManager::new(b"test-secret"));
        let middleware = SecurityMiddleware::new(auth_service, jwt_manager, true);

        let request = AuthRequest::new("GET".to_string(), "/api/test".to_string())
            .with_authorization("Bearer valid-token".to_string());

        let result = middleware.verify_request(&request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_request_without_auth_when_not_required() {
        let auth_service = Arc::new(MockAuthService);
        let jwt_manager = Arc::new(JwtTokenManager::new(b"test-secret"));
        let middleware = SecurityMiddleware::new(auth_service, jwt_manager, false);

        let request = AuthRequest::new("GET".to_string(), "/api/test".to_string());

        let result = middleware.verify_request(&request).await;
        assert!(result.is_ok());

        let auth_context = result.expect("Authentication should succeed for anonymous access");
        assert_eq!(auth_context.username, "anonymous");
    }

    #[tokio::test]
    async fn test_verify_request_without_auth_when_required() {
        let auth_service = Arc::new(MockAuthService);
        let jwt_manager = Arc::new(JwtTokenManager::new(b"test-secret"));
        let middleware = SecurityMiddleware::new(auth_service, jwt_manager, true);

        let request = AuthRequest::new("GET".to_string(), "/api/test".to_string());

        let result = middleware.verify_request(&request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidCredentials));
    }

    #[tokio::test]
    async fn test_extract_client_ip() {
        let auth_service = Arc::new(MockAuthService);
        let jwt_manager = Arc::new(JwtTokenManager::new(b"test-secret"));
        let middleware = SecurityMiddleware::new(auth_service, jwt_manager, false);

        let request = AuthRequest::new("GET".to_string(), "/api/test".to_string())
            .with_x_forwarded_for("192.168.1.100, 10.0.0.1".to_string());

        let ip = middleware.extract_client_ip(&request);
        assert_eq!(ip, Some("192.168.1.100".to_string()));
    }
}
