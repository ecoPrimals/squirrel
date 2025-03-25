//! GitHub API models
//!
//! This module contains data structures that represent GitHub API resources.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A GitHub user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID
    pub id: u64,
    /// Login name
    pub login: String,
    /// Display name
    pub name: Option<String>,
    /// Email address
    pub email: Option<String>,
    /// Avatar URL
    pub avatar_url: String,
    /// Bio
    pub bio: Option<String>,
    /// URL to user's GitHub page
    pub html_url: String,
    /// Whether the user is a site admin
    pub site_admin: bool,
}

/// A GitHub repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Repository ID
    pub id: u64,
    /// Repository name
    pub name: String,
    /// Full repository name (owner/name)
    pub full_name: String,
    /// Repository owner
    pub owner: User,
    /// Whether the repository is private
    pub private: bool,
    /// Repository description
    pub description: Option<String>,
    /// URL to repository's GitHub page
    pub html_url: String,
    /// Default branch
    pub default_branch: String,
    /// Repository language
    pub language: Option<String>,
    /// Number of forks
    pub forks_count: u64,
    /// Number of watchers
    pub watchers_count: u64,
    /// Number of stars
    pub stargazers_count: u64,
    /// Number of open issues
    pub open_issues_count: u64,
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