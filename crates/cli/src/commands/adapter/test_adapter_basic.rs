//! Basic adapter pattern tests
//!
//! These tests verify the core adapter pattern functionality
//! without depending on implementation-specific details.
//! 
//! The adapter pattern implemented here transforms an interface 
//! into another through composition. This test suite validates
//! basic adapter behaviors:
//! 
//! 1. Adapters forwarding method calls to adaptees
//! 2. Adapters providing type conversion across interfaces
//! 3. Adapters handling error states consistently

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::fmt;

// We define our own types for testing to avoid 
// relying on the implementation
type CommandResult<T> = Result<T, String>;

/// The basic command trait that we're adapting
trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: Vec<String>) -> CommandResult<String>;
}

/// Test command implementation
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

impl Command for TestCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn execute(&self, args: Vec<String>) -> CommandResult<String> {
        if args.is_empty() {
            Ok(self.result.clone())
        } else {
            Ok(format!("{} with args: {:?}", self.result, args))
        }
    }
}

/// Command registry that holds commands
struct CommandRegistry {
    commands: HashMap<String, Arc<dyn Command>>,
}

impl CommandRegistry {
    fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    fn register(&mut self, name: &str, command: Arc<dyn Command>) -> CommandResult<()> {
        self.commands.insert(name.to_string(), command);
        Ok(())
    }

    fn execute(&self, name: &str, args: Vec<String>) -> CommandResult<String> {
        match self.commands.get(name) {
            Some(cmd) => cmd.execute(args),
            None => Err(format!("Command not found: {}", name)),
        }
    }

    fn get_help(&self, name: &str) -> CommandResult<String> {
        match self.commands.get(name) {
            Some(cmd) => Ok(format!("{}: {}", cmd.name(), cmd.description())),
            None => Err(format!("Command not found: {}", name)),
        }
    }

    fn list_commands(&self) -> CommandResult<Vec<String>> {
        Ok(self.commands.keys().cloned().collect())
    }
}

/// The adapter that transforms the synchronous registry into an async interface
struct RegistryAdapter {
    registry: Arc<Mutex<CommandRegistry>>,
}

impl RegistryAdapter {
    fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(CommandRegistry::new())),
        }
    }

    fn register_command(&self, command: Arc<dyn Command>) -> CommandResult<()> {
        let mut registry = self.registry.lock().map_err(|e| format!("Lock error: {}", e))?;
        registry.register(command.name(), command)
    }

    async fn execute(&self, command: &str, args: Vec<String>) -> CommandResult<String> {
        let registry = self.registry.lock().map_err(|e| format!("Lock error: {}", e))?;
        registry.execute(command, args)
    }

    async fn get_help(&self, command: &str) -> CommandResult<String> {
        let registry = self.registry.lock().map_err(|e| format!("Lock error: {}", e))?;
        registry.get_help(command)
    }

    async fn list_commands(&self) -> CommandResult<Vec<String>> {
        let registry = self.registry.lock().map_err(|e| format!("Lock error: {}", e))?;
        registry.list_commands()
    }
}

/// Authentication type for MCP adapter
#[derive(Clone)]
enum Auth {
    User(String, String),  // username, password
    None,
}

/// MCP adapter with authentication
struct McpAdapter {
    adapter: RegistryAdapter,
    authorized_users: HashMap<String, String>,
}

impl McpAdapter {
    fn new() -> Self {
        let mut instance = Self {
            adapter: RegistryAdapter::new(),
            authorized_users: HashMap::new(),
        };
        
        // Add default admin user
        instance.authorized_users.insert("admin".to_string(), "password".to_string());
        
        instance
    }
    
    fn register_command(&self, command: Arc<dyn Command>) -> CommandResult<()> {
        self.adapter.register_command(command)
    }
    
    async fn execute_with_auth(&self, command: &str, args: Vec<String>, auth: Auth) -> CommandResult<String> {
        // Check authorization for admin commands
        match auth {
            Auth::User(username, password) => {
                // Verify user credentials
                if let Some(stored_password) = self.authorized_users.get(&username) {
                    if password != *stored_password {
                        return Err(format!("Authentication failed: Invalid password for user '{}'", username));
                    }
                } else {
                    return Err(format!("Authentication failed: User '{}' not found", username));
                }
                
                // Check if command requires admin role
                if command.starts_with("admin") && username != "admin" {
                    return Err(format!("Authorization failed: User '{}' cannot access admin command '{}'", 
                               username, command));
                }
            },
            Auth::None => {
                // Allow only public commands for anonymous users
                if command.starts_with("admin") {
                    return Err(format!("Authentication failed: Authentication required for command '{}'", command));
                }
            }
        }
        
        // Execute the command
        self.adapter.execute(command, args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_registry_adapter() -> CommandResult<()> {
        // Create a registry adapter
        let adapter = RegistryAdapter::new();
        
        // Create and register test commands
        let hello_cmd = TestCommand::new("hello", "Says hello", "Hello, world!");
        let echo_cmd = TestCommand::new("echo", "Echoes arguments", "Echo");
        
        adapter.register_command(Arc::new(hello_cmd))?;
        adapter.register_command(Arc::new(echo_cmd))?;
        
        // Test command execution without arguments
        let result = adapter.execute("hello", vec![]).await?;
        assert_eq!(result, "Hello, world!");
        
        // Test command execution with arguments
        let result = adapter.execute("echo", vec!["Hello".to_string(), "there!".to_string()]).await?;
        assert_eq!(result, "Echo with args: [\"Hello\", \"there!\"]");
        
        // Test help information
        let help = adapter.get_help("hello").await?;
        assert_eq!(help, "hello: Says hello");
        
        // Test listing commands
        let commands = adapter.list_commands().await?;
        assert_eq!(commands.len(), 2);
        assert!(commands.contains(&"hello".to_string()));
        assert!(commands.contains(&"echo".to_string()));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_mcp_adapter_authentication() -> CommandResult<()> {
        // Create an MCP adapter
        let adapter = McpAdapter::new();
        
        // Register a secure command
        let cmd = TestCommand::new("secure", "Secure command", "Secret data");
        adapter.register_command(Arc::new(cmd))?;
        
        // Register an admin command
        let admin_cmd = TestCommand::new("admin-cmd", "Admin command", "Admin data");
        adapter.register_command(Arc::new(admin_cmd))?;
        
        // Test admin authentication and command execution
        let admin_auth = Auth::User("admin".to_string(), "password".to_string());
        let result = adapter.execute_with_auth("admin-cmd", vec![], admin_auth.clone()).await?;
        assert_eq!(result, "Admin data");
        
        // Test anonymous access to regular command (should succeed)
        let result = adapter.execute_with_auth("secure", vec![], Auth::None).await?;
        assert_eq!(result, "Secret data");
        
        // Test anonymous access to admin command (should fail)
        let result = adapter.execute_with_auth("admin-cmd", vec![], Auth::None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Authentication failed"));
        
        // Test access with invalid credentials
        let invalid_auth = Auth::User("admin".to_string(), "wrong_password".to_string());
        let result = adapter.execute_with_auth("secure", vec![], invalid_auth).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid password"));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_nonexistent_command() -> CommandResult<()> {
        // Create a registry adapter
        let adapter = RegistryAdapter::new();
        
        // Try to execute a nonexistent command
        let result = adapter.execute("nonexistent", vec![]).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Command not found"));
        
        // Try to get help for a nonexistent command
        let result = adapter.get_help("nonexistent").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Command not found"));
        
        Ok(())
    }
} 