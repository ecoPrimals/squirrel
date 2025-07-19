//! Security configuration types for Squirrel MCP
//!
//! This module defines security-related configuration including
//! authentication backends, JWT settings, and security policies.

use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

/// Security configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    pub backend: SecurityBackend,
    pub jwt_secret_key_id: String,
    pub jwt_expiration: Duration,
    pub encryption_algorithm: String,
    pub hsm_provider: String,
    pub authentication_required: bool,
    pub session_timeout: Duration,
    pub max_failed_attempts: u32,
    pub lockout_duration: Duration,
}

/// Security backend options
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SecurityBackend {
    #[serde(rename = "beardog")]
    BearDog,
    #[serde(rename = "internal")]
    Internal,
}

/// BearDog specific configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BeardogConfig {
    pub auth_endpoint: String,
    pub jwt_secret_key_id: String,
    pub encryption_algorithm: String,
    pub hsm_provider: String,
    pub compliance_mode: String,
    pub audit_enabled: bool,
    pub timeout: Duration,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            backend: SecurityBackend::BearDog,
            jwt_secret_key_id: "squirrel-mcp-jwt".to_string(),
            jwt_expiration: Duration::from_secs(3600),
            encryption_algorithm: "aes-256-gcm".to_string(),
            hsm_provider: "softhsm".to_string(),
            authentication_required: true,
            session_timeout: Duration::from_secs(1800),
            max_failed_attempts: 5,
            lockout_duration: Duration::from_secs(300),
        }
    }
}

impl Default for BeardogConfig {
    fn default() -> Self {
        Self {
            auth_endpoint: env::var("BEARDOG_AUTH_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            jwt_secret_key_id: env::var("BEARDOG_JWT_SECRET_KEY_ID")
                .unwrap_or_else(|_| "squirrel-mcp-jwt".to_string()),
            encryption_algorithm: env::var("BEARDOG_ENCRYPTION_ALGORITHM")
                .unwrap_or_else(|_| "AES-256-GCM".to_string()),
            hsm_provider: env::var("BEARDOG_HSM_PROVIDER")
                .unwrap_or_else(|_| "SoftHSM".to_string()),
            compliance_mode: env::var("BEARDOG_COMPLIANCE_MODE")
                .unwrap_or_else(|_| "development".to_string()),
            audit_enabled: env::var("BEARDOG_AUDIT_ENABLED")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(true),
            timeout: Duration::from_secs(
                env::var("BEARDOG_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(3600),
            ),
        }
    }
}
