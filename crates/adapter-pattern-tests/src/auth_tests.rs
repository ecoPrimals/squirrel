// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::types::Command;
use std::sync::Arc;

fn test_cmd(name: &str, result: &str) -> Arc<dyn Command> {
    Arc::new(crate::types::TestCommand::new(name, "test", result))
}

#[tokio::test]
async fn test_mcp_adapter_default() {
    let adapter = McpAdapter::default();
    let cmds = adapter.list_commands().await.expect("should succeed");
    assert!(cmds.is_empty());
}

#[tokio::test]
async fn test_mcp_adapter_clone() {
    let adapter = McpAdapter::new();
    let cloned = adapter.clone();
    let a = adapter.list_commands().await.expect("should succeed");
    let b = cloned.list_commands().await.expect("should succeed");
    assert_eq!(a, b);
}

#[tokio::test]
async fn test_authenticate_user_valid() {
    let adapter = McpAdapter::new();
    adapter
        .register_command(test_cmd("hello", "hi"))
        .await
        .expect("should succeed");
    let auth = Auth::User("admin".to_string(), "password".to_string());
    let result = adapter.execute_with_auth("hello", vec![], auth).await;
    assert!(result.is_ok());
    assert_eq!(result.expect("should succeed"), "hi");
}

#[tokio::test]
async fn test_authenticate_user_invalid_password() {
    let adapter = McpAdapter::new();
    adapter
        .register_command(test_cmd("hello", "hi"))
        .await
        .expect("should succeed");
    let auth = Auth::User("admin".to_string(), "wrong".to_string());
    let result = adapter.execute_with_auth("hello", vec![], auth).await;
    assert!(result.is_err());
    assert!(matches!(result, Err(CommandError::AuthenticationFailed(_))));
}

#[tokio::test]
async fn test_authenticate_user_not_found() {
    let adapter = McpAdapter::new();
    adapter
        .register_command(test_cmd("hello", "hi"))
        .await
        .expect("should succeed");
    let auth = Auth::User("nonexistent".to_string(), "pass".to_string());
    let result = adapter.execute_with_auth("hello", vec![], auth).await;
    assert!(result.is_err());
    assert!(matches!(result, Err(CommandError::AuthenticationFailed(_))));
}

