//! GitHub API client
//!
//! This module provides a client for interacting with the GitHub API.

use std::sync::Arc;

use crate::auth::Authenticator;
use crate::http::{HttpClient, HttpClientConfig};
use crate::{Error, Result};

mod models;
mod endpoints;

pub use endpoints::*;
pub use models::*;

/// GitHub API client
pub struct GitHubClient {
    /// HTTP client for making requests
    http_client: Arc<dyn HttpClient>,
}

impl GitHubClient {
    /// Create a new GitHub client with the given authenticator
    pub fn new(auth: Arc<dyn Authenticator>) -> Self {
        let config = HttpClientConfig::new("https://api.github.com")
            .with_header("Accept", "application/vnd.github.v3+json")
            .with_rate_limit(60); // GitHub has a rate limit of 60 requests per hour for unauthenticated requests
            
        let http_client = crate::http::new_client(config, Some(auth));
        
        Self { http_client }
    }
    
    /// Create a new GitHub client with a personal access token
    pub fn with_token(token: impl Into<String>) -> Self {
        let auth = crate::auth::bearer_auth(token);
        Self::new(auth)
    }
    
    /// Create a new GitHub client with username and password
    pub fn with_basic_auth(username: impl Into<String>, password: impl Into<String>) -> Self {
        let auth = crate::auth::basic_auth(username, password);
        Self::new(auth)
    }
    
    /// Get information about the authenticated user
    pub async fn get_authenticated_user(&self) -> Result<User> {
        self.http_client.get_json("/user").await
    }
    
    /// List repositories for the authenticated user
    pub async fn list_repos(&self) -> Result<Vec<Repository>> {
        self.http_client.get_json("/user/repos").await
    }
    
    /// Get a specific repository by owner and name
    pub async fn get_repo(&self, owner: &str, repo: &str) -> Result<Repository> {
        self.http_client.get_json(&format!("/repos/{}/{}", owner, repo)).await
    }
    
    /// List issues for a repository
    pub async fn list_issues(&self, owner: &str, repo: &str) -> Result<Vec<Issue>> {
        self.http_client.get_json(&format!("/repos/{}/{}/issues", owner, repo)).await
    }
    
    /// Create a new issue in a repository
    pub async fn create_issue(&self, owner: &str, repo: &str, issue: NewIssue) -> Result<Issue> {
        self.http_client.post_json(&format!("/repos/{}/{}/issues", owner, repo), &issue).await
    }
    
    /// List pull requests for a repository
    pub async fn list_pull_requests(&self, owner: &str, repo: &str) -> Result<Vec<PullRequest>> {
        self.http_client.get_json(&format!("/repos/{}/{}/pulls", owner, repo)).await
    }
    
    /// Get a specific pull request
    pub async fn get_pull_request(&self, owner: &str, repo: &str, number: u64) -> Result<PullRequest> {
        self.http_client.get_json(&format!("/repos/{}/{}/pulls/{}", owner, repo, number)).await
    }
    
    /// Create a new pull request
    pub async fn create_pull_request(&self, owner: &str, repo: &str, pr: NewPullRequest) -> Result<PullRequest> {
        self.http_client.post_json(&format!("/repos/{}/{}/pulls", owner, repo), &pr).await
    }
} 