//! GitHub API models
//!
//! This module contains data structures that represent GitHub API resources.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// GitHub user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID
    pub id: u64,
    /// GitHub login username
    pub login: String,
    /// Display name
    #[serde(default)]
    pub name: Option<String>,
    /// Email address (if public)
    #[serde(default)]
    pub email: Option<String>,
    /// User bio
    #[serde(default)]
    pub bio: Option<String>,
    /// User location
    #[serde(default)]
    pub location: Option<String>,
    /// User avatar URL
    pub avatar_url: String,
    /// User GitHub profile URL
    pub html_url: String,
    /// Number of public repositories
    pub public_repos: u32,
    /// Number of public gists
    pub public_gists: u32,
    /// Number of followers
    pub followers: u32,
    /// Number of users followed
    pub following: u32,
    /// User creation date
    pub created_at: String,
    /// User profile update date
    pub updated_at: String,
}

/// GitHub repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Repository ID
    pub id: u64,
    /// Repository name
    pub name: String,
    /// Full repository name (owner/repo)
    pub full_name: String,
    /// Repository description
    #[serde(default)]
    pub description: Option<String>,
    /// Whether the repository is private
    pub private: bool,
    /// Repository owner
    pub owner: User,
    /// Repository HTML URL
    pub html_url: String,
    /// Repository API URL
    pub url: String,
    /// Default branch
    #[serde(default = "default_branch")]
    pub default_branch: String,
    /// Repository creation date
    pub created_at: String,
    /// Repository update date
    pub updated_at: String,
    /// Repository stars count
    pub stargazers_count: u32,
    /// Repository forks count
    pub forks_count: u32,
    /// Open issues count
    pub open_issues_count: u32,
    /// Whether the repository is a fork
    pub fork: bool,
    /// Repository size in KB
    pub size: u32,
    /// Repository language
    #[serde(default)]
    pub language: Option<String>,
    /// Whether the repository is archived
    #[serde(default)]
    pub archived: bool,
    /// Whether the repository is disabled
    #[serde(default)]
    pub disabled: bool,
}

/// Request to create a new repository
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateRepoRequest {
    /// Repository name
    pub name: String,
    /// Repository description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the repository is private
    #[serde(default)]
    pub private: bool,
    /// Whether to auto-initialize with a README
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_init: Option<bool>,
    /// Git ignore template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gitignore_template: Option<String>,
    /// License template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_template: Option<String>,
    /// Whether to allow issues
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_issues: Option<bool>,
    /// Whether to allow projects
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_projects: Option<bool>,
    /// Whether to allow wiki
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_wiki: Option<bool>,
    /// Whether to allow discussions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_discussions: Option<bool>,
}

/// Returns the default branch name for repositories
fn default_branch() -> String {
    "main".to_string()
}

/// A GitHub issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Issue ID
    pub id: u64,
    /// Issue number
    pub number: u64,
    /// Issue title
    pub title: String,
    /// Issue body
    pub body: Option<String>,
    /// Issue state
    pub state: String,
    /// Issue creator
    pub user: User,
    /// Issue assignees
    pub assignees: Vec<User>,
    /// Issue labels
    pub labels: Vec<Label>,
    /// URL to issue's GitHub page
    pub html_url: String,
    /// Creation time
    pub created_at: String,
    /// Last updated time
    pub updated_at: String,
    /// Closed time
    pub closed_at: Option<String>,
}

/// Input for creating a new issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewIssue {
    /// Issue title
    pub title: String,
    /// Issue body
    pub body: Option<String>,
    /// Issue assignees
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<String>>,
    /// Issue labels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
}

/// A GitHub pull request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// PR ID
    pub id: u64,
    /// PR number
    pub number: u64,
    /// PR title
    pub title: String,
    /// PR body
    pub body: Option<String>,
    /// PR state
    pub state: String,
    /// PR creator
    pub user: User,
    /// URL to pull request's GitHub page
    pub html_url: String,
    /// Base branch
    pub base: PullRequestBranch,
    /// Head branch
    pub head: PullRequestBranch,
    /// Whether the PR is merged
    pub merged: Option<bool>,
    /// Whether the PR is mergeable
    pub mergeable: Option<bool>,
    /// Whether there are merge conflicts
    pub mergeable_state: Option<String>,
    /// Creation time
    pub created_at: String,
    /// Last updated time
    pub updated_at: String,
    /// Closed time
    pub closed_at: Option<String>,
    /// Merged time
    pub merged_at: Option<String>,
}

/// A branch in a pull request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestBranch {
    /// Branch label
    pub label: String,
    /// Branch reference
    pub ref_field: String,
    /// Repository that contains the branch
    pub repo: Repository,
    /// Branch SHA
    pub sha: String,
    /// User that owns the branch
    pub user: User,
}

/// Input for creating a new pull request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPullRequest {
    /// PR title
    pub title: String,
    /// PR body
    pub body: Option<String>,
    /// Head branch
    pub head: String,
    /// Base branch
    pub base: String,
    /// Whether the PR is a draft
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
}

/// A GitHub label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    /// Label ID
    pub id: u64,
    /// Label name
    pub name: String,
    /// Label description
    pub description: Option<String>,
    /// Label color
    pub color: String,
    /// Whether the label is a default label
    pub default: bool,
}

/// A GitHub commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    /// Commit SHA
    pub sha: String,
    /// Commit author
    pub author: CommitAuthor,
    /// Commit committer
    pub committer: CommitAuthor,
    /// Commit message
    pub message: String,
    /// Commit tree
    pub tree: CommitTree,
    /// Commit parents
    pub parents: Vec<CommitParent>,
    /// URL to commit's GitHub page
    pub html_url: String,
}

/// A commit author or committer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAuthor {
    /// Name
    pub name: String,
    /// Email
    pub email: String,
    /// Date
    pub date: String,
}

/// A commit tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitTree {
    /// Tree SHA
    pub sha: String,
    /// Tree URL
    pub url: String,
}

/// A commit parent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitParent {
    /// Parent SHA
    pub sha: String,
    /// Parent URL
    pub url: String,
    /// Parent HTML URL
    pub html_url: String,
}

/// GitHub user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// GitHub repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub private: bool,
    pub owner: GitHubUser,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// GitHub issue information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub user: GitHubUser,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// GitHub pull request information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPullRequest {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub user: GitHubUser,
    pub created_at: String,
    pub updated_at: String,
    pub merged_at: Option<String>,
    pub head: GitHubPRRef,
    pub base: GitHubPRRef,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// GitHub pull request reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPRRef {
    pub label: String,
    pub ref_name: String,
    pub sha: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Request body for creating a new issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueRequest {
    pub title: String,
    pub body: Option<String>,
    pub assignees: Option<Vec<String>>,
    pub labels: Option<Vec<String>>,
}

/// Request body for creating a new pull request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestRequest {
    pub title: String,
    pub body: Option<String>,
    pub head: String,
    pub base: String,
    pub draft: Option<bool>,
} 