use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use clap::Command as ClapCommand;
use tokio::sync::Mutex;

// Define our own Command trait
#[async_trait]
pub trait TestCommand: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, args: Vec<String>) -> Result<String, String>;
    fn parser(&self) -> ClapCommand;
}

// A simple test command implementation
struct SimpleCommand {
    name: String,
    description: String,
    result: String,
}

impl SimpleCommand {
    fn new(name: &str, description: &str, result: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            result: result.to_string(),
        }
    }
}

#[async_trait]
impl TestCommand for SimpleCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, args: Vec<String>) -> Result<String, String> {
        if args.is_empty() {
            Ok(self.result.clone())
        } else {
            Ok(format!("{}: {}", self.result, args.join(" ")))
        }
    }

    fn parser(&self) -> ClapCommand {
        ClapCommand::new("test")
            .about("A test command")
    }
}

// A simple registry for commands
struct CommandRegistry {
    commands: HashMap<String, Arc<dyn TestCommand>>,
}

impl CommandRegistry {
    fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    fn register(&mut self, name: &str, command: Arc<dyn TestCommand>) -> Result<(), String> {
        if self.commands.contains_key(name) {
            return Err(format!("Command '{}' already registered", name));
        }
        self.commands.insert(name.to_string(), command);
        Ok(())
    }

    fn get_command(&self, name: &str) -> Option<&Arc<dyn TestCommand>> {
        self.commands.get(name)
    }

    fn list_commands(&self) -> Vec<String> {
        self.commands.keys().cloned().collect()
    }
}

// A simple adapter trait
#[async_trait]
trait CommandAdapter {
    async fn execute_command(&self, command: &str, args: Vec<String>) -> Result<String, String>;
    async fn get_help(&self, command: &str) -> Result<String, String>;
    async fn list_commands(&self) -> Result<Vec<String>, String>;
}

// Registry adapter implementation
struct RegistryAdapter {
    registry: Arc<Mutex<CommandRegistry>>,
}

impl RegistryAdapter {
    fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(CommandRegistry::new())),
        }
    }

    async fn register_command(&self, name: &str, command: Arc<dyn TestCommand>) -> Result<(), String> {
        let mut registry = self.registry.lock().await;
        registry.register(name, command)
    }
}

#[async_trait]
impl CommandAdapter for RegistryAdapter {
    async fn execute_command(&self, command: &str, args: Vec<String>) -> Result<String, String> {
        let registry = self.registry.lock().await;
        let cmd = registry.get_command(command)
            .ok_or_else(|| format!("Command '{}' not found", command))?;
        cmd.execute(args).await
    }

    async fn get_help(&self, command: &str) -> Result<String, String> {
        let registry = self.registry.lock().await;
        let cmd = registry.get_command(command)
            .ok_or_else(|| format!("Command '{}' not found", command))?;
        Ok(format!("{}: {}", cmd.name(), cmd.description()))
    }

    async fn list_commands(&self) -> Result<Vec<String>, String> {
        let registry = self.registry.lock().await;
        Ok(registry.list_commands())
    }
}

// MCP adapter - adapts command execution with authentication
struct McpAdapter {
    registry_adapter: Arc<RegistryAdapter>,
    auth_required: bool,
}

impl McpAdapter {
    fn new(registry_adapter: Arc<RegistryAdapter>, auth_required: bool) -> Self {
        Self {
            registry_adapter,
            auth_required,
        }
    }

    async fn authorize(&self, token: Option<&str>) -> Result<bool, String> {
        if !self.auth_required {
            return Ok(true);
        }
        
        match token {
            Some("valid_token") => Ok(true),
            Some(_) => Ok(false),
            None => Ok(false),
        }
    }
}

#[async_trait]
impl CommandAdapter for McpAdapter {
    async fn execute_command(&self, command: &str, args: Vec<String>) -> Result<String, String> {
        // Extract auth token from args if present
        let token = args.first().map(|s| s.as_str());
        
        // Check authorization
        let authorized = self.authorize(token).await?;
        if !authorized {
            return Err("Unauthorized".to_string());
        }
        
        // Execute command with remaining args
        let args_without_token = if token.is_some() {
            args[1..].to_vec()
        } else {
            args
        };
        
        self.registry_adapter.execute_command(command, args_without_token).await
    }

    async fn get_help(&self, command: &str) -> Result<String, String> {
        self.registry_adapter.get_help(command).await
    }

    async fn list_commands(&self) -> Result<Vec<String>, String> {
        self.registry_adapter.list_commands().await
    }
}

