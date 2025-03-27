// Tests for the adapter implementations
use std::sync::{Arc, Mutex};
use clap::Command as ClapCommand;
use clap::Arg;
use commands::Command;

/// Mock Command trait for testing
pub trait TestCommand: Send + Sync {
    /// Get the command name
    fn name(&self) -> &str;
    
    /// Get the command description
    fn description(&self) -> &str;
    
    /// Execute the command
    fn execute(&self, args: Vec<String>) -> Result<String, String>;
    
    /// Create the command parser
    fn parser(&self) -> ClapCommand {
        ClapCommand::new("test")
            .about("Test command for unit tests")
            .arg(Arg::new("args")
                .action(clap::ArgAction::Append)
                .help("Arguments for the test command"))
    }
}

/// Mock command implementation for testing
pub struct MockTestCommand {
    name: String,
    description: String,
    result: String,
}

impl MockTestCommand {
    /// Create a new mock command
    pub fn new(name: &str, description: &str, result: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            result: result.to_string(),
        }
    }
}

impl TestCommand for MockTestCommand {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn execute(&self, args: Vec<String>) -> Result<String, String> {
        if args.is_empty() {
            Ok(self.result.clone())
        } else {
            Ok(format!("{}: {}", self.result, args.join(" ")))
        }
    }
}

/// Mock adapter trait
trait MockAdapter: Send + Sync {
    fn register_command(&self, name: &str, command: Arc<dyn TestCommand>) -> Result<(), String>;
    fn execute(&self, command: &str, args: Vec<String>) -> Result<String, String>;
    fn get_help(&self, command: &str) -> Result<String, String>;
    fn list_commands(&self) -> Result<Vec<String>, String>;
}

/// Simple mock adapter implementation
struct SimpleMockAdapter {
    commands: Mutex<std::collections::HashMap<String, Arc<dyn TestCommand>>>,
}

impl SimpleMockAdapter {
    fn new() -> Self {
        Self {
            commands: Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl MockAdapter for SimpleMockAdapter {
    fn register_command(&self, name: &str, command: Arc<dyn TestCommand>) -> Result<(), String> {
        let mut commands = self.commands.lock().map_err(|e| format!("Lock error: {}", e))?;
        commands.insert(name.to_string(), command);
        Ok(())
    }

    fn execute(&self, command: &str, args: Vec<String>) -> Result<String, String> {
        let commands = self.commands.lock().map_err(|e| format!("Lock error: {}", e))?;
        match commands.get(command) {
            Some(cmd) => cmd.execute(args),
            None => Err(format!("Command not found: {}", command)),
        }
    }

    fn get_help(&self, command: &str) -> Result<String, String> {
        let commands = self.commands.lock().map_err(|e| format!("Lock error: {}", e))?;
        match commands.get(command) {
            Some(_) => Ok(format!("Help for {}", command)),
            None => Err(format!("Command not found: {}", command)),
        }
    }

    fn list_commands(&self) -> Result<Vec<String>, String> {
        let commands = self.commands.lock().map_err(|e| format!("Lock error: {}", e))?;
        Ok(commands.keys().cloned().collect())
    }
}

/// Mock MCP adapter that adds authorization
struct MockMcpAdapter {
    inner: SimpleMockAdapter,
    authorized_users: Mutex<std::collections::HashSet<String>>,
}

impl MockMcpAdapter {
    fn new() -> Self {
        Self {
            inner: SimpleMockAdapter::new(),
            authorized_users: Mutex::new(["admin".to_string(), "test_user".to_string()].into()),
        }
    }

    fn is_authorized(&self, user: &str) -> bool {
        let users = self.authorized_users.lock().unwrap();
        users.contains(user)
    }
}

impl MockAdapter for MockMcpAdapter {
    fn register_command(&self, name: &str, command: Arc<dyn TestCommand>) -> Result<(), String> {
        self.inner.register_command(name, command)
    }

    fn execute(&self, command: &str, args: Vec<String>) -> Result<String, String> {
        // Get the user from the first argument
        if args.is_empty() {
            return Err("Missing user argument".to_string());
        }
        
        let user = &args[0];
        if !self.is_authorized(user) {
            return Err(format!("User {} is not authorized", user));
        }
        
        // Pass the remaining arguments to the command
        self.inner.execute(command, args[1..].to_vec())
    }

    fn get_help(&self, command: &str) -> Result<String, String> {
        self.inner.get_help(command)
    }

    fn list_commands(&self) -> Result<Vec<String>, String> {
        self.inner.list_commands()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_adapter_basic() {
        let adapter = SimpleMockAdapter::new();
        let command = MockTestCommand::new("test", "Test command", "Test result");
        adapter.register_command("test", Arc::new(command)).unwrap();
        
        // Execute without args
        let result = adapter.execute("test", vec![]).unwrap();
        assert_eq!(result, "Test result");
        
        // Execute with args
        let result = adapter.execute("test", vec!["arg1".to_string(), "arg2".to_string()]).unwrap();
        assert_eq!(result, "Test result: arg1 arg2");
        
        // Get help
        let help = adapter.get_help("test").unwrap();
        assert_eq!(help, "Help for test");
        
        // List commands
        let commands = adapter.list_commands().unwrap();
        assert_eq!(commands, vec!["test".to_string()]);
        
        // Execute non-existent command
        let result = adapter.execute("nonexistent", vec![]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_mcp_adapter_authentication() {
        let adapter = MockMcpAdapter::new();
        let command = MockTestCommand::new("admin_cmd", "Admin command", "Admin result");
        adapter.register_command("admin_cmd", Arc::new(command)).unwrap();
        
        // Authorized user
        let result = adapter.execute("admin_cmd", vec!["admin".to_string()]).unwrap();
        assert_eq!(result, "Admin result");
        
        // Another authorized user with args
        let result = adapter.execute("admin_cmd", vec!["test_user".to_string(), "arg1".to_string()]).unwrap();
        assert_eq!(result, "Admin result: arg1");
        
        // Unauthorized user
        let result = adapter.execute("admin_cmd", vec!["guest".to_string()]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "User guest is not authorized");
        
        // Missing user
        let result = adapter.execute("admin_cmd", vec![]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing user argument");
    }
}