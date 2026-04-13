// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Authentication provider implementations.
//!
//! This module contains the security provider integration,
//! including authentication, encryption, and compliance monitoring.
//!
//! **REQUIRES**: http-auth feature (brings reqwest → ring)

use super::types::{AuditEvent, ComplianceCheck, Permission, TokenResponse};
use anyhow::anyhow;
use crate::{Result, Error};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::capability_discovery::SecurityServiceDiscovery;
use uuid::Uuid;

/// Security authentication provider
#[derive(Debug, Clone)]
pub struct AuthProvider {
    client: Client,
    base_url: String,
    api_key: String,
}

/// Security provider authentication context (internal)
#[derive(Debug, Clone)]
pub struct BeardogAuthContext {
    pub user_id: Uuid,
    pub username: String,
    pub permissions: Vec<BeardogPermission>,
    pub session_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub issued_at: DateTime<Utc>,
    pub roles: Vec<String>,
}

/// Security provider permission structure (internal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeardogPermission {
    pub resource: String,
    pub action: String,
    pub scope: Option<String>,
}

/// Authentication request
#[derive(Debug, Clone)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
    pub remember_me: bool,
}

/// Session information from security provider
#[derive(Debug, Clone)]
pub struct BeardogSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub roles: Vec<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// User information from security provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<BeardogPermission>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub login_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
    pub active: bool,
}

/// Internal API request structures
#[derive(Serialize, Deserialize)]
struct AuthApiRequest {
    username: String,
    password: String,
    remember_me: bool,
}

#[derive(Serialize, Deserialize)]
struct AuthApiResponse {
    user_id: String,
    username: String,
    permissions: Vec<PermissionApi>,
    session_id: String,
    expires_at: String,
    issued_at: String,
    roles: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct PermissionApi {
    resource: String,
    action: String,
    scope: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct JwtVerifyRequest {
    token: String,
}

#[derive(Serialize, Deserialize)]
struct SessionCreateRequest {
    id: String,
    user_id: String,
    username: String,
    roles: Vec<String>,
    expires_at: String,
    created_at: String,
}

impl AuthProvider {
    /// Create a new security authentication provider
    pub async fn new(endpoint: &str, api_key: &str) -> Result<Self> {
        Self::new_with_timeout(endpoint, api_key, std::time::Duration::from_secs(30)).await
    }

    /// Create a new security authentication provider with custom timeout
    pub async fn new_with_timeout(
        endpoint: &str,
        api_key: &str,
        timeout: Duration,
    ) -> Result<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            client,
            base_url: endpoint.to_string(),
            api_key: api_key.to_string(),
        })
    }

    /// Verify a JWT token
    pub async fn verify_jwt(&self, token: &str) -> Result<crate::JwtClaims> {
        let request = JwtVerifyRequest {
            token: token.to_string(),
        };

        let response = self
            .client
            .post(&format!("{}/auth/verify-jwt", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("JWT verification request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "JWT verification failed: {}",
                response.status()
            ));
        }

        let claims: crate::JwtClaims = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse JWT claims: {}", e))?;

        Ok(claims)
    }

    /// Get user permissions
    pub async fn get_permissions(&self, user_id: &Uuid) -> Result<Vec<BeardogPermission>> {
        let response = self
            .client
            .get(&format!(
                "{}/auth/users/{}/permissions",
                self.base_url, user_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| anyhow!("Permissions request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to get permissions: {}",
                response.status()
            ));
        }

        let permissions: Vec<PermissionApi> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse permissions: {}", e))?;

        Ok(permissions
            .into_iter()
            .map(|p| BeardogPermission {
                resource: p.resource,
                action: p.action,
                scope: p.scope,
            })
            .collect())
    }

    /// Create a session
    pub async fn create_session(&self, session: &BeardogSession) -> Result<()> {
        let request = SessionCreateRequest {
            id: session.id.to_string(),
            user_id: session.user_id.to_string(),
            username: session.username.clone(),
            roles: session.roles.clone(),
            expires_at: session.expires_at.to_rfc3339(),
            created_at: session.created_at.to_rfc3339(),
        };

        let response = self
            .client
            .post(&format!("{}/auth/sessions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Session creation request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to create session: {}",
                response.status()
            ));
        }

        Ok(())
    }

    /// Authenticate user credentials
    pub async fn authenticate(&self, request: &AuthRequest) -> Result<BeardogAuthContext> {
        let auth_request = AuthApiRequest {
            username: request.username.clone(),
            password: request.password.clone(),
            remember_me: request.remember_me,
        };

        let response = self
            .client
            .post(&format!("{}/auth/authenticate", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&auth_request)
            .send()
            .await
            .map_err(|e| anyhow!("Authentication request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Authentication failed: {}",
                response.status()
            ));
        }

        let auth_response: AuthApiResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse authentication response: {}", e))?;

        let user_id = Uuid::parse_str(&auth_response.user_id)
            .map_err(|e| anyhow!("Invalid user ID: {}", e))?;

        let session_id = Uuid::parse_str(&auth_response.session_id)
            .map_err(|e| anyhow!("Invalid session ID: {}", e))?;

        let expires_at = DateTime::parse_from_rfc3339(&auth_response.expires_at)
            .map_err(|e| anyhow!("Invalid expires_at: {}", e))?
            .with_timezone(&Utc);

        let issued_at = DateTime::parse_from_rfc3339(&auth_response.issued_at)
            .map_err(|e| anyhow!("Invalid issued_at: {}", e))?
            .with_timezone(&Utc);

        let permissions = auth_response
            .permissions
            .into_iter()
            .map(|p| BeardogPermission {
                resource: p.resource,
                action: p.action,
                scope: p.scope,
            })
            .collect();

        Ok(BeardogAuthContext {
            user_id,
            username: auth_response.username,
            permissions,
            session_id,
            expires_at,
            issued_at,
            roles: auth_response.roles,
        })
    }

    /// Refresh authentication token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        let request = serde_json::json!({
            "refresh_token": refresh_token
        });

        let response = self
            .client
            .post(&format!("{}/auth/refresh", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Token refresh request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Token refresh failed: {}",
                response.status()
            ));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse token response: {}", e))?;

        Ok(token_response)
    }

    /// Get user information
    pub async fn get_user_info(&self, user_id: &Uuid) -> Result<UserInfo> {
        let response = self
            .client
            .get(&format!("{}/auth/users/{}", self.base_url, user_id))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| anyhow!("User info request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to get user info: {}",
                response.status()
            ));
        }

        let user_info: UserInfo = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse user info: {}", e))?;

        Ok(user_info)
    }

    /// Invalidate a session
    pub async fn invalidate_session(&self, session_id: &Uuid) -> Result<()> {
        let response = self
            .client
            .delete(&format!("{}/auth/sessions/{}", self.base_url, session_id))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| anyhow!("Session invalidation request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to invalidate session: {}",
                response.status()
            ));
        }

        Ok(())
    }

    /// Check if user has specific permission
    pub async fn has_permission(
        &self,
        user_id: &Uuid,
        permission: &BeardogPermission,
    ) -> Result<bool> {
        let request = serde_json::json!({
            "resource": permission.resource,
            "action": permission.action,
            "scope": permission.scope
        });

        let response = self
            .client
            .post(&format!(
                "{}/auth/users/{}/check-permission",
                self.base_url, user_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Permission check request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Permission check failed: {}",
                response.status()
            ));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse permission check response: {}", e))?;

        Ok(result
            .get("has_permission")
            .and_then(|v| v.as_bool())
            .unwrap_or(false))
    }
}

