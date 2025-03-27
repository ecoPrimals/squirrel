// This is a completely standalone test file that doesn't depend on any project modules
// It implements simplified versions of the adapter pattern for testing purposes

use std::sync::{Arc, Mutex};
use std::fmt;
use std::collections::HashMap;
use async_trait::async_trait;

/// Error type for command execution
#[derive(Debug)]
pub enum MockCommandError {
    NotFound(String),
    Execution(String),
    Other(String),
}

impl fmt::Display for MockCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MockCommandError::NotFound(cmd) => write!(f, "Command not found: {}", cmd),
            MockCommandError::Execution(reason) => write!(f, "Execution failed: {}", reason),
            MockCommandError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for MockCommandError {}

/// Result type for command operations
pub type MockCommandResult<T> = Result<T, MockCommandError>;

/// Simplified Command trait for testing
pub trait MockCommand: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: Vec<String>) -> MockCommandResult<String>;
    fn clone_box(&self) -> Box<dyn MockCommand + Send + Sync>;
}

/// Test command implementation
#[derive(Clone)]
struct TestCommand {
    name: String,
    description: String,
    result: String,
}

impl TestCommand {
    fn new(name: &str, description: &str, result: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            result: result.to_string(),
        }
    }
}

impl MockCommand for TestCommand {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn execute(&self, args: Vec<String>) -> MockCommandResult<String> {
        if args.is_empty() {
            Ok(self.result.clone())
        } else {
            Ok(format!("{} with args: {:?}", self.result, args))
        }
    }
    
    fn clone_box(&self) -> Box<dyn MockCommand + Send + Sync> {
        Box::new(self.clone())
    }
}

/// Simplified command registry for testing
struct MockCommandRegistry {
    commands: HashMap<String, Arc<dyn MockCommand + Send + Sync>>,
}

impl MockCommandRegistry {
    fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }
    
    fn register(&mut self, name: &str, command: Arc<dyn MockCommand + Send + Sync>) -> MockCommandResult<()> {
        self.commands.insert(name.to_string(), command);
        Ok(())
    }
    
    fn execute(&self, name: &str, args: Vec<String>) -> MockCommandResult<String> {
        match self.commands.get(name) {
            Some(cmd) => cmd.execute(args),
            None => Err(MockCommandError::NotFound(name.to_string())),
        }
    }
    
    fn get_help(&self, name: &str) -> MockCommandResult<String> {
        match self.commands.get(name) {
            Some(cmd) => Ok(format!("{}: {}", cmd.name(), cmd.description())),
            None => Err(MockCommandError::NotFound(name.to_string())),
        }
    }
    
    fn list_commands(&self) -> MockCommandResult<Vec<String>> {
        Ok(self.commands.keys().cloned().collect())
    }
}

/// Command Registry Adapter - adapts the MockCommandRegistry
struct CommandRegistryAdapter {
    registry: Arc<Mutex<MockCommandRegistry>>,
}

impl CommandRegistryAdapter {
    fn new() -> Self {
        let registry = MockCommandRegistry::new();
        Self {
            registry: Arc::new(Mutex::new(registry)),
        }
    }
    
    fn register_command(&self, command: Arc<dyn MockCommand + Send + Sync>) -> Result<(), String> {
        let cmd_name = command.name().to_string();
        let mut registry = self.registry.lock().map_err(|e| e.to_string())?;
        registry.register(&cmd_name, command).map_err(|e| e.to_string())
    }
    
    async fn execute(&self, command: &str, args: Vec<String>) -> Result<String, String> {
        let registry = self.registry.lock().map_err(|e| e.to_string())?;
        registry.execute(command, args).map_err(|e| e.to_string())
    }
    
    async fn get_help(&self, command: &str) -> Result<String, String> {
        let registry = self.registry.lock().map_err(|e| e.to_string())?;
        registry.get_help(command).map_err(|e| e.to_string())
    }
    
    async fn list_commands(&self) -> Result<Vec<String>, String> {
        let registry = self.registry.lock().map_err(|e| e.to_string())?;
        registry.list_commands().map_err(|e| e.to_string())
    }
}

