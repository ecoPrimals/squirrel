// This module contains isolated tests for the adapter concepts,
// without dependencies on potentially problematic code.
// It implements its own mock traits and structures for testing purposes.

use std::collections::HashMap;
use std::sync::Arc;

/// A simplified Command trait for testing purposes
pub trait Command {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: Vec<String>) -> Result<String, String>;
}

/// A mock command implementation for testing
pub struct MockCommand {
    name: String,
    description: String,
    result: String,
}

impl MockCommand {
    pub fn new(name: &str, description: &str, result: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            result: result.to_string(),
        }
    }
}

impl Command for MockCommand {
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

/// A trait for command adapters in isolated tests
pub trait MockAdapter {
    fn register_command(&mut self, command: Arc<dyn Command>) -> Result<(), String>;
    fn execute_command(&self, name: &str, args: Vec<String>) -> Result<String, String>;
    fn get_help(&self, name: &str) -> Result<String, String>;
    fn list_commands(&self) -> Result<Vec<String>, String>;
}

/// A simple adapter implementation for testing
pub struct SimpleMockAdapter {
    commands: HashMap<String, Arc<dyn Command>>,
}

impl SimpleMockAdapter {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }
}

impl MockAdapter for SimpleMockAdapter {
    fn register_command(&mut self, command: Arc<dyn Command>) -> Result<(), String> {
        let name = command.name().to_string();
        self.commands.insert(name, command);
        Ok(())
    }

    fn execute_command(&self, name: &str, args: Vec<String>) -> Result<String, String> {
        match self.commands.get(name) {
            Some(cmd) => cmd.execute(args),
            None => Err(format!("Command not found: {}", name)),
        }
    }

    fn get_help(&self, name: &str) -> Result<String, String> {
        match self.commands.get(name) {
            Some(cmd) => Ok(format!("{}: {}", cmd.name(), cmd.description())),
            None => Err(format!("Command not found: {}", name)),
        }
    }

    fn list_commands(&self) -> Result<Vec<String>, String> {
        Ok(self.commands.keys().cloned().collect())
    }
}

/// A mock MCP adapter for testing authentication
pub struct MockMcpAdapter {
    adapter: SimpleMockAdapter,
    require_auth: bool,
    valid_users: Vec<String>,
}

impl MockMcpAdapter {
    pub fn new(require_auth: bool) -> Self {
        Self {
            adapter: SimpleMockAdapter::new(),
            require_auth,
            valid_users: vec!["admin".to_string(), "user".to_string()],
        }
    }

    pub fn register_command(&mut self, command: Arc<dyn Command>) -> Result<(), String> {
        self.adapter.register_command(command)
    }

    pub fn handle_command(&self, command: &str, args: Vec<String>, user: Option<&str>) -> Result<String, String> {
        // Check authentication if required
        if self.require_auth {
            match user {
                Some(username) if self.valid_users.contains(&username.to_string()) => {
                    // User is authorized, proceed
                }
                _ => return Err("Authentication failed".to_string()),
            }
        }

        // Execute the command
        self.adapter.execute_command(command, args)
    }
}

/// A mock Plugin adapter that wraps results with a prefix
pub struct MockPluginAdapter {
    inner: SimpleMockAdapter,
    prefix: String,
}

impl MockPluginAdapter {
    pub fn new(prefix: &str) -> Self {
        Self {
            inner: SimpleMockAdapter::new(),
            prefix: prefix.to_string(),
        }
    }
    
    pub fn register_command(&mut self, command: Arc<dyn Command>) -> Result<(), String> {
        self.inner.register_command(command)
    }
}

impl MockAdapter for MockPluginAdapter {
    fn register_command(&mut self, command: Arc<dyn Command>) -> Result<(), String> {
        self.inner.register_command(command)
    }

    fn execute_command(&self, name: &str, args: Vec<String>) -> Result<String, String> {
        // Add prefix to result
        self.inner.execute_command(name, args).map(|result| format!("{}: {}", self.prefix, result))
    }

    fn get_help(&self, name: &str) -> Result<String, String> {
        self.inner.get_help(name).map(|help| format!("{}: {}", self.prefix, help))
    }