#[tokio::test]
async fn test_authenticate_token_valid() {
    let mut adapter = McpAdapter::new();
    adapter
        .register_command(test_cmd("hello", "hi"))
        .await
        .expect("should succeed");
    let token = adapter
        .generate_token("admin", "password")
        .expect("token gen");
    let auth = Auth::Token(token);
    let result = adapter.execute_with_auth("hello", vec![], auth).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_authenticate_token_invalid() {
    let adapter = McpAdapter::new();
    adapter
        .register_command(test_cmd("hello", "hi"))
        .await
        .expect("should succeed");
    let auth = Auth::Token("invalid-token".to_string());
    let result = adapter.execute_with_auth("hello", vec![], auth).await;
    assert!(result.is_err());
    assert!(matches!(result, Err(CommandError::AuthenticationFailed(_))));
}

#[tokio::test]
async fn test_authenticate_api_key_valid() {
    let adapter = McpAdapter::new();
    adapter
        .register_command(test_cmd("hello", "hi"))
        .await
        .expect("should succeed");
    let auth = Auth::ApiKey("squirrel-api-key".to_string());
    let result = adapter.execute_with_auth("hello", vec![], auth).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_authenticate_api_key_invalid() {
    let adapter = McpAdapter::new();
    adapter
        .register_command(test_cmd("hello", "hi"))
        .await
        .expect("should succeed");
    let auth = Auth::ApiKey("bad-key".to_string());
    let result = adapter.execute_with_auth("hello", vec![], auth).await;
    assert!(result.is_err());
    assert!(matches!(result, Err(CommandError::AuthenticationFailed(_))));
}

#[tokio::test]
async fn test_authenticate_none() {
    let adapter = McpAdapter::new();
    adapter
        .register_command(test_cmd("hello", "hi"))
        .await
        .expect("should succeed");
    let result = adapter.execute_with_auth("hello", vec![], Auth::None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_authorize_admin_command_without_auth() {
    let adapter = McpAdapter::new();
    adapter
        .register_command(test_cmd("admin-cmd", "admin"))
        .await
        .expect("should succeed");
    let result = adapter
        .execute_with_auth("admin-cmd", vec![], Auth::None)
        .await;
    assert!(result.is_err());
    assert!(matches!(result, Err(CommandError::AuthorizationFailed(_))));
}

#[tokio::test]
async fn test_authorize_admin_command_with_admin() {
    let adapter = McpAdapter::new();
    adapter
        .register_command(test_cmd("admin-cmd", "admin"))
        .await
        .expect("should succeed");
    let auth = Auth::User("admin".to_string(), "password".to_string());
    let result = adapter.execute_with_auth("admin-cmd", vec![], auth).await;
    assert!(result.is_ok());
    assert_eq!(result.expect("should succeed"), "admin");
}

#[tokio::test]
async fn test_add_user_regular() {
    let adapter = McpAdapter::new();
    adapter.add_user("bob", "bob123", false);
    adapter
        .register_command(test_cmd("hello", "hi"))
        .await
        .expect("should succeed");
    let auth = Auth::User("bob".to_string(), "bob123".to_string());
    let result = adapter.execute_with_auth("hello", vec![], auth).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_user_admin() {
    let adapter = McpAdapter::new();
    adapter.add_user("superadmin", "super123", true);
    adapter
        .register_command(test_cmd("admin-cmd", "admin"))
        .await
        .expect("should succeed");
    let auth = Auth::User("superadmin".to_string(), "super123".to_string());
    let result = adapter.execute_with_auth("admin-cmd", vec![], auth).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_command_with_permissions() {
    let mut adapter = McpAdapter::new();
    adapter.add_command_with_permissions("power-cmd", vec![UserRole::PowerUser]);
    adapter
        .register_command(test_cmd("power-cmd", "power"))
        .await
        .expect("should succeed");
    adapter.add_user("bob", "bob123", false);
    let auth = Auth::User("bob".to_string(), "bob123".to_string());
    let result = adapter.execute_with_auth("power-cmd", vec![], auth).await;
    assert!(result.is_err());
    assert!(matches!(result, Err(CommandError::AuthorizationFailed(_))));
    let auth = Auth::ApiKey("squirrel-api-key".to_string());
    let result = adapter.execute_with_auth("power-cmd", vec![], auth).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_generate_token_success() {
    let mut adapter = McpAdapter::new();
    let token = adapter
        .generate_token("admin", "password")
        .expect("should succeed");
    assert!(token.starts_with("token-admin-"));
}

#[tokio::test]
async fn test_generate_token_wrong_password() {
    let mut adapter = McpAdapter::new();
    let result = adapter.generate_token("admin", "wrong");
    assert!(result.is_err());
    assert!(matches!(result, Err(CommandError::AuthenticationFailed(_))));
}

#[tokio::test]
async fn test_generate_token_user_not_found() {
    let mut adapter = McpAdapter::new();
    let result = adapter.generate_token("nobody", "pass");
    assert!(result.is_err());
    assert!(matches!(result, Err(CommandError::AuthenticationFailed(_))));
}

#[tokio::test]
async fn test_command_logs() {
    let adapter = McpAdapter::new();
    adapter
        .register_command(test_cmd("hello", "hi"))
        .await
        .expect("should succeed");
    let _ = adapter.execute_with_auth("hello", vec![], Auth::None).await;
    let logs = adapter.get_command_logs();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].command, "hello");
    assert!(logs[0].success);
}

#[tokio::test]
async fn test_command_logs_failure() {
    use crate::types::Command;

    #[derive(Debug)]
    struct FailingCommand;
    impl Command for FailingCommand {
        fn name(&self) -> &'static str {
            "fail-cmd"
        }
        fn description(&self) -> &'static str {
            "Fails"
        }
        fn execute(&self, _args: Vec<String>) -> CommandResult<String> {
            Err(CommandError::ExecutionFailed(
                "intentional failure".to_string(),
            ))
        }
    }

    let adapter = McpAdapter::new();
    adapter
        .register_command(Arc::new(FailingCommand))
        .await
        .expect("should succeed");
    let _ = adapter
        .execute_with_auth("fail-cmd", vec![], Auth::None)
        .await;
    let logs = adapter.get_command_logs();
    assert_eq!(logs.len(), 1);
    assert!(!logs[0].success);
}

#[tokio::test]
async fn test_get_formatted_command_logs() {
    let adapter = McpAdapter::new();
    adapter
        .register_command(test_cmd("hello", "hi"))
        .await
        .expect("should succeed");
    let _ = adapter.execute_with_auth("hello", vec![], Auth::None).await;
    let formatted = adapter.get_formatted_command_logs();
    assert_eq!(formatted.len(), 1);
    assert!(formatted[0].contains("hello"));
    assert!(formatted[0].contains("SUCCESS"));
}

#[tokio::test]
async fn test_register_command_auto_admin_permission() {
    let adapter = McpAdapter::new();
    adapter
        .register_command(test_cmd("admin-stats", "stats"))
        .await
        .expect("should succeed");
    let result = adapter
        .execute_with_auth("admin-stats", vec![], Auth::None)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_available_commands_admin() {
    let adapter = McpAdapter::new();
    adapter
        .register_command(test_cmd("hello", "hi"))
        .await
        .expect("should succeed");
    adapter
        .register_command(test_cmd("admin-cmd", "admin"))
        .await
        .expect("should succeed");
    let auth = Auth::User("admin".to_string(), "password".to_string());
    let cmds = adapter
        .get_available_commands(auth)
        .await
        .expect("should succeed");
    assert!(cmds.contains(&"hello".to_string()));
    assert!(cmds.contains(&"admin-cmd".to_string()));
}

#[tokio::test]
async fn test_get_available_commands_anonymous() {
    let adapter = McpAdapter::new();
    adapter
        .register_command(test_cmd("hello", "hi"))
        .await
        .expect("should succeed");
    adapter
        .register_command(test_cmd("admin-cmd", "admin"))
        .await
        .expect("should succeed");
    let cmds = adapter
        .get_available_commands(Auth::None)
        .await
        .expect("should succeed");
    assert!(cmds.contains(&"hello".to_string()));
    assert!(!cmds.contains(&"admin-cmd".to_string()));
}

#[tokio::test]
async fn test_command_adapter_execute() {
    let adapter = McpAdapter::new();
    adapter
        .register_command(test_cmd("hello", "hi"))
        .await
        .expect("should succeed");
    let result = <McpAdapter as CommandAdapter>::execute(&adapter, "hello", vec![]).await;
    assert!(result.is_ok());
    assert_eq!(result.expect("should succeed"), "hi");
}

#[tokio::test]
async fn test_command_adapter_get_help() {
    let adapter = McpAdapter::new();
    let cmd = Arc::new(crate::types::TestCommand::new("hello", "Says hello", "hi"));
    adapter.register_command(cmd).await.expect("should succeed");
    let help = <McpAdapter as CommandAdapter>::get_help(&adapter, "hello")
        .await
        .expect("should succeed");
    assert_eq!(help, "hello: Says hello");
}

#[tokio::test]
async fn test_command_adapter_get_help_not_found() {
    let adapter = McpAdapter::new();
    let result = <McpAdapter as CommandAdapter>::get_help(&adapter, "missing").await;
    assert!(result.is_err());
    assert!(matches!(result, Err(CommandError::NotFound(_))));
}

#[tokio::test]
async fn test_execute_command_not_found() {
    let adapter = McpAdapter::new();
    let result = adapter
        .execute_with_auth("nonexistent", vec![], Auth::None)
        .await;
    assert!(result.is_err());
    assert!(matches!(result, Err(CommandError::NotFound(_))));
}