/// Security provider encryption service
#[derive(Debug, Clone)]
pub struct EncryptionService {
    client: Client,
    base_url: String,
    api_key: String,
}

impl EncryptionService {
    /// Discover encryption service endpoint via capability matching
    async fn discover_encryption_service_endpoint() -> Result<String> {
        let discovery = SecurityServiceDiscovery::new();
        discovery.discover_encryption_service().await
    }
    /// Create a new encryption service
    pub async fn new(algorithm: &str, hsm_provider: &str) -> Result<Self> {
        // Discover encryption service through capability adapter - no hardcoded endpoints
        let base_url = Self::discover_encryption_service_endpoint().await?;

        // Prefer SECURITY_API_KEY (capability-oriented). BEARDOG_API_KEY is legacy compatibility only.
        let api_key = std::env::var("SECURITY_API_KEY")
            .or_else(|_| std::env::var("BEARDOG_API_KEY"))
            .unwrap_or_default();

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        // Initialize encryption service with algorithm and HSM provider
        let init_request = serde_json::json!({
            "algorithm": algorithm,
            "hsm_provider": hsm_provider
        });

        let response = client
            .post(&format!("{}/encryption/initialize", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&init_request)
            .send()
            .await
            .map_err(|e| anyhow!("Encryption service initialization failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to initialize encryption service: {}",
                response.status()
            ));
        }

        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    /// Encrypt data with context
    pub async fn encrypt_with_context(&self, data: &[u8], context: &str) -> Result<Vec<u8>> {
        let request = serde_json::json!({
            "data": STANDARD.encode(data),
            "context": context
        });

        let response = self
            .client
            .post(&format!("{}/encryption/encrypt", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Encryption request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Encryption failed: {}",
                response.status()
            ));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse encryption response: {}", e))?;

        let encrypted_data = result
            .get("encrypted_data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing encrypted_data in response"))?;

        STANDARD.decode(encrypted_data)
            .map_err(|e| anyhow!("Failed to decode encrypted data: {}", e))
    }

    /// Decrypt data with context
    pub async fn decrypt_with_context(&self, data: &[u8], context: &str) -> Result<Vec<u8>> {
        let request = serde_json::json!({
            "data": STANDARD.encode(data),
            "context": context
        });

        let response = self
            .client
            .post(&format!("{}/encryption/decrypt", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Decryption request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Decryption failed: {}",
                response.status()
            ));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse decryption response: {}", e))?;

        let decrypted_data = result
            .get("decrypted_data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing decrypted_data in response"))?;

