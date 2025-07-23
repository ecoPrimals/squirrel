//! Squirrel Authentication & Security System
//!
//! Modern authentication system leveraging beardog patterns and squirrel ecosystem integration.
//! This completely replaces the severely outdated legacy auth code with a clean, modern architecture.
//!
//! ## Features
//!
//! - **Clean Error Handling**: Using thiserror with detailed error context
//! - **Beardog Integration**: Leveraging modern beardog security patterns  
//! - **Ecosystem Integration**: Deep integration with squirrel MCP configuration
//! - **Modern Rust Patterns**: No anyhow conflicts, clean Result types
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

use squirrel_mcp_config::get_service_endpoints;

pub mod auth;
pub mod errors;
pub mod session;
pub mod types;

// Modern re-exports leveraging beardog patterns
pub use auth::AuthService;
pub use errors::{AuthError, AuthResult};
pub use session::{Session, SessionManager};
pub use types::{AuthContext, LoginRequest, LoginResponse, Permission, User};

/// Initialize the authentication system with current configuration
pub async fn initialize() -> AuthResult<()> {
    let endpoints = get_service_endpoints();
    tracing::info!(
        "Initializing modern auth system with endpoints: security_service={}, mcp={}",
        endpoints.security_service_endpoint,
        endpoints.mcp_endpoint
    );
    Ok(())
}
