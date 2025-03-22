use std::sync::Arc;
use crate::{
    Command, CommandResult,
    auth::{User, AuthManager, BasicAuthProvider, AuthCredentials},
    adapter::{create_initialized_registry_adapter, McpCommandAdapter, McpCommandRequest, McpExecutionContext},
};

// Dummy command for testing
#[derive(Debug, Clone)]
struct TestMcpCommand {
    name: String,
}

impl TestMcpCommand {
    fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Command for TestMcpCommand {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        "Test command for MCP integration"
    }
    
    fn execute(&self, args: &[String]) -> CommandResult<String> {
        Ok(format!("TestMcpCommand '{}' executed with args: {:?}", self.name, args))
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("mcp_test")
            .about("Test command for MCP integration")
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

#[tokio::test]
async fn test_mcp_command_integration() {
    // Create auth manager
    let auth_manager = AuthManager::with_provider(Box::new(BasicAuthProvider::new()));
    
    // Create a test user
    let user = User::standard("mcp_test_user", "MCP Test User");
    let password = "test_password";
    auth_manager.add_user_with_password(user.clone(), password).await.unwrap();
    
    // Create command registry adapter
    let registry_adapter = create_initialized_registry_adapter().unwrap();
    
    // Create MCP adapter
    let mcp_adapter = Arc::new(McpCommandAdapter::new(
        Arc::new(auth_manager),
        registry_adapter.clone()
    ));
    
    // Register test command
    let test_command = TestMcpCommand::new("mcp_test");
    registry_adapter.register_command(Box::new(test_command.clone())).unwrap();
    
    // Test 1: Execute command without authentication
    let request = McpCommandRequest {
        command: "mcp_test".to_string(),
        arguments: vec!["arg1".to_string(), "arg2".to_string()],
        credentials: None,
        context: McpExecutionContext {
            working_directory: None,
            environment: None,
            session_id: None,
            timestamp: None,
        },
    };
    
    let response = mcp_adapter.handle_command(&request).await;
    
    assert!(response.success, "Command should succeed without authentication");
    assert!(response.output.is_some(), "Response should have output");
    assert!(response.output.unwrap().contains("mcp_test"), "Output should mention the command");
    
    // Test 2: Execute command with valid authentication
    let request_with_auth = McpCommandRequest {
        command: "mcp_test".to_string(),
        arguments: vec!["arg1".to_string(), "arg2".to_string()],
        credentials: Some(AuthCredentials::Basic {
            username: "mcp_test_user".to_string(),
            password: password.to_string(),
        }),
        context: McpExecutionContext {
            working_directory: None,
            environment: None,
            session_id: None,
            timestamp: None,
        },
    };
    
    let response = mcp_adapter.handle_command(&request_with_auth).await;
    
    assert!(response.success, "Command should succeed with valid authentication");
    assert!(response.output.is_some(), "Response should have output");
    
    // Test 3: Execute command with invalid authentication
    let request_with_invalid_auth = McpCommandRequest {
        command: "mcp_test".to_string(),
        arguments: vec!["arg1".to_string(), "arg2".to_string()],
        credentials: Some(AuthCredentials::Basic {
            username: "mcp_test_user".to_string(),
            password: "wrong_password".to_string(),
        }),
        context: McpExecutionContext {
            working_directory: None,
            environment: None,
            session_id: None,
            timestamp: None,
        },
    };
    
    let response = mcp_adapter.handle_command(&request_with_invalid_auth).await;
    
    assert!(!response.success, "Command should fail with invalid authentication");
    assert!(response.error.is_some(), "Response should have error message");
    assert!(response.error.unwrap().contains("Authentication failed"), "Error should mention authentication failure");
    
    // Test 4: Execute non-existent command
    let request_invalid_command = McpCommandRequest {
        command: "non_existent_command".to_string(),
        arguments: vec![],
        credentials: None,
        context: McpExecutionContext {
            working_directory: None,
            environment: None,
            session_id: None,
            timestamp: None,
        },
    };
    
    let response = mcp_adapter.handle_command(&request_invalid_command).await;
    
    assert!(!response.success, "Non-existent command should fail");
    assert!(response.error.is_some(), "Response should have error message");
    assert!(response.error.unwrap().contains("Command execution failed"), "Error should mention command execution failure");
} 