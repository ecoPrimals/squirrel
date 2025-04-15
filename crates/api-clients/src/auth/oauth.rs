//! OAuth2 authentication for API clients

use async_trait::async_trait;
use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::Result;
use super::Authenticator;

/// OAuth2 token type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenType {
    Bearer,
    Basic,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Bearer => write!(f, "Bearer"),
            TokenType::Basic => write!(f, "Basic"),
        }
    }
}

/// OAuth2 token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    access_token: String,
    token_type: TokenType,
    expires_in: Option<u64>,
    refresh_token: Option<String>,
    scope: Option<String>,
}

impl OAuthTokenResponse {
    /// Get the access token
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    /// Get the token type
    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    /// Get the expiration time in seconds
    pub fn expires_in(&self) -> Option<u64> {
        self.expires_in
    }

    /// Get the refresh token
    pub fn refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_deref()
    }

    /// Get the scope
    pub fn scope(&self) -> Option<&str> {
        self.scope.as_deref()
    }
}

/// OAuth2 authenticator
#[derive(Debug, Clone)]
pub struct OAuthAuthenticator {
    token: String,
    token_type: TokenType,
}

impl OAuthAuthenticator {
    /// Create a new OAuth2 authenticator
    pub fn new(token_result: OAuthTokenResponse) -> Self {
        Self {
            token: token_result.access_token().to_string(),
            token_type: token_result.token_type().clone(),
        }
    }
}

#[async_trait]
impl Authenticator for OAuthAuthenticator {
    async fn authenticate(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        Ok(request.header(
            "Authorization",
            format!("{} {}", self.token_type, self.token),
        ))
    }
} 