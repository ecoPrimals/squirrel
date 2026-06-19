// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Standalone authentication service for Squirrel MCP.
//!
//! Provides in-memory credential validation and session management.
//! Production environments should delegate to a discovered security capability
//! provider; this module is the failsafe standalone fallback.

use crate::errors::{AuthError, AuthResult};
use crate::session::SessionManager;
use crate::types::{AuthContext, AuthProvider, LoginResponse, Session, User};

use chrono::Duration;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Standalone authentication service.
///
/// Uses in-memory credential store and [`SessionManager`] for session lifecycle.
/// In production, prefer capability-discovered security providers; this service
/// is the autonomous fallback when no external provider is available.
pub struct AuthService {
    sessions: SessionManager,
    users: RwLock<HashMap<String, StoredUser>>,
    provider: AuthProvider,
}

struct StoredUser {
    user: User,
    password_hash: String,
}

impl AuthService {
    /// Create a new standalone auth service.
    ///
    /// Seeds a default `admin` / `admin123` user for development.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] if the service cannot initialize.
    pub fn new() -> AuthResult<Self> {
        let mut users = HashMap::new();

        let mut admin = User::new("admin", "admin@localhost");
        admin.roles = vec!["admin".to_string()];

        users.insert(
            "admin".to_string(),
            StoredUser {
                user: admin,
                password_hash: "admin123".to_string(),
            },
        );

        debug!("AuthService initialized (standalone mode, 1 seeded user)");

        Ok(Self {
            sessions: SessionManager::new(),
            users: RwLock::new(users),
            provider: AuthProvider::Standalone,
        })
    }

    /// Authenticate a login request against the local credential store.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] on internal failures (not on bad credentials —
    /// those are signalled via `LoginResponse.success == false`).
    pub async fn authenticate(
        &self,
        request: crate::types::LoginRequest,
    ) -> AuthResult<LoginResponse> {
        let req_username = request.username.clone();
        let req_password = request.password.clone();
        drop(request);

        let users = self.users.read().await;

        let Some(stored) = users.get(&req_username) else {
            warn!("Login attempt for unknown user: {}", req_username);
            drop(users);
            return Ok(LoginResponse {
                success: false,
                user_context: None,
                session_token: None,
                expires_at: None,
                error_message: Some("Invalid credentials".to_string()),
            });
        };

        if stored.password_hash != req_password {
            warn!("Failed login for user: {}", req_username);
            drop(users);
            return Ok(LoginResponse {
                success: false,
                user_context: None,
                session_token: None,
                expires_at: None,
                error_message: Some("Invalid credentials".to_string()),
            });
        }

        let duration = Duration::hours(8);
        let session = Session::new(stored.user.id, duration, self.provider.clone());
        let session_id = session.id;
        let expires_at = session.expires_at;
        let ctx = AuthContext::new(&stored.user, session_id, expires_at, self.provider.clone());
        drop(users);

        self.sessions.create_session(session).await?;

        debug!("Authenticated user: {}", req_username);

        Ok(LoginResponse {
            success: true,
            user_context: Some(ctx),
            session_token: Some(session_id.to_string()),
            expires_at: Some(expires_at),
            error_message: None,
        })
    }

    /// Validate an existing session token.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] if the session store cannot be read.
    pub async fn validate_session(&self, token: &str) -> AuthResult<Option<AuthContext>> {
        let session_id: uuid::Uuid = token
            .parse()
            .map_err(|_| AuthError::token_error("validate", "invalid session token format"))?;

        let Some(session) = self.sessions.get_session(&session_id).await? else {
            return Ok(None);
        };

        let context = {
            let users = self.users.read().await;
            users.values().find_map(|stored| {
                if stored.user.id == session.user_id {
                    Some(AuthContext::new(
                        &stored.user,
                        session.id,
                        session.expires_at,
                        session.auth_provider.clone(),
                    ))
                } else {
                    None
                }
            })
        };

        Ok(context)
    }

    /// Invalidate (logout) a session by token.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] if the session store cannot be updated.
    pub async fn logout(&self, token: &str) -> AuthResult<bool> {
        let session_id: uuid::Uuid = token
            .parse()
            .map_err(|_| AuthError::token_error("logout", "invalid session token format"))?;

        self.sessions.invalidate_session(&session_id).await
    }

    /// Return the authentication provider type for this service.
    #[must_use]
    pub fn get_auth_provider(&self) -> AuthProvider {
        self.provider.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LoginRequest;

    #[tokio::test]
    async fn standalone_auth_initializes() {
        let service = AuthService::new();
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn admin_login_succeeds() {
        let service = AuthService::new().expect("init");
        let resp = service
            .authenticate(LoginRequest::new("admin", "admin123"))
            .await
            .expect("auth");
        assert!(resp.success);
        assert!(resp.session_token.is_some());
    }

    #[tokio::test]
    async fn wrong_password_fails() {
        let service = AuthService::new().expect("init");
        let resp = service
            .authenticate(LoginRequest::new("admin", "wrong"))
            .await
            .expect("auth");
        assert!(!resp.success);
    }

    #[tokio::test]
    async fn unknown_user_fails() {
        let service = AuthService::new().expect("init");
        let resp = service
            .authenticate(LoginRequest::new("nobody", "pass"))
            .await
            .expect("auth");
        assert!(!resp.success);
    }

    #[tokio::test]
    async fn session_roundtrip() {
        let service = AuthService::new().expect("init");
        let resp = service
            .authenticate(LoginRequest::new("admin", "admin123"))
            .await
            .expect("auth");
        let token = resp.session_token.clone().expect("token");

        let ctx = service.validate_session(&token).await.expect("validate");
        assert!(ctx.is_some());
        assert_eq!(ctx.expect("ctx").username, "admin");
    }

    #[tokio::test]
    async fn logout_invalidates_session() {
        let service = AuthService::new().expect("init");
        let resp = service
            .authenticate(LoginRequest::new("admin", "admin123"))
            .await
            .expect("auth");
        let token = resp.session_token.clone().expect("token");

        let ok = service.logout(&token).await.expect("logout");
        assert!(ok);

        let ctx = service.validate_session(&token).await.expect("validate");
        assert!(ctx.is_none());
    }

    #[tokio::test]
    async fn provider_is_standalone() {
        let service = AuthService::new().expect("init");
        assert!(matches!(
            service.get_auth_provider(),
            AuthProvider::Standalone
        ));
    }
}
