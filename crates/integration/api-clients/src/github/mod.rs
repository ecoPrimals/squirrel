//! GitHub API client implementation

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::auth::BearerAuthenticator;
use crate::http::{DefaultHttpClient, HttpClientExt};
use crate::Result;

/// GitHub API client
#[derive(Clone)]
pub struct GitHubClient {
    http_client: DefaultHttpClient,
}

impl GitHubClient {
    /// Create a new GitHub API client
    pub fn new(token: impl Into<String>) -> Self {
        let auth = Arc::new(BearerAuthenticator::new(token.into()));
        let http_client = DefaultHttpClient::new()
            .with_base_url("https://api.github.com")
            .with_authenticator(auth);

        Self { http_client }
    }

    /// Get the authenticated user
    pub async fn get_user(&self) -> Result<User> {
        self.http_client.get_json("/user").await
    }

    /// List repositories for the authenticated user
    pub async fn list_repos(&self) -> Result<Vec<Repository>> {
        self.http_client.get_json("/user/repos").await
    }

    /// Get a repository by owner and name
    pub async fn get_repo(&self, owner: &str, name: &str) -> Result<Repository> {
        self.http_client
            .get_json(&format!("/repos/{owner}/{name}"))
            .await
    }

    /// Create a new repository
    pub async fn create_repo(&self, request: CreateRepoRequest) -> Result<Repository> {
        self.http_client.post_json("/user/repos", &request).await
    }
}

/// Represents a GitHub user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// The user's login name
    pub login: String,
    /// The user's unique identifier
    pub id: u64,
    /// The user's display name, if set
    pub name: Option<String>,
    /// The user's email address, if public
    pub email: Option<String>,
    /// URL to the user's avatar image
    pub avatar_url: Option<String>,
}

/// Represents a GitHub repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// The repository's unique identifier
    pub id: u64,
    /// The repository's name
    pub name: String,
    /// The repository's full name (owner/name)
    pub full_name: String,
    /// Whether the repository is private
    pub private: bool,
    /// The repository's description, if any
    pub description: Option<String>,
    /// The repository's HTML URL
    pub html_url: String,
    /// The repository's SSH URL for cloning
    pub ssh_url: String,
    /// The repository's HTTPS URL for cloning
    pub clone_url: String,
}

/// Parameters for creating a new repository
#[derive(Debug, Clone, Serialize)]
pub struct CreateRepoRequest {
    /// The name of the repository
    pub name: String,
    /// The repository's description
    pub description: Option<String>,
    /// Whether the repository should be private
    pub private: bool,
    /// Whether to initialize the repository with a README
    pub auto_init: bool,
}