        STANDARD.decode(decrypted_data)
            .map_err(|e| anyhow!("Failed to decode decrypted data: {}", e))
    }
}

/// Security provider compliance monitoring service
#[derive(Debug, Clone)]
pub struct ComplianceMonitor {
    client: Client,
    base_url: String,
    api_key: String,
}

impl ComplianceMonitor {
    /// Discover compliance service endpoint via capability matching
    async fn discover_compliance_service_endpoint() -> Result<String> {
        let discovery = SecurityServiceDiscovery::new();
        discovery.discover_compliance_service().await
    }
    /// Create a new compliance monitor
    pub async fn new(mode: &str, audit_enabled: bool) -> Result<Self> {
        // Discover compliance service through capability adapter - no hardcoded endpoints  
        let base_url = Self::discover_compliance_service_endpoint().await?;

        // Prefer SECURITY_API_KEY (capability-oriented). BEARDOG_API_KEY is legacy compatibility only.
        let api_key = std::env::var("SECURITY_API_KEY")
            .or_else(|_| std::env::var("BEARDOG_API_KEY"))
            .unwrap_or_default();

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        // Initialize compliance monitor
        let init_request = serde_json::json!({
            "mode": mode,
            "audit_enabled": audit_enabled
        });

        let response = client
            .post(&format!("{}/compliance/initialize", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&init_request)
            .send()
            .await
            .map_err(|e| anyhow!("Compliance monitor initialization failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to initialize compliance monitor: {}",
                response.status()
            ));
        }

        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    /// Record an audit event
    pub async fn record_event(&self, event: &AuditEvent) -> Result<()> {
        let request = serde_json::json!({
            "event_type": event.event_type,
            "user_id": event.user_id.map(|id| id.to_string()),
            "username": event.username,
            "success": event.success,
            "ip_address": event.ip_address,
            "timestamp": event.timestamp.to_rfc3339(),
            "details": event.details
        });

        let response = self
            .client
            .post(&format!("{}/compliance/audit", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Audit event recording failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to record audit event: {}",
                response.status()
            ));
        }

        Ok(())
    }

    /// Check compliance for a specific action
    pub async fn check_compliance(&self, check: &ComplianceCheck) -> Result<bool> {
        let request = serde_json::json!({
            "user_id": check.user_id.to_string(),
            "action": check.action,
            "resource": check.resource,
            "timestamp": check.timestamp.to_rfc3339()
        });

        let response = self
            .client
            .post(&format!("{}/compliance/check", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Compliance check failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Compliance check failed: {}",
                response.status()
            ));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse compliance check response: {}", e))?;

        Ok(result
            .get("compliant")
            .and_then(|v| v.as_bool())
            .unwrap_or(false))
    }
}

/// Utility functions for converting between provider-internal and public types
impl From<BeardogPermission> for Permission {
    fn from(beardog_perm: BeardogPermission) -> Self {
        Permission {
            resource: beardog_perm.resource,
            action: beardog_perm.action,
            scope: beardog_perm.scope,
        }
    }
}

impl From<Permission> for BeardogPermission {
    fn from(permission: Permission) -> Self {
        BeardogPermission {
            resource: permission.resource,
            action: permission.action,
            scope: permission.scope,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_conversion() {
        let beardog_perm = BeardogPermission {
            resource: "mcp".to_string(),
            action: "read".to_string(),
            scope: Some("scope1".to_string()),
        };

        let permission: Permission = beardog_perm.clone().into();
        assert_eq!(permission.resource, beardog_perm.resource);
        assert_eq!(permission.action, beardog_perm.action);
        assert_eq!(permission.scope, beardog_perm.scope);

        let converted_back: BeardogPermission = permission.into();
        assert_eq!(converted_back.resource, beardog_perm.resource);
        assert_eq!(converted_back.action, beardog_perm.action);
        assert_eq!(converted_back.scope, beardog_perm.scope);
    }

    #[test]
    fn test_auth_request_creation() {
        let request = AuthRequest {
            username: "test_user".to_string(),
            password: "password123".to_string(),
            remember_me: true,
        };

        assert_eq!(request.username, "test_user");
        assert_eq!(request.password, "password123");
        assert!(request.remember_me);
    }

    #[test]
    fn test_beardog_session_creation() {
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let expires_at = Utc::now() + chrono::Duration::hours(1);
        let created_at = Utc::now();

        let session = BeardogSession {
            id: session_id,
            user_id,
            username: "test_user".to_string(),
            roles: vec!["user".to_string()],
            expires_at,
            created_at,
        };

        assert_eq!(session.id, session_id);
        assert_eq!(session.user_id, user_id);
        assert_eq!(session.username, "test_user");
        assert_eq!(session.roles, vec!["user".to_string()]);
    }
}
