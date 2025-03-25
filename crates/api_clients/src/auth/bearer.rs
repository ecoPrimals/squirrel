//! Bearer token authentication for API clients

use reqwest::RequestBuilder;
use secrecy::{Secret, SecretString};

use super::Authenticator;

/// Bearer token authentication implementation
pub struct BearerAuth {
    /// Bearer token for authentication
    token: SecretString,
}

impl BearerAuth {
    /// Create a new bearer token authenticator
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: Secret::new(token.into()),
        }
    }
}

impl Authenticator for BearerAuth {
    fn authenticate(&self, request: RequestBuilder) -> RequestBuilder {
        let auth_value = format!("Bearer {}", self.token.expose_secret());
        
        request.header(reqwest::header::AUTHORIZATION, auth_value)
    }
} 