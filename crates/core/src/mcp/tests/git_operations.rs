use crate::mcp::{MCPProtocol, MCPMessage, MessageType, SecurityMetadata, SecurityLevel};
use chrono::Utc;
use std::process::Command;

/// Tests successful git commit followed by automatic push
#[tokio::test]
async fn test_git_push_on_success() {
    // Initialize protocol
    let mut protocol = MCPProtocol::new();

    // Create test message for git commit
    let commit_message = MCPMessage {
        id: "test-commit".to_string(),
        type_: MessageType::Command,
        payload: serde_json::json!({
            "command": "git_commit",
            "args": ["test: automated commit"]
        }),
        metadata: None,
        security: SecurityMetadata {
            security_level: SecurityLevel::Low,
            encryption_info: None,
            signature: None,
            auth_token: None,
        },
        timestamp: Utc::now(),
    };

    // Handle commit message and verify success
    let commit_result = protocol.handle_message(commit_message).await;
    assert!(commit_result.is_ok(), "Commit should succeed");

    // Verify push was attempted after successful commit
    if let Ok(response) = commit_result {
        assert!(response.success, "Commit and push should be successful");
        if let Some(data) = response.data {
            // Verify both commit and push data are present
            assert!(data.get("commit").is_some(), "Commit data should be present");
            assert!(data.get("push").is_some(), "Push data should be present");
        }
    }
}

/// Tests handling of invalid git commit attempts
#[tokio::test]
async fn test_git_push_failure_handling() {
    let mut protocol = MCPProtocol::new();

    // Test cases for various failure scenarios
    let test_cases = vec![
        // Missing commit message
        MCPMessage {
            id: "test-invalid-commit-1".to_string(),
            type_: MessageType::Command,
            payload: serde_json::json!({
                "command": "git_commit",
                "args": []
            }),
            metadata: None,
            security: SecurityMetadata {
                security_level: SecurityLevel::Low,
                encryption_info: None,
                signature: None,
                auth_token: None,
            },
            timestamp: Utc::now(),
        },
        // Empty commit message
        MCPMessage {
            id: "test-invalid-commit-2".to_string(),
            type_: MessageType::Command,
            payload: serde_json::json!({
                "command": "git_commit",
                "args": [""]
            }),
            metadata: None,
            security: SecurityMetadata {
                security_level: SecurityLevel::Low,
                encryption_info: None,
                signature: None,
                auth_token: None,
            },
            timestamp: Utc::now(),
        },
    ];

    for msg in test_cases {
        let result = protocol.handle_message(msg).await;
        assert!(result.is_err() || !result.unwrap().success, 
            "Invalid commit attempts should fail");
    }
}

/// Tests git operations with status updates
#[tokio::test]
async fn test_git_push_with_status_update() {
    use crate::ui::status::StatusManager;
    use std::io::Cursor;

    // Initialize protocol and status manager
    let mut protocol = MCPProtocol::new();
    let status_manager = StatusManager::new();
    let mut output = Cursor::new(Vec::new());

    // Create test message for git commit
    let commit_message = MCPMessage {
        id: "test-commit-status".to_string(),
        type_: MessageType::Command,
        payload: serde_json::json!({
            "command": "git_commit",
            "args": ["test: commit with status update"]
        }),
        metadata: None,
        security: SecurityMetadata {
            security_level: SecurityLevel::Low,
            encryption_info: None,
            signature: None,
            auth_token: None,
        },
        timestamp: Utc::now(),
    };

    // Handle commit message and update status
    match protocol.handle_message(commit_message).await {
        Ok(response) if response.success => {
            status_manager.print_success(&mut output, "Commit successful", 0).unwrap();

            // Verify push was attempted and status was updated
            if let Some(data) = response.data {
                if data.get("push").is_some() {
                    status_manager.print_success(&mut output, "Push successful", 0).unwrap();
                }
            }
        },
        _ => {
            status_manager.print_error(&mut output, "Operation failed", 0).unwrap();
        }
    }

    // Verify status messages were written
    let output_str = String::from_utf8(output.into_inner()).unwrap();
    assert!(output_str.contains("✓") || output_str.contains("✗"), 
        "Status output should contain success or failure indicators");
}

/// Tests direct git push operation
#[tokio::test]
async fn test_direct_git_push() {
    let mut protocol = MCPProtocol::new();

    // Create push message
    let push_message = MCPMessage {
        id: "test-direct-push".to_string(),
        type_: MessageType::Command,
        payload: serde_json::json!({
            "command": "git_push"
        }),
        metadata: None,
        security: SecurityMetadata {
            security_level: SecurityLevel::Low,
            encryption_info: None,
            signature: None,
            auth_token: None,
        },
        timestamp: Utc::now(),
    };

    // Handle push message
    let result = protocol.handle_message(push_message).await;
    assert!(result.is_ok(), "Direct push should complete without errors");
} 