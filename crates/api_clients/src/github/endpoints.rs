//! GitHub API endpoints
//!
//! This module contains constants and functions for GitHub API endpoints.

/// GitHub API base URL
pub const API_BASE_URL: &str = "https://api.github.com";

/// GitHub API version header
pub const API_VERSION_HEADER: &str = "application/vnd.github.v3+json";

/// GitHub OAuth authorization URL
pub const OAUTH_AUTH_URL: &str = "https://github.com/login/oauth/authorize";

/// GitHub OAuth token URL
pub const OAUTH_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";

/// Rate limit header for requests remaining
pub const RATE_LIMIT_REMAINING_HEADER: &str = "X-RateLimit-Remaining";

/// Rate limit header for reset time
pub const RATE_LIMIT_RESET_HEADER: &str = "X-RateLimit-Reset";

/// Generate user endpoint
pub fn user_endpoint() -> String {
    "/user".to_string()
}

/// Generate user repos endpoint
pub fn user_repos_endpoint() -> String {
    "/user/repos".to_string()
}

/// Generate repo endpoint
pub fn repo_endpoint(owner: &str, repo: &str) -> String {
    format!("/repos/{}/{}", owner, repo)
}

/// Generate repo issues endpoint
pub fn repo_issues_endpoint(owner: &str, repo: &str) -> String {
    format!("/repos/{}/{}/issues", owner, repo)
}

/// Generate repo pull requests endpoint
pub fn repo_pulls_endpoint(owner: &str, repo: &str) -> String {
    format!("/repos/{}/{}/pulls", owner, repo)
}

/// Generate specific pull request endpoint
pub fn repo_pull_endpoint(owner: &str, repo: &str, number: u64) -> String {
    format!("/repos/{}/{}/pulls/{}", owner, repo, number)
}

/// Generate repo commits endpoint
pub fn repo_commits_endpoint(owner: &str, repo: &str) -> String {
    format!("/repos/{}/{}/commits", owner, repo)
}

/// Generate specific commit endpoint
pub fn repo_commit_endpoint(owner: &str, repo: &str, sha: &str) -> String {
    format!("/repos/{}/{}/commits/{}", owner, repo, sha)
}

/// Generate repo branches endpoint
pub fn repo_branches_endpoint(owner: &str, repo: &str) -> String {
    format!("/repos/{}/{}/branches", owner, repo)
}

/// Generate specific branch endpoint
pub fn repo_branch_endpoint(owner: &str, repo: &str, branch: &str) -> String {
    format!("/repos/{}/{}/branches/{}", owner, repo, branch)
}

/// Generate repo contents endpoint
pub fn repo_contents_endpoint(owner: &str, repo: &str, path: &str) -> String {
    format!("/repos/{}/{}/contents/{}", owner, repo, path)
}

/// Generate user endpoint for a specific user
pub fn specific_user_endpoint(username: &str) -> String {
    format!("/users/{}", username)
}

/// Generate user repos endpoint for a specific user
pub fn specific_user_repos_endpoint(username: &str) -> String {
    format!("/users/{}/repos", username)
}

/// Generate gists endpoint
pub fn gists_endpoint() -> String {
    "/gists".to_string()
}

/// Generate specific gist endpoint
pub fn gist_endpoint(gist_id: &str) -> String {
    format!("/gists/{}", gist_id)
} 