// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Login flows, session validation, and credential helpers.

use super::AuthService;
use crate::errors::{AuthError, AuthResult};
use crate::session::Session;
use crate::types::{
    AuthContext, AuthProvider, LoginRequest, LoginResponse, SecurityCapabilityInfo, User,
};
use chrono::Duration;
use serde_json::json;
use tracing::debug;
use uuid::Uuid;

impl AuthService {
    /// Authenticate user with discovered security capability
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] if network I/O, HTTP, JSON parsing, or session storage fails.
    pub async fn authenticate(&self, request: LoginRequest) -> AuthResult<LoginResponse> {
        match &self.auth_provider {
            AuthProvider::SecurityCapability {
                endpoint,
                capability_info,
                ..
            } => {
                self.authenticate_with_security_capability(request, endpoint, capability_info)
                    .await
            }
            AuthProvider::Standalone => self.authenticate_standalone(request).await,
            AuthProvider::Development => self.authenticate_development(request).await,
        }
    }

    /// Authenticate using any discovered security capability - completely generic
    async fn authenticate_with_security_capability(
        &self,
        request: LoginRequest,
        endpoint: &str,
        capability_info: &SecurityCapabilityInfo,
    ) -> AuthResult<LoginResponse> {
        // Use generic auth API patterns that work across primals
        let auth_url = format!("{}/api/auth/login", endpoint.trim_end_matches('/'));

        let payload = json!({
            "username": request.username,
            "password": request.password,
            "additional_factors": request.additional_factors,
            "client_info": {
                "primal": "squirrel",
                "version": "v1"
            }
        });

        debug!(
            "Authenticating with security capability at: {} (type: {})",
            auth_url, capability_info.primal_type
        );

        let response = self.client.post(&auth_url).json(&payload).send().await?;

        if response.status().is_success() {
            let auth_data: serde_json::Value = response.json().await?;

            // Parse generic security response
            let user = Self::parse_security_user_response(&auth_data)?;
            let session_duration = Duration::hours(8); // Default 8-hour session
            let session = Session::new(user.id, session_duration, self.auth_provider.clone());

            let auth_context = AuthContext::new(
                &user,
                session.id,
                session.expires_at,
                self.auth_provider.clone(),
            );

            // Store session and get needed values before moving
            let session_id = session.id;
            let expires_at = session.expires_at;
            self.session_manager.create_session(session).await?;

            Ok(LoginResponse {
                success: true,
                user_context: Some(auth_context),
                session_token: Some(session_id.to_string()),
                expires_at: Some(expires_at),
                error_message: None,
            })
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Ok(LoginResponse {
                success: false,
                user_context: None,
                session_token: None,
                expires_at: None,
                error_message: Some(format!(
                    "Security capability authentication failed: {error_text}"
                )),
            })
        }
    }

    /// Standalone authentication (failsafe fallback)
    async fn authenticate_standalone(&self, request: LoginRequest) -> AuthResult<LoginResponse> {
        debug!("Authenticating in standalone mode");

        // Simple credential check for demo/fallback
        if let Some(user) = self.users.get(&request.username) {
            // In a real implementation, you'd hash and compare passwords properly
            if Self::verify_password(&request.password, &request.username) {
                let session_duration = Duration::hours(8);
                let session = Session::new(user.id, session_duration, AuthProvider::Standalone);

                let auth_context = AuthContext::new(
                    user,
                    session.id,
                    session.expires_at,
                    AuthProvider::Standalone,
                );

                // Store session and get needed values before moving
                let session_id = session.id;
                let expires_at = session.expires_at;
                self.session_manager.create_session(session).await?;

                Ok(LoginResponse {
                    success: true,
                    user_context: Some(auth_context),
                    session_token: Some(session_id.to_string()),
                    expires_at: Some(expires_at),
                    error_message: None,
                })
            } else {
                Ok(LoginResponse {
                    success: false,
                    user_context: None,
                    session_token: None,
                    expires_at: None,
                    error_message: Some("Invalid credentials".to_string()),
                })
            }
        } else {
            Ok(LoginResponse {
                success: false,
                user_context: None,
                session_token: None,
                expires_at: None,
                error_message: Some("User not found".to_string()),
            })
        }
    }

    /// Development authentication (always succeeds for testing)
    async fn authenticate_development(&self, request: LoginRequest) -> AuthResult<LoginResponse> {
        debug!("Authenticating in development mode");

        let user = User::new(&request.username, format!("{}@dev.local", request.username));
        let session_duration = Duration::hours(24); // Long session for dev
        let session = Session::new(user.id, session_duration, AuthProvider::Development);

        let auth_context = AuthContext::new(
            &user,
            session.id,
            session.expires_at,
            AuthProvider::Development,
        );

        let session_id = session.id;
        let expires_at = session.expires_at;
        self.session_manager.create_session(session).await?;

        Ok(LoginResponse {
            success: true,
            user_context: Some(auth_context),
            session_token: Some(session_id.to_string()),
            expires_at: Some(expires_at),
            error_message: None,
        })
    }

    /// Validate a session token
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] if the token is malformed or session lookup fails.
    pub async fn validate_session(&self, session_token: &str) -> AuthResult<Option<AuthContext>> {
        let session_id = Uuid::parse_str(session_token)
            .map_err(|e| AuthError::token_error("parse", e.to_string()))?;

        if let Some(session) = self.session_manager.get_session(&session_id).await? {
            if session.is_expired() || !session.is_active {
                return Ok(None);
            }

            // Get user information based on auth provider
            if let Some(user) = self.get_user_by_id(&session.user_id)? {
                let auth_context = AuthContext::new(
                    &user,
                    session.id,
                    session.expires_at,
                    session.auth_provider.clone(),
                );
                Ok(Some(auth_context))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Invalidate a session
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] if the token cannot be parsed or invalidation fails.
    pub async fn logout(&self, session_token: &str) -> AuthResult<bool> {
        let session_id = Uuid::parse_str(session_token)
            .map_err(|e| AuthError::token_error("parse", e.to_string()))?;

        self.session_manager.invalidate_session(&session_id).await
    }

    /// Get current authentication provider
    pub const fn get_auth_provider(&self) -> &AuthProvider {
        &self.auth_provider
    }

    // Helper methods

    fn parse_security_user_response(data: &serde_json::Value) -> AuthResult<User> {
        let username = data["username"]
            .as_str()
            .or_else(|| data["user"]["username"].as_str())
            .ok_or_else(|| AuthError::internal_error("Missing username in security response"))?;

        let default_email = format!("{username}@security.local");
        let email = data["email"]
            .as_str()
            .or_else(|| data["user"]["email"].as_str())
            .unwrap_or(&default_email);

        let mut user = User::new(username, email);

        // Parse roles if available from any security provider format
        if let Some(roles) = data["roles"]
            .as_array()
            .or_else(|| data["user"]["roles"].as_array())
        {
            for role in roles {
                if let Some(role_str) = role.as_str() {
                    user.roles.push(role_str.to_string());
                }
            }
        }

        Ok(user)
    }

    fn verify_password(password: &str, username: &str) -> bool {
        // Simple fallback verification - in production use proper hashing
        match username {
            "admin" => password == "admin123",
            "user" => password == "user123",
            _ => false,
        }
    }

    fn get_user_by_id(&self, user_id: &Uuid) -> AuthResult<Option<User>> {
        // In standalone mode, find user by ID
        for user in self.users.values() {
            if user.id == *user_id {
                return Ok(Some(user.clone()));
            }
        }
        Ok(None)
    }
}