/// This is a simplified mock adapter trait for testing
#[async_trait]
trait MockAdapter: Send + Sync {
    async fn execute(&self, command: &str, args: Vec<String>) -> Result<String, String>;
    async fn get_help(&self, command: &str) -> Result<String, String>;
}

/// MCP (Machine Context Protocol) Command Adapter
struct McpCommandAdapter {
    adapter: CommandRegistryAdapter,
    authorized_users: HashMap<String, String>, // username -> password
}

impl McpCommandAdapter {
    fn new() -> Self {
        Self {
            adapter: CommandRegistryAdapter::new(),
            authorized_users: {
                let mut map = HashMap::new();
                map.insert("admin".to_string(), "password".to_string());
                map
            },
        }
    }
    
    fn register_command(&self, command: Arc<dyn MockCommand + Send + Sync>) -> Result<(), String> {
        self.adapter.register_command(command)
    }
    
    async fn execute_with_auth(&self, command: &str, args: Vec<String>, auth: Auth) -> Result<String, String> {
        // Check authorization
        match auth {
            Auth::User(username, password) => {
                if let Some(stored_password) = self.authorized_users.get(&username) {
                    if password != *stored_password {
                        return Err(format!("Invalid password for user '{}'", username));
                    }
                } else {
                    return Err(format!("User '{}' not found", username));
                }
                
                // Check if command is admin-only
                if command.starts_with("admin") && username != "admin" {
                    return Err(format!("User '{}' is not authorized to execute command '{}'", username, command));
                }
            },
            Auth::None => {
                // Allow only public commands
                if command.starts_with("admin") {
                    return Err(format!("Authentication required for command '{}'", command));
                }
            }
        }
        
        self.adapter.execute(command, args).await
    }
    
    async fn get_available_commands(&self, auth: Auth) -> Result<Vec<String>, String> {
        let mut commands = self.adapter.list_commands().await?;
        
        // Filter admin commands if not admin
        match auth {
            Auth::User(username, _) => {
                if username != "admin" {
                    commands.retain(|cmd| !cmd.starts_with("admin"));
                }
            },
            Auth::None => {
                commands.retain(|cmd| !cmd.starts_with("admin"));
            }
        }
        
        Ok(commands)
    }
}

/// Authentication types
enum Auth {
    User(String, String), // username, password
    None,
}

/// Commands Plugin Adapter - adapts plugin commands
struct CommandsPluginAdapter {
    adapter: CommandRegistryAdapter,
}

impl CommandsPluginAdapter {
    fn new() -> Self {
        Self {
            adapter: CommandRegistryAdapter::new(),
        }
    }
    
    fn register_command(&self, command: Arc<dyn MockCommand + Send + Sync>) -> Result<(), String> {
        self.adapter.register_command(command)
    }
    
    async fn execute_command(&self, command: &str, args: Vec<String>) -> Result<String, String> {
        self.adapter.execute(command, args).await
    }
    
    async fn get_command_help(&self, command: &str) -> Result<String, String> {
        self.adapter.get_help(command).await
    }
    
    async fn get_commands(&self) -> Result<Vec<String>, String> {
        self.adapter.list_commands().await
    }
}

// Tests

