//! Basic authentication for API clients

use base64::{engine::general_purpose, Engine as _};
use reqwest::RequestBuilder;
use secrecy::{Secret, SecretString};

use super::Authenticator;

/// Basic authentication implementation
pub struct BasicAuth {
    /// Username for basic authentication
    username: String,
    /// Password for basic authentication
    password: SecretString,
}

impl BasicAuth {
    /// Create a new basic authenticator
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: Secret::new(password.into()),
        }
    }
}

impl Authenticator for BasicAuth {
    fn authenticate(&self, request: RequestBuilder) -> RequestBuilder {
        let auth_value = format!(
            "Basic {}",
            general_purpose::STANDARD.encode(format!(
                "{}:{}",
                self.username,
                self.password.expose_secret()
            ))
        );
        
        request.header(reqwest::header::AUTHORIZATION, auth_value)
    }
} 