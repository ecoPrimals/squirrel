// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(
    clippy::option_if_let_else,
    clippy::unnecessary_map_or,
    clippy::unused_self,
    clippy::unnecessary_wraps,
    reason = "Auth subsystem; progressive style and documentation tightening"
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![warn(missing_docs)]
//! Squirrel Authentication & Security System
//!
//! Modern authentication system leveraging capability-based discovery and ecosystem integration.
//!
//! This completely replaces the severely outdated legacy auth code with a clean, modern architecture.
//!
//! ## TRUE ecoBin Architecture (v1.3.0) via Capability Discovery
//!
//! - **Production Mode**: JWT validation delegated to capability provider (Pure Rust!)
//! - **Dev Mode**: Local JWT validation via feature flag (for fast iteration)
//! - **Capability-Based**: Discovers JWT validation capability at runtime (not hardcoded!)
//! - **Zero C Dependencies**: Production mode uses Pure Rust stack
//!
//! ## Capability Discovery Pattern
//!
//! Instead of hardcoding a specific primal, we discover capabilities:
//! - Squirrel asks: "Who provides jwt.validate capability?"
//! - Currently: Security provider primal provides it
//! - Future: Any primal with JWT capability can provide it
//!
//! ## Features
//!
//! - **Clean Error Handling**: Using thiserror with detailed error context
//! - **Capability Integration**: JWT delegation via Unix socket JSON-RPC
//! - **Ecosystem Integration**: Deep integration with Squirrel MCP configuration
//! - **Modern Rust Patterns**: No anyhow conflicts, clean Result types
//! - **Feature-Gated JWT**: Production (delegated) vs Dev (local)
//!
//! ## Usage
//!
//! ```rust,no_run
//! use squirrel_mcp_auth::{AuthResult, capability_jwt::CapabilityJwtConfig};
//!
//! # fn example() -> AuthResult<()> {
//! let config = CapabilityJwtConfig::default();
//! // Production: DelegatedJwtClient::new(config)?
//! // Dev: DelegatedJwtClient::new_dev(secret)?
//! # Ok(())
//! # }
//! ```

// Removed: use squirrel_mcp_config::get_service_endpoints;

// HTTP-based auth module (OPTIONAL - for HTTP authentication)
// Production uses capability-based auth via Unix sockets!
#[cfg(feature = "http-auth")]
pub mod auth;

pub mod errors;
pub mod session;
pub mod types;

// TRUE ecoBin: Delegated JWT client (Production mode, capability-based)
pub mod delegated_jwt_client;

// JWT implementations (feature-gated for TRUE ecoBin!)
// - Production (delegated-jwt): Capability-based crypto (TRUE PRIMAL! 🌍)
// - Dev/Testing (local-jwt): Local HMAC (brings ring)
#[cfg(feature = "delegated-jwt")]
pub mod capability_crypto;
#[cfg(feature = "delegated-jwt")]
pub mod capability_jwt;

// Capability-based modules (security provider = discovered at runtime)
#[cfg(feature = "delegated-jwt")]
pub mod security_provider_client;

#[cfg(feature = "delegated-jwt")]
pub mod ecosystem_jwt;

/// Deprecated alias for security provider client (use `security_provider_client` instead).
#[deprecated(
    since = "0.1.0",
    note = "Use security_provider_client instead. Auth discovers security provider via capability, not by name."
)]
#[cfg(feature = "delegated-jwt")]
pub mod beardog_client {
    pub use super::security_provider_client::*;
}

/// Deprecated alias for ecosystem JWT (use `ecosystem_jwt` instead).
#[deprecated(
    since = "0.1.0",
    note = "Use ecosystem_jwt instead. JWT uses capability-discovered crypto provider."
)]
#[cfg(feature = "delegated-jwt")]
pub mod beardog_jwt {
    pub use super::ecosystem_jwt::*;
}

#[cfg(feature = "local-jwt")]
pub mod jwt;

// Modern re-exports leveraging capability-based patterns
#[cfg(feature = "http-auth")]
pub use auth::AuthService;
pub use delegated_jwt_client::DelegatedJwtClient;
pub use errors::{AuthError, AuthResult};
pub use session::{Session, SessionManager};
pub use types::{AuthContext, JwtClaims, LoginRequest, LoginResponse, Permission, User};

// TRUE ecoBin: Capability-based crypto & JWT (Production - Pure Rust!)
#[cfg(feature = "delegated-jwt")]
pub use capability_crypto::{CapabilityCryptoConfig, CapabilityCryptoProvider};
#[cfg(feature = "delegated-jwt")]
pub use capability_jwt::{
    CapabilityJwtConfig, CapabilityJwtService, JwtClaims as CapabilityJwtClaims,
};

// Primary exports from capability-based modules
#[cfg(feature = "delegated-jwt")]
pub use ecosystem_jwt::{BearDogJwtConfig, BearDogJwtService, JwtClaims as BearDogJwtClaims};
#[cfg(feature = "delegated-jwt")]
pub use security_provider_client::{BearDogClient, BearDogClientConfig};

// Dev/Testing: Local JWT (brings ring)
#[cfg(feature = "local-jwt")]
pub use jwt::JwtTokenManager;

/// Initialize the authentication system with current configuration
///
/// Multi-tier endpoint resolution:
/// - Security: `SECURITY_SERVICE_ENDPOINT` → `SECURITY_AUTHENTICATION_PORT` → 8443
/// - MCP: `MCP_ENDPOINT` → `MCP_PORT` → 8444
///
/// # Errors
///
/// Returns [`AuthError`] if initialization fails.
pub fn initialize() -> AuthResult<()> {
    // Multi-tier security endpoint resolution
    let security_endpoint = std::env::var("SECURITY_SERVICE_ENDPOINT").unwrap_or_else(|_| {
        let port = std::env::var("SECURITY_AUTHENTICATION_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8443); // Default security auth port
        format!("http://localhost:{port}")
    });

    // Multi-tier MCP endpoint resolution
    let mcp_endpoint = std::env::var("MCP_ENDPOINT").unwrap_or_else(|_| {
        let port = std::env::var("MCP_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8444); // Default MCP HTTP port
        format!("http://127.0.0.1:{port}")
    });

    #[cfg(feature = "delegated-jwt")]
    tracing::info!(
        "Initializing modern auth system (TRUE ecoBin mode - JWT delegated via capability discovery)"
    );

    #[cfg(feature = "local-jwt")]
    tracing::info!("Initializing modern auth system (Dev mode - local JWT validation)");

    tracing::info!(
        "Endpoints: security_service={}, mcp={}",
        security_endpoint,
        mcp_endpoint
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_returns_ok() {
        let result = initialize();
        assert!(result.is_ok());
    }

    #[test]
    fn initialize_with_security_endpoint_env() {
        let result = temp_env::with_var(
            "SECURITY_SERVICE_ENDPOINT",
            Some("http://custom:9000"),
            initialize,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn initialize_with_security_port_env() {
        let result = temp_env::with_var("SECURITY_AUTHENTICATION_PORT", Some("9999"), initialize);
        assert!(result.is_ok());
    }

    #[test]
    fn initialize_with_mcp_endpoint_env() {
        let result = temp_env::with_var("MCP_ENDPOINT", Some("http://mcp:9998"), initialize);
        assert!(result.is_ok());
    }

    #[test]
    fn initialize_with_mcp_port_env() {
        let result = temp_env::with_var("MCP_PORT", Some("8888"), initialize);
        assert!(result.is_ok());
    }
}
