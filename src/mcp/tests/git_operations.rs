use std::path::PathBuf;
use tokio;
use tempfile::tempdir;

use crate::mcp::{
    GitManager, GitConfig, GitOperation, GitStatus, GitError,
    MCPError, error::ProtocolError,
};

#[tokio::test]
async fn test_git_initialization() {
    let temp_dir = tempdir().unwrap();
    let git_manager = GitManager::new(GitConfig {
        repo_path: temp_dir.path().to_path_buf(),
        branch: "main".to_string(),
        remote: None,
        credentials: None,
    });

    let result = git_manager.initialize().await;
    assert!(result.is_ok(), "Git initialization failed");
}

#[tokio::test]
async fn test_git_operations() {
    let temp_dir = tempdir().unwrap();
    let git_manager = GitManager::new(GitConfig {
        repo_path: temp_dir.path().to_path_buf(),
        branch: "main".to_string(),
        remote: None,
        credentials: None,
    });

    // Initialize git repository
    git_manager.initialize().await.unwrap();

    // Test file creation and commit
    let test_file = temp_dir.path().join("test.txt");
    std::fs::write(&test_file, "test content").unwrap();

    let result = git_manager.add_file(&test_file).await;
    assert!(result.is_ok(), "Failed to add file");

    let result = git_manager.commit("Initial commit").await;
    assert!(result.is_ok(), "Failed to commit changes");

    // Test branch operations
    let result = git_manager.create_branch("feature/test").await;
    assert!(result.is_ok(), "Failed to create branch");

    let result = git_manager.switch_branch("feature/test").await;
    assert!(result.is_ok(), "Failed to switch branch");

    // Test status
    let status = git_manager.get_status().await;
    assert!(status.is_ok(), "Failed to get status");
    let status = status.unwrap();
    assert_eq!(status, GitStatus::Clean, "Repository should be clean");
}

#[tokio::test]
async fn test_git_error_handling() {
    let temp_dir = tempdir().unwrap();
    let git_manager = GitManager::new(GitConfig {
        repo_path: temp_dir.path().to_path_buf(),
        branch: "main".to_string(),
        remote: None,
        credentials: None,
    });

    // Test invalid operation
    let result = git_manager.execute_operation(GitOperation::Commit {
        message: "".to_string(),
        files: vec![],
    }).await;
    assert!(result.is_err(), "Empty commit should fail");

    // Test invalid branch
    let result = git_manager.switch_branch("nonexistent").await;
    assert!(result.is_err(), "Switching to nonexistent branch should fail");

    // Test invalid file
    let result = git_manager.add_file(&PathBuf::from("nonexistent.txt")).await;
    assert!(result.is_err(), "Adding nonexistent file should fail");
}

#[tokio::test]
async fn test_git_merge_operations() {
    let temp_dir = tempdir().unwrap();
    let git_manager = GitManager::new(GitConfig {
        repo_path: temp_dir.path().to_path_buf(),
        branch: "main".to_string(),
        remote: None,
        credentials: None,
    });

    // Initialize git repository
    git_manager.initialize().await.unwrap();

    // Create and commit initial content
    let main_file = temp_dir.path().join("main.txt");
    std::fs::write(&main_file, "main content").unwrap();
    git_manager.add_file(&main_file).await.unwrap();
    git_manager.commit("Initial commit").await.unwrap();

    // Create feature branch
    git_manager.create_branch("feature/test").await.unwrap();
    git_manager.switch_branch("feature/test").await.unwrap();

    // Modify content in feature branch
    std::fs::write(&main_file, "feature content").unwrap();
    git_manager.add_file(&main_file).await.unwrap();
    git_manager.commit("Feature commit").await.unwrap();

    // Switch back to main and merge
    git_manager.switch_branch("main").await.unwrap();
    let result = git_manager.merge_branch("feature/test").await;
    assert!(result.is_ok(), "Failed to merge feature branch");

    // Verify merged content
    let content = std::fs::read_to_string(&main_file).unwrap();
    assert_eq!(content, "feature content", "Content should be updated after merge");
}

#[tokio::test]
async fn test_git_remote_operations() {
    let temp_dir = tempdir().unwrap();
    let git_manager = GitManager::new(GitConfig {
        repo_path: temp_dir.path().to_path_buf(),
        branch: "main".to_string(),
        remote: Some("https://github.com/test/repo.git".to_string()),
        credentials: None,
    });

    // Initialize git repository
    git_manager.initialize().await.unwrap();

    // Test remote operations
    let result = git_manager.fetch_remote().await;
    assert!(result.is_err(), "Fetch should fail without valid remote");

    let result = git_manager.push_changes().await;
    assert!(result.is_err(), "Push should fail without valid remote");

    // Test with invalid remote URL
    let result = git_manager.set_remote("invalid-url").await;
    assert!(result.is_err(), "Setting invalid remote should fail");
} 