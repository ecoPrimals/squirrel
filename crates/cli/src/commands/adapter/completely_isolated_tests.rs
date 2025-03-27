// Completely isolated test implementations for adapter pattern
// This module provides simplified mock implementations for testing adapter concepts
// without dependencies on the actual implementation

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

// Simple command trait
#[async_trait]
pub trait IsolatedCommand: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, args: Vec<String>) -> Result<String, String>;
}

// Simple test command implementation
pub struct SimpleTestCommand {
    name: String,
    description: String,
    result: String,
}

impl SimpleTestCommand {
    pub fn new(name: &str, description: &str, result: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            result: result.to_string(),
        }
    }
}

#[async_trait]
impl IsolatedCommand for SimpleTestCommand {
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
}

// Simple adapter
pub struct SimpleAdapter {
    commands: HashMap<String, Arc<dyn IsolatedCommand>>,
}

impl SimpleAdapter {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn register_command(&mut self, command: Arc<dyn IsolatedCommand>) -> Result<(), String> {
        let name = command.name().to_string();
        if self.commands.contains_key(&name) {
            return Err(format!("Command '{}' already registered", name));
        }
        self.commands.insert(name, command);
        Ok(())
    }

    pub fn execute_command(&self, command: &str, args: Vec<String>) -> Result<String, String> {
        let cmd = self.get_command(command)?;
        // Create a blocking runtime for this synchronous API
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(cmd.execute(args))
    }

    pub fn get_help(&self, command: &str) -> Result<String, String> {
        let cmd = self.get_command(command)?;
        Ok(format!("Help for {}: {}", cmd.name(), cmd.description()))
    }

    pub fn list_commands(&self) -> Result<Vec<String>, String> {
        Ok(self.commands.keys().cloned().collect())
    }

    fn get_command(&self, command: &str) -> Result<&Arc<dyn IsolatedCommand>, String> {
        self.commands.get(command).ok_or_else(|| format!("Command '{}' not found", command))
    }
}

// MCP adapter with authentication/authorization
pub struct McpAdapter {
    inner: SimpleAdapter,
    admin_commands: Vec<String>,
}

impl McpAdapter {
    pub fn new() -> Self {
        Self {
            inner: SimpleAdapter::new(),
            admin_commands: Vec::new(),
        }
    }

    pub fn register_command(&mut self, command: Arc<dyn IsolatedCommand>) -> Result<(), String> {
        self.inner.register_command(command)
    }

    pub fn add_admin_command(&mut self, command: &str) {
        if !self.admin_commands.contains(&command.to_string()) {
            self.admin_commands.push(command.to_string());
        }
    }

    pub fn execute_command(&self, command: &str, args: Vec<String>) -> Result<String, String> {
        // Check if command is admin-only
        if self.admin_commands.contains(&command.to_string()) {
            // Get user (first arg)
            let user = args.first().map(|s| s.as_str()).ok_or_else(|| 
                "Authentication required for admin commands".to_string()
            )?;
            
            // Check if user is admin (in a real implementation, this would check credentials)
            if user != "admin" {
                return Err(format!("User '{}' is not authorized for admin commands", user));
            }
            
            // Execute command with remaining args
            self.inner.execute_command(command, args[1..].to_vec())
        } else {
            // Non-admin command, execute directly
            self.inner.execute_command(command, args)
        }
    }

    pub fn get_help(&self, command: &str) -> Result<String, String> {
        self.inner.get_help(command)
    }

    pub fn list_commands(&self) -> Result<Vec<String>, String> {
        self.inner.list_commands()
    }
}

// Plugin trait
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn commands(&self) -> Vec<&Arc<dyn IsolatedCommand>>;
    fn get_command(&self, name: &str) -> Option<&Arc<dyn IsolatedCommand>>;
}

// Simple plugin implementation
pub struct SimplePlugin {
    name: String,
    commands: HashMap<String, Arc<dyn IsolatedCommand>>,
}

impl SimplePlugin {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            commands: HashMap::new(),
        }
    }

    pub fn add_command(&mut self, command: Arc<dyn IsolatedCommand>) {
        self.commands.insert(command.name().to_string(), command);
    }
}

impl Plugin for SimplePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn commands(&self) -> Vec<&Arc<dyn IsolatedCommand>> {
        self.commands.values().collect()
    }

    fn get_command(&self, name: &str) -> Option<&Arc<dyn IsolatedCommand>> {
        self.commands.get(name)
    }
}

// Plugin adapter
pub struct PluginAdapter {
    plugins: Vec<Box<dyn Plugin>>,
    command_map: HashMap<String, (String, Arc<dyn IsolatedCommand>)>,
}

impl PluginAdapter {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            command_map: HashMap::new(),
        }
    }

    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) -> Result<(), String> {
        let plugin_name = plugin.name().to_string();
        
        // Add all commands from this plugin to the command map
        for cmd in plugin.commands() {
            let cmd_name = cmd.name().to_string();
            if self.command_map.contains_key(&cmd_name) {
                return Err(format!("Command '{}' is already registered", cmd_name));
            }
            self.command_map.insert(cmd_name.clone(), (plugin_name.clone(), cmd.clone()));
        }
        
        self.plugins.push(plugin);
        Ok(())
    }

    pub fn register_command(&mut self, _command: Arc<dyn IsolatedCommand>) -> Result<(), String> {
        // Direct command registration is not allowed in plugin adapter
        Err("Direct command registration is not allowed; register a plugin instead".to_string())
    }

    pub fn execute_command(&self, command: &str, args: Vec<String>) -> Result<String, String> {
        if let Some((plugin_name, cmd)) = self.command_map.get(command) {
            // Create a blocking runtime for this synchronous API
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(cmd.execute(args.clone()))?;
            Ok(result)
        } else {
            Err(format!("Command '{}' not found", command))
        }
    }

    pub fn get_help(&self, command: &str) -> Result<String, String> {
        if let Some((_, cmd)) = self.command_map.get(command) {
            Ok(format!("Help for {}: {}", cmd.name(), cmd.description()))
        } else {
            Err(format!("Command '{}' not found", command))
        }
    }

    pub fn list_commands(&self) -> Result<Vec<String>, String> {
        Ok(self.command_map.keys().cloned().collect())
    }
} 