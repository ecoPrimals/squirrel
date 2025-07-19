//! Authentication module
//!
//! Provides authentication traits and basic implementations.
//! Full authentication is handled by the BearDog security framework.

use serde::{Deserialize, Serialize};

/// Generic authenticator trait
pub trait Authenticator: Send + Sync {
    /// Authenticate a request by adding appropriate headers
    fn authenticate(&self, headers: &mut std::collections::HashMap<String, String>);
}

/// Bearer token authenticator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearerAuthenticator {
    token: String,
}

impl BearerAuthenticator {
    /// Create a new bearer authenticator
    pub fn new(token: String) -> Self {
        Self { token }
    }
}

impl Authenticator for BearerAuthenticator {
    fn authenticate(&self, headers: &mut std::collections::HashMap<String, String>) {
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.token),
        );
    }
}

/// No-op authenticator for public APIs
#[derive(Debug, Default)]
pub struct NoAuthenticator;

impl Authenticator for NoAuthenticator {
    fn authenticate(&self, _headers: &mut std::collections::HashMap<String, String>) {
        // No authentication required
    }
}