    fn list_commands(&self) -> Result<Vec<String>, String> {
        self.inner.list_commands()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use clap::Command as ClapCommand;
    use async_trait::async_trait;

    // Define a base trait for common non-async methods
    pub trait TestCommandBase: Send + Sync {
        fn name(&self) -> &str;
        fn description(&self) -> &str;
        fn parser(&self) -> ClapCommand;
    }

    // Extend the base trait with async methods
    #[async_trait]
    pub trait AsyncTestCommand: TestCommandBase {
        async fn execute(&self, args: Vec<String>) -> Result<String, String>;
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

    impl TestCommandBase for SimpleCommand {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn parser(&self) -> ClapCommand {
            ClapCommand::new("test")
                .about("A test command")
        }
    }

    #[async_trait]
    impl AsyncTestCommand for SimpleCommand {
        async fn execute(&self, args: Vec<String>) -> Result<String, String> {
            if args.is_empty() {
                Ok(self.result.clone())
            } else {
                Ok(format!("{}: {}", self.result, args.join(" ")))
            }
        }
    }

    // Type erasure wrapper to avoid using dyn AsyncTestCommand directly
    struct TypeErasedCommand<T: AsyncTestCommand> {
        command: T
    }

    impl<T: AsyncTestCommand> TypeErasedCommand<T> {
        fn new(command: T) -> Self {
            Self { command }
        }

        async fn execute(&self, args: Vec<String>) -> Result<String, String> {
            self.command.execute(args).await
        }

        fn name(&self) -> &str {
            self.command.name()
        }

        fn description(&self) -> &str {
            self.command.description()
        }

        fn parser(&self) -> ClapCommand {
            self.command.parser()
        }
    }

    // A simple registry for commands - using concrete types instead of trait objects
    struct CommandRegistry {
        commands: HashMap<String, Arc<SimpleCommand>>,
    }

    impl CommandRegistry {
        fn new() -> Self {
            Self {
                commands: HashMap::new(),
            }
        }

        fn register(&mut self, name: &str, command: Arc<SimpleCommand>) -> Result<(), String> {
            if self.commands.contains_key(name) {
                return Err(format!("Command '{}' already registered", name));
            }
            self.commands.insert(name.to_string(), command);
            Ok(())
        }

        fn get_command(&self, name: &str) -> Option<&Arc<SimpleCommand>> {
            self.commands.get(name)
        }

        fn list_commands(&self) -> Vec<String> {
            self.commands.keys().cloned().collect()
        }
    }

    // Async command adapter trait
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

        async fn register_command(&self, name: &str, command: Arc<SimpleCommand>) -> Result<(), String> {
            let mut registry = self.registry.lock().await;
            registry.register(name, command)
        }
    }

    #[async_trait]
    impl CommandAdapter for RegistryAdapter {
        async fn execute_command(&self, command: &str, args: Vec<String>) -> Result<String, String> {
            let registry = self.registry.lock().await;
            match registry.get_command(command) {
                Some(cmd) => cmd.execute(args).await,
                None => Err(format!("Command not found: {}", command)),
            }
        }

        async fn get_help(&self, command: &str) -> Result<String, String> {
            let registry = self.registry.lock().await;
            match registry.get_command(command) {
                Some(cmd) => Ok(format!("{}: {}", cmd.name(), cmd.description())),
                None => Err(format!("Command not found: {}", command)),
            }
        }

        async fn list_commands(&self) -> Result<Vec<String>, String> {
            let registry = self.registry.lock().await;
            Ok(registry.list_commands())
        }
    }

    // MCP adapter implementation
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
                Some(_) => Err("Invalid token".to_string()),
                None => Err("Missing token".to_string()),
            }
        }
    }

    #[async_trait]
    impl CommandAdapter for McpAdapter {
        async fn execute_command(&self, command: &str, args: Vec<String>) -> Result<String, String> {
            // Check if the first arg is a token and use it for auth if needed
            let (token, actual_args) = if !args.is_empty() && args[0].starts_with("token=") {
                let token = args[0].strip_prefix("token=").unwrap();
                (Some(token), args[1..].to_vec())
            } else {
                (None, args)
            };

            // Authorize
            if let Err(e) = self.authorize(token).await {
                return Err(e);
            }

            // Execute command
            self.registry_adapter.execute_command(command, actual_args).await
        }

        async fn get_help(&self, command: &str) -> Result<String, String> {
            self.registry_adapter.get_help(command).await
        }

        async fn list_commands(&self) -> Result<Vec<String>, String> {
            self.registry_adapter.list_commands().await
        }
    }

    // Plugin adapter implementation
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
            if self.loaded {
                return Ok(());
            }

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
    #[tokio::test]
    async fn test_registry_adapter() {
        // Create the registry adapter
        let adapter = Arc::new(RegistryAdapter::new());

        // Register commands
        let echo_cmd = Arc::new(SimpleCommand::new("echo", "Echo command", "Echo"));
        let hello_cmd = Arc::new(SimpleCommand::new("hello", "Hello command", "Hello"));

        adapter.register_command("echo", echo_cmd).await.unwrap();
        adapter.register_command("hello", hello_cmd).await.unwrap();

        // Test command execution
        let result = adapter.execute_command("echo", vec!["world".to_string()]).await;
        assert_eq!(result, Ok("Echo: world".to_string()));

        let result = adapter.execute_command("hello", vec![]).await;
        assert_eq!(result, Ok("Hello".to_string()));

        // Test help
        let help = adapter.get_help("echo").await;
        assert_eq!(help, Ok("echo: Echo command".to_string()));

        // Test list
        let commands = adapter.list_commands().await;
        assert!(commands.is_ok());
        let commands = commands.unwrap();
        assert_eq!(commands.len(), 2);
        assert!(commands.contains(&"echo".to_string()));
        assert!(commands.contains(&"hello".to_string()));
    }

    #[tokio::test]
    async fn test_mcp_adapter() {
        // Create the registry adapter
        let reg_adapter = Arc::new(RegistryAdapter::new());

        // Register commands
        let echo_cmd = Arc::new(SimpleCommand::new("echo", "Echo command", "Echo"));
        let hello_cmd = Arc::new(SimpleCommand::new("hello", "Hello command", "Hello"));

        reg_adapter.register_command("echo", echo_cmd).await.unwrap();
        reg_adapter.register_command("hello", hello_cmd).await.unwrap();

        // Create MCP adapter with auth required
        let mcp_adapter = McpAdapter::new(reg_adapter.clone(), true);

        // Test with missing token
        let result = mcp_adapter.execute_command("echo", vec!["world".to_string()]).await;
        assert!(result.is_err());
        assert_eq!(result, Err("Missing token".to_string()));

        // Test with invalid token
        let result = mcp_adapter.execute_command("echo", vec!["token=invalid".to_string(), "world".to_string()]).await;
        assert!(result.is_err());
        assert_eq!(result, Err("Invalid token".to_string()));

        // Test with valid token
        let result = mcp_adapter.execute_command("echo", vec!["token=valid_token".to_string(), "world".to_string()]).await;
        assert_eq!(result, Ok("Echo: world".to_string()));

        // Create MCP adapter without auth required
        let mcp_adapter = McpAdapter::new(reg_adapter.clone(), false);

        // Test without token
        let result = mcp_adapter.execute_command("echo", vec!["world".to_string()]).await;
        assert_eq!(result, Ok("Echo: world".to_string()));
    }

    #[tokio::test]
    async fn test_plugin_adapter() {
        // Create the registry adapter
        let reg_adapter = Arc::new(RegistryAdapter::new());

        // Register commands
        let echo_cmd = Arc::new(SimpleCommand::new("echo", "Echo command", "Echo"));
        let hello_cmd = Arc::new(SimpleCommand::new("hello", "Hello command", "Hello"));

        reg_adapter.register_command("echo", echo_cmd).await.unwrap();
        reg_adapter.register_command("hello", hello_cmd).await.unwrap();

        // Create Plugin adapter
        let mut plugin_adapter = PluginAdapter::new(reg_adapter.clone());

        // Test before loading plugins
        let result = plugin_adapter.execute_command("echo", vec!["world".to_string()]).await;
        assert!(result.is_err());
        assert_eq!(result, Err("Plugins not loaded".to_string()));

        // Load plugins
        plugin_adapter.load_plugins().await.unwrap();

        // Test after loading plugins
        let result = plugin_adapter.execute_command("echo", vec!["world".to_string()]).await;
        assert_eq!(result, Ok("Echo: world".to_string()));

        // Test list commands
        let commands = plugin_adapter.list_commands().await;
        assert!(commands.is_ok());
        let commands = commands.unwrap();
        assert_eq!(commands.len(), 2);
        assert!(commands.contains(&"echo".to_string()));
        assert!(commands.contains(&"hello".to_string()));
    }
} 