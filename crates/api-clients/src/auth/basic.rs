//! Basic authentication for API clients

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::RequestBuilder;
use reqwest::header::AUTHORIZATION;
use secrecy::{ExposeSecret, Secret};

use crate::Result;
use super::Authenticator;

/// Basic authentication provider
#[derive(Debug, Clone)]
pub struct BasicAuthenticator {
    /// Username for basic authentication
    username: Secret<String>,
    /// Password for basic authentication
    password: Secret<String>,
}

impl BasicAuthenticator {
    /// Create a new basic authenticator
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: Secret::new(username.into()),
            password: Secret::new(password.into()),
        }
    }
}

#[async_trait]
impl Authenticator for BasicAuthenticator {
    async fn authenticate(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        let auth = format!(
            "Basic {}",
            STANDARD.encode(format!(
                "{}:{}",
                self.username.expose_secret(),
                self.password.expose_secret()
            ))
        );
        Ok(request.header(AUTHORIZATION, auth))
    }
} 