// Plugin adapter - adapts registry adapter to work with plugins
struct PluginAdapter {
    registry_adapter: Arc<RegistryAdapter>,
    loaded: bool,
}

impl PluginAdapter {
    fn new(registry_adapter: Arc<RegistryAdapter>) -> Self {
        Self {
            registry_adapter,
            loaded: false,
        }
    }
    
    async fn load_plugins(&mut self) -> Result<(), String> {
        // Simulate loading plugins
        self.loaded = true;
        Ok(())
    }
}

#[async_trait]
impl CommandAdapter for PluginAdapter {
    async fn execute_command(&self, command: &str, args: Vec<String>) -> Result<String, String> {
        if !self.loaded {
            return Err("Plugins not loaded".to_string());
        }
        self.registry_adapter.execute_command(command, args).await
    }

    async fn get_help(&self, command: &str) -> Result<String, String> {
        if !self.loaded {
            return Err("Plugins not loaded".to_string());
        }
        self.registry_adapter.get_help(command).await
    }

    async fn list_commands(&self) -> Result<Vec<String>, String> {
        if !self.loaded {
            return Err("Plugins not loaded".to_string());
        }
        self.registry_adapter.list_commands().await
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_registry_adapter() {
        let adapter = RegistryAdapter::new();
        
        // Register a test command
        let command = Arc::new(SimpleCommand::new(
            "test",
            "A test command",
            "Test successful"
        ));
        
        adapter.register_command("test", command).await.unwrap();
        
        // Test command execution
        let result = adapter.execute_command("test", vec![]).await.unwrap();
        assert_eq!(result, "Test successful");
        
        // Test command execution with arguments
        let result = adapter.execute_command("test", vec!["arg1".to_string(), "arg2".to_string()]).await.unwrap();
        assert_eq!(result, "Test successful: arg1 arg2");
        
        // Test command help
        let help = adapter.get_help("test").await.unwrap();
        assert_eq!(help, "test: A test command");
        
        // Test list commands
        let commands = adapter.list_commands().await.unwrap();
        assert_eq!(commands, vec!["test"]);
        
        // Test command not found
        let result = adapter.execute_command("nonexistent", vec![]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mcp_adapter() {
        let registry_adapter = Arc::new(RegistryAdapter::new());
        
        // Register a test command
        let command = Arc::new(SimpleCommand::new(
            "test",
            "A test command",
            "Test successful"
        ));
        
        registry_adapter.register_command("test", command).await.unwrap();
        
        // Create MCP adapter without auth required
        let mcp_adapter = McpAdapter::new(registry_adapter.clone(), false);
        
        // Test command execution without auth
        let result = mcp_adapter.execute_command("test", vec![]).await.unwrap();
        assert_eq!(result, "Test successful");
        
        // Create MCP adapter with auth required
        let mcp_adapter_auth = McpAdapter::new(registry_adapter, true);
        
        // Test unauthorized execution
        let result = mcp_adapter_auth.execute_command("test", vec![]).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unauthorized");
        
        // Test authorized execution
        let result = mcp_adapter_auth.execute_command("test", vec!["valid_token".to_string(), "arg1".to_string()]).await.unwrap();
        assert_eq!(result, "Test successful: arg1");
        
        // Test invalid token
        let result = mcp_adapter_auth.execute_command("test", vec!["invalid_token".to_string()]).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unauthorized");
    }

    #[tokio::test]
    async fn test_plugin_adapter() {
        let registry_adapter = Arc::new(RegistryAdapter::new());
        
        // Register a test command
        let command = Arc::new(SimpleCommand::new(
            "test",
            "A test command",
            "Test successful"
        ));
        
        registry_adapter.register_command("test", command).await.unwrap();
        
        // Create plugin adapter (not loaded yet)
        let mut plugin_adapter = PluginAdapter::new(registry_adapter);
        
        // Test before loading
        let result = plugin_adapter.execute_command("test", vec![]).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Plugins not loaded");
        
        // Load plugins
        plugin_adapter.load_plugins().await.unwrap();
        
        // Test after loading
        let result = plugin_adapter.execute_command("test", vec![]).await.unwrap();
        assert_eq!(result, "Test successful");
        
        let help = plugin_adapter.get_help("test").await.unwrap();
        assert_eq!(help, "test: A test command");
        
        let commands = plugin_adapter.list_commands().await.unwrap();
        assert_eq!(commands, vec!["test"]);
    }
} 