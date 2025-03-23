use std::sync::Arc;
use serde::{Serialize, Deserialize};
use crate::auth::{AuthCredentials, AuthManager, User};
use thiserror::Error;

/// Error types specific to MCP command integration
#[derive(Debug, Error)]
pub enum McpIntegrationError {
    /// Authentication error
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    
    /// Authorization error
    #[error("Authorization failed: {0}")]
    AuthorizationError(String),
    
    /// Command execution error
    #[error("Command execution failed: {0}")]
    CommandExecutionError(String),
    
    /// Invalid request format
    #[error("Invalid request format: {0}")]
    InvalidRequestFormat(String),
}

/// Result type for MCP integration operations
pub type McpResult<T> = std::result::Result<T, McpIntegrationError>;

/// A request to execute a command via MCP protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCommandRequest {
    /// The name of the command to execute
    pub command: String,
    
    /// The arguments to pass to the command
    pub arguments: Vec<String>,
    
    /// Authentication credentials
    pub credentials: Option<AuthCredentials>,
    
    /// Execution context information
    pub context: McpExecutionContext,
}

/// Context information for command execution via MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpExecutionContext {
    /// Working directory for command execution
    pub working_directory: Option<String>,
    
    /// Environment variables
    pub environment: Option<std::collections::HashMap<String, String>>,
    
    /// Session ID for tracking related commands
    pub session_id: Option<String>,
    
    /// Request timestamp
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

/// Response to a command execution request via MCP protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCommandResponse {
    /// Whether the command execution was successful
    pub success: bool,
    
    /// The output of the command, if successful
    pub output: Option<String>,
    
    /// Error message, if the command failed
    pub error: Option<String>,
    
    /// Additional metadata about the execution
    pub metadata: Option<serde_json::Value>,
    
    /// Response timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl McpCommandResponse {
    /// Creates a successful response
    pub fn success(output: String) -> Self {
        Self {
            success: true,
            output: Some(output),
            error: None,
            metadata: None,
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Creates an error response
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            output: None,
            error: Some(error),
            metadata: None,
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Adds metadata to the response
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// MCP command handler adapter for integrating with the command system
pub struct McpCommandAdapter {
    /// Auth manager for authentication and authorization
    auth_manager: Arc<AuthManager>,
    
    /// Command registry adapter for executing commands
    command_adapter: Arc<crate::adapter::helper::CommandRegistryAdapter>,
}

impl McpCommandAdapter {
    /// Creates a new MCP command adapter
    pub fn new(
        auth_manager: Arc<AuthManager>,
        command_adapter: Arc<crate::adapter::helper::CommandRegistryAdapter>,
    ) -> Self {
        Self {
            auth_manager,
            command_adapter,
        }
    }
    
    /// Handles an MCP command request
    pub async fn handle_command(&self, request: &McpCommandRequest) -> McpCommandResponse {
        // Authenticate if credentials are provided
        let user = match &request.credentials {
            Some(credentials) => {
                match self.authenticate(credentials).await {
                    Ok(user) => Some(user),
                    Err(e) => {
                        return McpCommandResponse::error(format!("Authentication failed: {}", e));
                    }
                }
            },
            None => None,
        };
        
        // Execute the command
        match self.execute_command(&request.command, &request.arguments, user.as_ref()).await {
            Ok(output) => McpCommandResponse::success(output),
            Err(e) => McpCommandResponse::error(format!("Command execution failed: {}", e)),
        }
    }
    
    /// Authenticates a user with the provided credentials
    async fn authenticate(&self, credentials: &AuthCredentials) -> McpResult<User> {
        self.auth_manager.authenticate(credentials).await
            .map_err(|e| McpIntegrationError::AuthenticationError(e.to_string()))
    }
    
    /// Executes a command with the provided arguments and user
    async fn execute_command(&self, command: &str, args: &[String], user: Option<&User>) -> McpResult<String> {
        // If we have a user, check authorization
        if let Some(user) = user {
            // Get the command from the registry
            let registry = self.command_adapter.get_registry()
                .map_err(|e| McpIntegrationError::CommandExecutionError(e.to_string()))?;
            
            // Get the command information within a scope to release the lock before awaiting
            let cmd = {
                let registry_lock = registry.lock().map_err(|_| {
                    McpIntegrationError::CommandExecutionError("Failed to acquire registry lock".to_string())
                })?;
                
                // Check if the command exists
                if !registry_lock.command_exists(command).map_err(|e| {
                    McpIntegrationError::CommandExecutionError(e.to_string())
                })? {
                    return Err(McpIntegrationError::CommandExecutionError(
                        format!("Command not found: {}", command)
                    ));
                }
                
                // Get the command - this returns a Result<Box<dyn Command>>
                registry_lock.get_command(command).map_err(|e| {
                    McpIntegrationError::CommandExecutionError(e.to_string())
                })?
            }; // The MutexGuard is dropped here
            
            // Check authorization - the lock is no longer held at this point
            let auth_result = self.auth_manager.authorize(user, cmd.as_ref()).await
                .map_err(|e| McpIntegrationError::AuthorizationError(e.to_string()))?;
            
            if !auth_result {
                return Err(McpIntegrationError::AuthorizationError(
                    format!("User {} is not authorized to execute command {}", user.name, command)
                ));
            }
        }
        
        // Execute the command
        let args_vec = args.to_vec();
        match self.command_adapter.execute_command(command, args_vec) {
            Ok(output) => Ok(output),
            Err(e) => Err(McpIntegrationError::CommandExecutionError(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mcp_command_request_serialization() {
        // Create a request
        let request = McpCommandRequest {
            command: "test".to_string(),
            arguments: vec!["arg1".to_string(), "arg2".to_string()],
            credentials: Some(AuthCredentials::Basic {
                username: "testuser".to_string(),
                password: "password".to_string(),
            }),
            context: McpExecutionContext {
                working_directory: Some("/home/user".to_string()),
                environment: Some(std::collections::HashMap::new()),
                session_id: Some("test-session".to_string()),
                timestamp: Some(chrono::Utc::now()),
            },
        };
        
        // Serialize to JSON
        let json = serde_json::to_string(&request).unwrap();
        
        // Deserialize from JSON
        let deserialized: McpCommandRequest = serde_json::from_str(&json).unwrap();
        
        // Check values
        assert_eq!(deserialized.command, "test");
        assert_eq!(deserialized.arguments, vec!["arg1", "arg2"]);
        assert!(matches!(deserialized.credentials, Some(AuthCredentials::Basic { .. })));
    }
    
    #[tokio::test]
    async fn test_mcp_command_response_creation() {
        // Create a success response
        let success = McpCommandResponse::success("Command output".to_string());
        assert!(success.success);
        assert_eq!(success.output, Some("Command output".to_string()));
        assert_eq!(success.error, None);
        
        // Create an error response
        let error = McpCommandResponse::error("Error message".to_string());
        assert!(!error.success);
        assert_eq!(error.output, None);
        assert_eq!(error.error, Some("Error message".to_string()));
    }
} 