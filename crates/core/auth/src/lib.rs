// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Authentication mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

#![expect(
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
//! - **Feature-Gated JWT**: Production uses delegated capability JWT (`delegated-jwt`)
//!
//! ## Usage
//!
//! ```rust,no_run
//! use squirrel_mcp_auth::{AuthResult, capability_jwt::CapabilityJwtConfig};
//!
//! # fn example() -> AuthResult<()> {
//! let config = CapabilityJwtConfig::default();
//! // Production: DelegatedJwtClient::new(config)?
//! # Ok(())
//! # }
//! ```

pub mod auth;
pub mod errors;
pub mod session;
pub mod types;

// TRUE ecoBin: Delegated JWT client (Production mode, capability-based)
pub mod delegated_jwt_client;

// JWT implementations (feature-gated for TRUE ecoBin!)
// - Production (delegated-jwt): Capability-based crypto (TRUE PRIMAL! 🌍)
#[cfg(feature = "delegated-jwt")]
pub mod capability_crypto;
#[cfg(feature = "delegated-jwt")]
pub mod capability_jwt;

// Capability-based modules (security provider = discovered at runtime)
#[cfg(feature = "delegated-jwt")]
pub mod security_provider_client;

#[cfg(feature = "delegated-jwt")]
pub mod ecosystem_jwt;


// Modern re-exports leveraging capability-based patterns
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

// Primary exports from capability-based modules (canonical capability-oriented names)
#[cfg(feature = "delegated-jwt")]
pub use ecosystem_jwt::{
    JwtClaims as SecurityProviderJwtClaims, SecurityProviderJwtConfig, SecurityProviderJwtService,
};
#[cfg(feature = "delegated-jwt")]
pub use security_provider_client::{SecurityProviderClient, SecurityProviderClientConfig};


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
    let security_endpoint = universal_constants::network::discover_peer_http_origin(
        "SECURITY_SERVICE_ENDPOINT",
        "SECURITY_SERVICE_HOST",
        "SECURITY_AUTHENTICATION_PORT",
        universal_constants::network::DEFAULT_LOCALHOST,
        universal_constants::network::DEFAULT_SECURITY_PORT,
    );

    // Multi-tier MCP endpoint resolution
    let mcp_endpoint = universal_constants::network::discover_peer_http_origin(
        "MCP_ENDPOINT",
        "MCP_HOST",
        "MCP_PORT",
        universal_constants::network::LOCALHOST_IPV4,
        8444,
    );

    #[cfg(feature = "delegated-jwt")]
    tracing::info!(
        "Initializing modern auth system (TRUE ecoBin mode - JWT delegated via capability discovery)"
    );

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
