//! Squirrel Authentication & Security System
//!
//! Modern authentication system leveraging capability-based discovery and ecosystem integration.
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
//! Instead of hardcoding "BearDog", we discover capabilities:
//! - Squirrel asks: "Who provides jwt.validate capability?"
//! - Currently: BearDog (Security & Crypto Primal) provides it
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
//! ```rust
//! use squirrel_mcp_auth::{AuthService, AuthResult, LoginRequest};
//!
//! # async fn example() -> AuthResult<()> {
//! let auth_service = AuthService::new().await?;
//! let result = auth_service.authenticate(LoginRequest::new("user", "pass")).await?;
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

// DEPRECATED: BearDog-specific modules (use capability_ instead!)
#[deprecated(
    note = "Use capability_crypto instead. BearDog is DEV knowledge - TRUE PRIMAL uses capability discovery!"
)]
#[cfg(feature = "delegated-jwt")]
pub mod beardog_client;

#[deprecated(
    note = "Use capability_jwt instead. BearDog is DEV knowledge - TRUE PRIMAL uses capability discovery!"
)]
#[cfg(feature = "delegated-jwt")]
pub mod beardog_jwt;

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
pub use capability_crypto::{CapabilityCryptoProvider, CapabilityCryptoConfig};
#[cfg(feature = "delegated-jwt")]
pub use capability_jwt::{
    CapabilityJwtConfig, CapabilityJwtService, JwtClaims as CapabilityJwtClaims,
};

// DEPRECATED: BearDog-specific exports (use capability_ instead!)
#[allow(deprecated)]
#[cfg(feature = "delegated-jwt")]
pub use beardog_client::{BearDogClient, BearDogClientConfig};
#[allow(deprecated)]
#[cfg(feature = "delegated-jwt")]
pub use beardog_jwt::{BearDogJwtConfig, BearDogJwtService, JwtClaims as BearDogJwtClaims};

// Dev/Testing: Local JWT (brings ring)
#[cfg(feature = "local-jwt")]
pub use jwt::JwtTokenManager;

/// Initialize the authentication system with current configuration
///
/// Multi-tier endpoint resolution:
/// - Security: SECURITY_SERVICE_ENDPOINT → SECURITY_AUTHENTICATION_PORT → 8443
/// - MCP: MCP_ENDPOINT → MCP_PORT → 8444
pub async fn initialize() -> AuthResult<()> {
    // Multi-tier security endpoint resolution
    let security_endpoint = std::env::var("SECURITY_SERVICE_ENDPOINT").unwrap_or_else(|_| {
        let port = std::env::var("SECURITY_AUTHENTICATION_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8443);  // Default security auth port
        format!("http://localhost:{}", port)
    });

    // Multi-tier MCP endpoint resolution
    let mcp_endpoint = std::env::var("MCP_ENDPOINT").unwrap_or_else(|_| {
        let port = std::env::var("MCP_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8444);  // Default MCP HTTP port
        format!("http://127.0.0.1:{}", port)
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
