//! Modern authentication service with capability discovery and standalone fallback
//!
//! Uses universal adapter pattern for capability discovery - no hardcoded primal dependencies.
//! Discovers any primal providing security/auth capabilities through network effects.

use crate::errors::{AuthError, AuthResult};
use crate::session::{Session, SessionManager};
use crate::types::{AuthContext, AuthProvider, LoginRequest, LoginResponse, User, SecurityCapabilityInfo};

use chrono::Duration;
use reqwest::Client;
use serde_json::json;
// Removed: use squirrel_mcp_config::get_service_endpoints;
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Modern authentication service supporting capability discovery and standalone fallback
#[derive(Debug)]
pub struct AuthService {
    /// HTTP client for external auth requests
    client: Client,
    /// Session manager for handling user sessions
    session_manager: SessionManager,
    /// Current authentication provider configuration
    auth_provider: AuthProvider,
    /// In-memory user store (for standalone mode)
    users: HashMap<String, User>,
}

impl AuthService {
    /// Create a new authentication service with pure capability discovery
    pub async fn new() -> AuthResult<Self> {
        let client = Client::new();
        let session_manager = SessionManager::new();
        
        // Pure capability discovery - no hardcoded primal dependencies
        let auth_provider = Self::discover_security_capability(&client).await;
        
        info!("Initialized auth service with provider: {:?}", auth_provider);
        
        // Initialize with some default users for standalone mode
        let mut users = HashMap::new();
        users.insert(
            "admin".to_string(),
            Self::create_default_admin_user(),
        );
        users.insert(
            "user".to_string(),
            Self::create_default_user(),
        );
        
        Ok(Self {
            client,
            session_manager,
            auth_provider,
            users,
        })
    }

    /// Discover security capability through universal adapter - no hardcoded primal knowledge
    async fn discover_security_capability(client: &Client) -> AuthProvider {
        // Try to discover ANY primal with security capabilities through universal adapter
        let security_endpoint = std::env::var("SECURITY_SERVICE_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8443".to_string());
        
        debug!("Attempting security capability discovery at: {}", security_endpoint);
        
        match Self::test_security_capability(client, security_endpoint).await {
            Ok(capability_info) => {
                info!("Security capability discovered: {:?}", capability_info);
                AuthProvider::SecurityCapability {
                    endpoint: security_endpoint.clone(),
                    discovery_method: "universal_adapter_discovery".to_string(),
                    capability_info,
                }
            }
            Err(e) => {
                debug!("Security capability discovery failed: {}. Using standalone fallback", e);
                AuthProvider::Standalone
            }
        }
    }

