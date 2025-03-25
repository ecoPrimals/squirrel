//! OAuth2 authentication for API clients

use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, RefreshToken, TokenResponse,
    TokenUrl,
};
use reqwest::RequestBuilder;
use secrecy::{Secret, SecretString};
use std::sync::{Arc, Mutex};

use crate::Error;
use super::Authenticator;

/// OAuth2 authentication configuration
#[derive(Debug, Clone)]
pub struct OAuth2Config {
    /// Client ID for OAuth2
    pub client_id: String,
    /// Client secret for OAuth2
    pub client_secret: String,
    /// Authorization URL
    pub auth_url: String,
    /// Token URL
    pub token_url: String,
    /// Redirect URL
    pub redirect_url: Option<String>,
    /// Initial access token (if available)
    pub access_token: Option<String>,
    /// Initial refresh token (if available)
    pub refresh_token: Option<String>,
}

/// OAuth2 token information
#[derive(Debug, Clone)]
struct TokenInfo {
    /// Access token
    access_token: SecretString,
    /// Refresh token
    refresh_token: Option<SecretString>,
    /// Expiration time in seconds from creation
    expires_in: Option<u64>,
    /// When the token was created
    created_at: std::time::Instant,
}

/// OAuth2 authentication implementation
pub struct OAuth2Auth {
    /// OAuth2 client
    client: BasicClient,
    /// Current token information
    token: Arc<Mutex<Option<TokenInfo>>>,
}

impl OAuth2Auth {
    /// Create a new OAuth2 authenticator
    pub fn new(config: OAuth2Config) -> Self {
        let client = BasicClient::new(
            ClientId::new(config.client_id),
            Some(ClientSecret::new(config.client_secret)),
            AuthUrl::new(config.auth_url).unwrap(),
            Some(TokenUrl::new(config.token_url).unwrap()),
        );

        let client = if let Some(redirect_url) = config.redirect_url {
            client.set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
        } else {
            client
        };

        let token = if let Some(access_token) = config.access_token {
            let token_info = TokenInfo {
                access_token: Secret::new(access_token),
                refresh_token: config.refresh_token.map(Secret::new),
                expires_in: None,
                created_at: std::time::Instant::now(),
            };
            Some(token_info)
        } else {
            None
        };

        Self {
            client,
            token: Arc::new(Mutex::new(token)),
        }
    }

    /// Refresh the access token using the refresh token
    async fn refresh_token(&self) -> Result<(), Error> {
        let refresh_token = {
            let token_guard = self.token.lock().unwrap();
            if let Some(token) = &*token_guard {
                if let Some(refresh_token) = &token.refresh_token {
                    RefreshToken::new(refresh_token.expose_secret().clone())
                } else {
                    return Err(Error::Authentication(
                        "No refresh token available".to_string(),
                    ));
                }
            } else {
                return Err(Error::Authentication(
                    "No token information available".to_string(),
                ));
            }
        };

        let token_result = self
            .client
            .exchange_refresh_token(&refresh_token)
            .request_async(
                oauth2::reqwest::async_http_client,
            )
            .await
            .map_err(|e| Error::OAuth(e.to_string()))?;

        let access_token = token_result.access_token().secret().clone();
        let refresh_token = token_result.refresh_token().map(|rt| rt.secret().clone());
        let expires_in = token_result.expires_in();

        let token_info = TokenInfo {
            access_token: Secret::new(access_token),
            refresh_token: refresh_token.map(Secret::new),
            expires_in,
            created_at: std::time::Instant::now(),
        };

        let mut token_guard = self.token.lock().unwrap();
        *token_guard = Some(token_info);

        Ok(())
    }

    /// Check if the token is expired and needs to be refreshed
    fn is_token_expired(&self) -> bool {
        let token_guard = self.token.lock().unwrap();
        if let Some(token) = &*token_guard {
            if let Some(expires_in) = token.expires_in {
                let elapsed = token.created_at.elapsed().as_secs();
                // Refresh if within 60 seconds of expiration
                return elapsed + 60 >= expires_in;
            }
        }
        false
    }
}

impl Authenticator for OAuth2Auth {
    fn authenticate(&self, request: RequestBuilder) -> RequestBuilder {
        // Check if the token is expired and needs to be refreshed
        // In a real implementation, we would need to handle this asynchronously
        // For now, we just use the current token

        let token_guard = self.token.lock().unwrap();
        if let Some(token) = &*token_guard {
            let auth_value = format!("Bearer {}", token.access_token.expose_secret());
            request.header(reqwest::header::AUTHORIZATION, auth_value)
        } else {
            // No token available, cannot authenticate
            request
        }
    }
} 