#[tokio::test]
async fn test_registry_adapter_with_test_command() {
    // Create registry adapter
    let adapter = CommandRegistryAdapter::new();
    
    // Create test command
    let test_cmd = TestCommand::new(
        "test", 
        "A test command", 
        "Test command result"
    );
    
    // Register command
    adapter.register_command(Arc::new(test_cmd)).unwrap();
    
    // Execute command without arguments
    let result = adapter.execute("test", vec![]).await.unwrap();
    assert_eq!(result, "Test command result");
    
    // Execute with arguments
    let result = adapter.execute(
        "test", 
        vec!["arg1".to_string(), "arg2".to_string()]
    ).await.unwrap();
    assert_eq!(result, "Test command result with args: [\"arg1\", \"arg2\"]");
    
    // Get help
    let help = adapter.get_help("test").await.unwrap();
    assert_eq!(help, "test: A test command");
    
    // List commands
    let commands = adapter.list_commands().await.unwrap();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0], "test");
    
    // Nonexistent command
    let result = adapter.execute("nonexistent", vec![]).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_mcp_adapter() {
    // Create MCP adapter
    let adapter = McpCommandAdapter::new();
    
    // Register a test command
    let test_cmd = TestCommand::new(
        "test", 
        "A test command", 
        "Test command result"
    );
    
    // Register an admin command
    let admin_cmd = TestCommand::new(
        "admin-cmd", 
        "An admin command", 
        "Admin command result"
    );
    
    adapter.register_command(Arc::new(test_cmd)).unwrap();
    adapter.register_command(Arc::new(admin_cmd)).unwrap();
    
    // Execute regular command with valid authentication
    let result = adapter.execute_with_auth(
        "test", 
        vec![],
        Auth::User("admin".to_string(), "password".to_string())
    ).await.unwrap();
    assert_eq!(result, "Test command result");
    
    // Execute admin command with valid admin authentication
    let result = adapter.execute_with_auth(
        "admin-cmd", 
        vec![],
        Auth::User("admin".to_string(), "password".to_string())
    ).await.unwrap();
    assert_eq!(result, "Admin command result");
    
    // Execute admin command without authentication (should fail)
    let result = adapter.execute_with_auth(
        "admin-cmd", 
        vec![],
        Auth::None
    ).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Authentication required"));
    
    // Execute regular command without authentication (should succeed)
    let result = adapter.execute_with_auth(
        "test", 
        vec![],
        Auth::None
    ).await.unwrap();
    assert_eq!(result, "Test command result");
    
    // Get available commands with admin authentication
    let commands = adapter.get_available_commands(
        Auth::User("admin".to_string(), "password".to_string())
    ).await.unwrap();
    assert_eq!(commands.len(), 2);
    assert!(commands.contains(&"test".to_string()));
    assert!(commands.contains(&"admin-cmd".to_string()));
    
    // Get available commands without authentication
    let commands = adapter.get_available_commands(Auth::None).await.unwrap();
    assert_eq!(commands.len(), 1);
    assert!(commands.contains(&"test".to_string()));
    assert!(!commands.contains(&"admin-cmd".to_string()));
}

#[tokio::test]
async fn test_plugin_adapter() {
    // Create plugin adapter
    let adapter = CommandsPluginAdapter::new();
    
    // Register a test command
    let test_cmd = TestCommand::new(
        "test", 
        "A test command", 
        "Test command result"
    );
    
    adapter.register_command(Arc::new(test_cmd)).unwrap();
    
    // Get commands
    let commands = adapter.get_commands().await.unwrap();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0], "test");
    
    // Execute command
    let result = adapter.execute_command("test", vec!["arg1".to_string(), "arg2".to_string()]).await.unwrap();
    assert_eq!(result, "Test command result with args: [\"arg1\", \"arg2\"]");
    
    // Get help
    let help = adapter.get_command_help("test").await.unwrap();
    assert_eq!(help, "test: A test command");
    
    // Nonexistent command
    let result = adapter.execute_command("nonexistent", vec![]).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_mcp_adapter_authentication() {
    // Create MCP adapter
    let adapter = McpCommandAdapter::new();
    
    // Register a test command
    let test_cmd = TestCommand::new(
        "test", 
        "A test command", 
        "Test command result"
    );
    
    adapter.register_command(Arc::new(test_cmd)).unwrap();
    
    // Valid authentication
    let result = adapter.execute_with_auth(
        "test", 
        vec![],
        Auth::User("admin".to_string(), "password".to_string())
    ).await;
    assert!(result.is_ok());
    
    // Invalid password
    let result = adapter.execute_with_auth(
        "test", 
        vec![],
        Auth::User("admin".to_string(), "wrong_password".to_string())
    ).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid password"));
    
    // Invalid user
    let result = adapter.execute_with_auth(
        "test", 
        vec![],
        Auth::User("nonexistent".to_string(), "password".to_string())
    ).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("User 'nonexistent' not found"));
} 