    /// Test any primal for security capability - completely generic
    async fn test_security_capability(client: &Client, endpoint: &str) -> AuthResult<SecurityCapabilityInfo> {
        let health_url = format!("{}/health", endpoint.trim_end_matches('/'));
        
        let response = client
            .get(&health_url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await?;

        if response.status().is_success() {
            // Check for generic security capability indicators
            if let Ok(body) = response.text().await {
                let capability_info = Self::parse_security_capability(&body)?;
                Ok(capability_info)
            } else {
                Err(AuthError::network_error("capability_test", "No response body"))
            }
        } else {
            Err(AuthError::network_error("capability_test", format!("HTTP {}", response.status())))
        }
    }

    /// Parse security capability information from any primal
    fn parse_security_capability(response_body: &str) -> AuthResult<SecurityCapabilityInfo> {
        // Look for generic security capability indicators (not primal-specific)
        let has_auth = response_body.contains("auth") || response_body.contains("authentication");
        let has_security = response_body.contains("security") || response_body.contains("secure");
        let has_session = response_body.contains("session") || response_body.contains("token");
        
        if has_auth || has_security || has_session {
            // Try to determine what type of primal this is from generic indicators
            let primal_type = if response_body.contains("beardog") || response_body.contains("BearDog") {
                "beardog".to_string()
            } else if response_body.contains("toadstool") || response_body.contains("ToadStool") {
                "toadstool".to_string()
            } else if response_body.contains("songbird") || response_body.contains("SongBird") {
                "songbird".to_string()
            } else {
                "unknown".to_string()
            };
            
            Ok(SecurityCapabilityInfo {
                primal_type,
                supports_auth: has_auth,
                supports_sessions: has_session,
                api_version: "v1".to_string(), // Default
            })
        } else {
            Err(AuthError::authorization_error("No security capabilities detected"))
        }
    }

    /// Authenticate user with discovered security capability
    pub async fn authenticate(&mut self, request: LoginRequest) -> AuthResult<LoginResponse> {
        match &self.auth_provider {
            AuthProvider::SecurityCapability { endpoint, capability_info, .. } => {
                let endpoint_clone = endpoint.clone();
                let capability_clone = capability_info.clone();
                self.authenticate_with_security_capability(request, &endpoint_clone, &capability_clone).await
            }
            AuthProvider::Standalone => {
                self.authenticate_standalone(request).await
            }
            AuthProvider::Development => {
                self.authenticate_development(request).await
            }
        }
    }

    /// Authenticate using any discovered security capability - completely generic
    async fn authenticate_with_security_capability(
        &mut self,
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

        debug!("Authenticating with security capability at: {} (type: {})", auth_url, capability_info.primal_type);

        let response = self
            .client
            .post(&auth_url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let auth_data: serde_json::Value = response.json().await?;
            
            // Parse generic security response
            let user = self.parse_security_user_response(&auth_data)?;
            let session_duration = Duration::hours(8); // Default 8-hour session
            let session = Session::new(
                user.id,
                session_duration,
                self.auth_provider.clone(),
            );

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
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Ok(LoginResponse {
                success: false,
                user_context: None,
                session_token: None,
                expires_at: None,
                error_message: Some(format!("Security capability authentication failed: {}", error_text)),
            })
        }
    }

    /// Standalone authentication (failsafe fallback)
    async fn authenticate_standalone(&mut self, request: LoginRequest) -> AuthResult<LoginResponse> {
        debug!("Authenticating in standalone mode");

        // Simple credential check for demo/fallback
        if let Some(user) = self.users.get(&request.username) {
            // In a real implementation, you'd hash and compare passwords properly
            if self.verify_password(&request.password, &request.username) {
                let session_duration = Duration::hours(8);
                let session = Session::new(
                    user.id,
                    session_duration,
                    AuthProvider::Standalone,
                );

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
    async fn authenticate_development(&mut self, request: LoginRequest) -> AuthResult<LoginResponse> {
        debug!("Authenticating in development mode");

        let user = User::new(&request.username, &format!("{}@dev.local", request.username));
        let session_duration = Duration::hours(24); // Long session for dev
        let session = Session::new(
            user.id,
            session_duration,
            AuthProvider::Development,
        );

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
    pub async fn validate_session(&self, session_token: &str) -> AuthResult<Option<AuthContext>> {
        let session_id = Uuid::parse_str(session_token)
            .map_err(|e| AuthError::token_error("parse", e.to_string()))?;

        if let Some(session) = self.session_manager.get_session(&session_id).await? {
            if session.is_expired() || !session.is_active {
                return Ok(None);
            }

            // Get user information based on auth provider
            if let Some(user) = self.get_user_by_id(&session.user_id).await? {
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
    pub async fn logout(&mut self, session_token: &str) -> AuthResult<bool> {
        let session_id = Uuid::parse_str(session_token)
            .map_err(|e| AuthError::token_error("parse", e.to_string()))?;

        self.session_manager.invalidate_session(&session_id).await
    }

    /// Get current authentication provider
    pub fn get_auth_provider(&self) -> &AuthProvider {
        &self.auth_provider
    }

    // Helper methods

    fn parse_security_user_response(&self, data: &serde_json::Value) -> AuthResult<User> {
        let username = data["username"]
            .as_str()
            .or_else(|| data["user"]["username"].as_str())
            .ok_or_else(|| AuthError::internal_error("Missing username in security response"))?;
        
        let default_email = format!("{}@security.local", username);
        let email = data["email"]
            .as_str()
            .or_else(|| data["user"]["email"].as_str())
            .unwrap_or(&default_email);

        let mut user = User::new(username, email);
        
        // Parse roles if available from any security provider format
        if let Some(roles) = data["roles"].as_array().or_else(|| data["user"]["roles"].as_array()) {
            for role in roles {
                if let Some(role_str) = role.as_str() {
                    user.roles.push(role_str.to_string());
                }
            }
        }

        Ok(user)
    }

    fn verify_password(&self, password: &str, username: &str) -> bool {
        // Simple fallback verification - in production use proper hashing
        match username {
            "admin" => password == "admin123",
            "user" => password == "user123",
            _ => false,
        }
    }

    async fn get_user_by_id(&self, user_id: &Uuid) -> AuthResult<Option<User>> {
        // In standalone mode, find user by ID
        for user in self.users.values() {
            if user.id == *user_id {
                return Ok(Some(user.clone()));
            }
        }
        Ok(None)
    }

    fn create_default_admin_user() -> User {
        let mut user = User::new("admin", "admin@squirrel.local");
        user.roles.push("admin".to_string());
        user.roles.push("user".to_string());
        user
    }

    fn create_default_user() -> User {
        let mut user = User::new("user", "user@squirrel.local");
        user.roles.push("user".to_string());
        user
    }
} 