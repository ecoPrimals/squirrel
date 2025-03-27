//! Adapter Pattern Implementation and Tests
//!
//! This crate demonstrates the adapter pattern in Rust with a command-based
//! architecture. Three main adapters are implemented:
//!
//! 1. Registry Adapter - Basic adapter for command registry operations
//! 2. MCP Adapter - Adapter with authentication and authorization
//! 3. Plugin Adapter - Adapter for plugin system integration
//!
//! Each adapter uses composition to transform one interface into another.
//!
//! ## What is the Adapter Pattern?
//!
//! The Adapter Pattern is a structural design pattern that allows objects with
//! incompatible interfaces to collaborate. It acts as a wrapper to convert one
//! interface into another that clients expect.
//!
//! ## When to Use the Adapter Pattern
//!
//! - When you need to use an existing class with an interface that doesn't match your needs
//! - When you want to reuse existing subclasses without extensive modification
//! - When you need to integrate with external systems or APIs
//! - When you want to isolate your code from third-party libraries
//!
//! ## Benefits of the Adapter Pattern
//!
//! - Enhances code reusability
//! - Enables integration between incompatible interfaces
//! - Promotes separation of concerns
//! - Provides a clean abstraction layer between components
//! - Makes testing easier through mocking and isolation

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::fmt::{self, Debug};
use async_trait::async_trait;

// Basic types
pub type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug)]
pub enum CommandError {
    NotFound(String),
    ExecutionFailed(String),
    AuthenticationFailed(String),
    AuthorizationFailed(String),
    Other(String),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::NotFound(s) => write!(f, "Command not found: {}", s),
            CommandError::ExecutionFailed(s) => write!(f, "Execution failed: {}", s),
            CommandError::AuthenticationFailed(s) => write!(f, "Authentication failed: {}", s),
            CommandError::AuthorizationFailed(s) => write!(f, "Authorization failed: {}", s),
            CommandError::Other(s) => write!(f, "Error: {}", s),
        }
    }
}

/// Command trait representing a command interface
pub trait Command: Send + Sync + Debug {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: Vec<String>) -> CommandResult<String>;
}

/// TestCommand is a mock command implementation for testing
#[derive(Debug, Clone)]
pub struct TestCommand {
    name: String,
    description: String,
    result: String,
}

impl TestCommand {
    pub fn new(name: &str, description: &str, result: &str) -> Self {
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

/// Command registry to store and execute commands
#[derive(Debug)]
pub struct CommandRegistry {
    commands: HashMap<String, Arc<dyn Command>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, command: Arc<dyn Command>) -> CommandResult<()> {
        self.commands.insert(name.to_string(), command);
        Ok(())
    }

    pub fn execute(&self, name: &str, args: Vec<String>) -> CommandResult<String> {
        match self.commands.get(name) {
            Some(cmd) => cmd.execute(args),
            None => Err(CommandError::NotFound(name.to_string())),
        }
    }

    pub fn get_help(&self, name: &str) -> CommandResult<String> {
        match self.commands.get(name) {
            Some(cmd) => Ok(format!("{}: {}", cmd.name(), cmd.description())),
            None => Err(CommandError::NotFound(name.to_string())),
        }
    }

    pub fn list_commands(&self) -> CommandResult<Vec<String>> {
        Ok(self.commands.keys().cloned().collect())
    }
}

/// Adapter interface for command operations
#[async_trait]
pub trait CommandAdapter: Send + Sync {
    async fn execute(&self, command: &str, args: Vec<String>) -> CommandResult<String>;
    async fn get_help(&self, command: &str) -> CommandResult<String>;
    async fn list_commands(&self) -> CommandResult<Vec<String>>;
}

/// Registry adapter implementation
#[derive(Debug)]
pub struct RegistryAdapter {
    commands: HashMap<String, Arc<dyn Command>>,
}

impl RegistryAdapter {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, command: Arc<dyn Command>) -> CommandResult<()> {
        self.commands.insert(name.to_string(), command);
        Ok(())
    }

    pub fn execute(&self, name: &str, args: Vec<String>) -> CommandResult<String> {
        match self.commands.get(name) {
            Some(cmd) => cmd.execute(args),
            None => Err(CommandError::NotFound(name.to_string())),
        }
    }

    pub fn get_help(&self, name: &str) -> CommandResult<String> {
        match self.commands.get(name) {
            Some(cmd) => Ok(format!("{}: {}", cmd.name(), cmd.description())),
            None => Err(CommandError::NotFound(name.to_string())),
        }
    }

    pub fn list_commands(&self) -> CommandResult<Vec<String>> {
        Ok(self.commands.keys().cloned().collect())
    }
}

#[async_trait]
impl CommandAdapter for RegistryAdapter {
    async fn execute(&self, command: &str, args: Vec<String>) -> CommandResult<String> {
        self.execute(command, args)
    }
    
    async fn get_help(&self, command: &str) -> CommandResult<String> {
        self.get_help(command)
    }
    
    async fn list_commands(&self) -> CommandResult<Vec<String>> {
        self.list_commands()
    }
}

/// Authentication type for MCP adapter
#[derive(Debug, Clone)]
pub enum Auth {
    User(String, String), // username, password
    Token(String), // authentication token
    ApiKey(String), // API key
    None,
}

/// User role for permission management
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    PowerUser,
    RegularUser,
    Guest,
}

/// Authentication result containing user information
#[derive(Debug, Clone)]
pub struct AuthUser {
    username: String,
    roles: Vec<UserRole>,
    token: Option<String>,
}

/// LogEntry for command audit logging
#[derive(Debug, Clone)]
pub struct CommandLogEntry {
    command: String,
    args: Vec<String>,
    user: Option<String>,
    timestamp: std::time::SystemTime,
    success: bool,
    message: String,
}

/// MCP adapter implementation with authentication
#[derive(Debug)]
pub struct McpAdapter {
    adapter: RegistryAdapter,
    authorized_users: HashMap<String, String>, // username -> password
    user_roles: HashMap<String, Vec<UserRole>>, // username -> roles
    command_permissions: HashMap<String, Vec<UserRole>>, // command -> required roles
    active_tokens: HashMap<String, AuthUser>, // token -> user
    command_log: Vec<CommandLogEntry>, // audit log
}

impl McpAdapter {
    pub fn new() -> Self {
        let mut instance = Self {
            adapter: RegistryAdapter::new(),
            authorized_users: HashMap::new(),
            user_roles: HashMap::new(),
            command_permissions: HashMap::new(),
            active_tokens: HashMap::new(),
            command_log: Vec::new(),
        };
        
        // Add default admin user
        instance.add_user("admin", "password", true);
        
        instance
    }
    
    pub fn add_user(&mut self, username: &str, password: &str, is_admin: bool) {
        self.authorized_users.insert(username.to_string(), password.to_string());
        
        // Assign roles based on admin status
        let roles = if is_admin {
            vec![UserRole::Admin]
        } else {
            vec![UserRole::RegularUser]
        };
        self.user_roles.insert(username.to_string(), roles);
        
        if is_admin {
            // Mark admin commands as requiring admin role
            for cmd in ["admin-cmd"].iter() {
                self.command_permissions.insert(cmd.to_string(), vec![UserRole::Admin]);
            }
        }
    }
    
    /// Add a command with specific role requirements
    pub fn add_command_with_permissions(&mut self, command_name: &str, roles: Vec<UserRole>) {
        self.command_permissions.insert(command_name.to_string(), roles);
    }
    
    /// Generate an authentication token for a user
    pub fn generate_token(&mut self, username: &str, password: &str) -> CommandResult<String> {
        // Verify credentials
        if let Some(stored_password) = self.authorized_users.get(username) {
            if password != stored_password {
                return Err(CommandError::AuthenticationFailed(
                    format!("Invalid password for user '{}'", username)
                ));
            }
        } else {
            return Err(CommandError::AuthenticationFailed(
                format!("User '{}' not found", username)
            ));
        }
        
        // Create a simple token (in a real system, use proper JWT or similar)
        let token = format!("token-{}-{}", username, std::time::SystemTime::now().elapsed().unwrap().as_secs());
        
        // Store the token with user info
        let roles = self.user_roles.get(username).cloned().unwrap_or_default();
        self.active_tokens.insert(token.clone(), AuthUser {
            username: username.to_string(),
            roles,
            token: Some(token.clone()),
        });
        
        Ok(token)
    }
    
    /// Authenticate user based on provided auth credentials
    async fn authenticate(&self, auth: &Auth) -> CommandResult<Option<AuthUser>> {
        match auth {
            Auth::User(username, password) => {
                // Verify user credentials
                if let Some(stored_password) = self.authorized_users.get(username) {
                    if password != stored_password {
                        return Err(CommandError::AuthenticationFailed(
                            format!("Invalid password for user '{}'", username)
                        ));
                    }
                    
                    // Get user roles
                    let roles = self.user_roles.get(username).cloned().unwrap_or_default();
                    
                    Ok(Some(AuthUser {
                        username: username.clone(),
                        roles,
                        token: None,
                    }))
                } else {
                    Err(CommandError::AuthenticationFailed(
                        format!("User '{}' not found", username)
                    ))
                }
            },
            Auth::Token(token) => {
                if let Some(user) = self.active_tokens.get(token) {
                    Ok(Some(user.clone()))
                } else {
                    Err(CommandError::AuthenticationFailed(
                        "Invalid or expired token".to_string()
                    ))
                }
            },
            Auth::ApiKey(key) => {
                // Simple API key validation (could be more sophisticated)
                if key == "squirrel-api-key" {
                    Ok(Some(AuthUser {
                        username: "api".to_string(),
                        roles: vec![UserRole::PowerUser],
                        token: None,
                    }))
                } else {
                    Err(CommandError::AuthenticationFailed(
                        "Invalid API key".to_string()
                    ))
                }
            },
            Auth::None => Ok(None),
        }
    }
    
    /// Check if a user has permission to execute a command
    fn authorize(&self, command: &str, user: Option<&AuthUser>) -> CommandResult<()> {
        // Check if command requires specific permissions
        if let Some(required_roles) = self.command_permissions.get(command) {
            match user {
                Some(user) => {
                    // Check if user has any of the required roles
                    let has_permission = user.roles.iter().any(|role| {
                        // Admin always has permission
                        *role == UserRole::Admin || required_roles.contains(role)
                    });
                    
                    if !has_permission {
                        return Err(CommandError::AuthorizationFailed(
                            format!("User '{}' is not authorized to execute command '{}'", user.username, command)
                        ));
                    }
                },
                None => {
                    return Err(CommandError::AuthenticationFailed(
                        format!("Authentication required for command '{}'", command)
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Log command execution for audit purposes
    fn log_command(&mut self, command: &str, args: &[String], user: Option<&AuthUser>, success: bool, message: String) {
        self.command_log.push(CommandLogEntry {
            command: command.to_string(),
            args: args.to_vec(),
            user: user.map(|u| u.username.clone()),
            timestamp: std::time::SystemTime::now(),
            success,
            message,
        });
    }
    
    /// Get command execution logs
    pub fn get_command_logs(&self) -> &[CommandLogEntry] {
        &self.command_log
    }
    
    pub fn register_command(&self, command: Arc<dyn Command>) -> CommandResult<()> {
        let cmd_name = command.name();
        
        // For admin commands, automatically add admin permission
        if cmd_name.starts_with("admin") && !self.command_permissions.contains_key(cmd_name) {
            let mut permissions = self.command_permissions.clone();
            permissions.insert(cmd_name.to_string(), vec![UserRole::Admin]);
        }
        
        self.adapter.register(cmd_name, command)
    }
    
    pub async fn execute_with_auth(&mut self, command: &str, args: Vec<String>, auth: Auth) -> CommandResult<String> {
        // Authenticate user
        let user = self.authenticate(&auth).await?;
        
        // Authorize command execution
        self.authorize(command, user.as_ref())?;
        
        // Execute the command
        let result: CommandResult<String> = self.adapter.execute(command, args.clone()).await;
        
        // Log the command execution
        match &result {
            Ok(output) => {
                self.log_command(command, &args, user.as_ref(), true, output.clone());
            },
            Err(e) => {
                self.log_command(command, &args, user.as_ref(), false, e.to_string());
            }
        }
        
        result
    }
    
    pub async fn get_available_commands(&self, auth: Auth) -> CommandResult<Vec<String>> {
        let user = self.authenticate(&auth).await.ok().flatten();
        let mut commands = self.adapter.list_commands().await?;
        
        // Filter commands based on user permissions
        match user {
            Some(user) => {
                // If user is admin, return all commands
                if user.roles.contains(&UserRole::Admin) {
                    return Ok(commands);
                }
                
                // Otherwise, filter based on permissions
                commands.retain(|cmd| {
                    if let Some(required_roles) = self.command_permissions.get(cmd) {
                        user.roles.iter().any(|role| required_roles.contains(role))
                    } else {
                        true // No specific permission required
                    }
                });
            },
            None => {
                // Anonymous users can only access commands without permission requirements
                commands.retain(|cmd| !self.command_permissions.contains_key(cmd));
            }
        }
        
        Ok(commands)
    }
}

#[async_trait]
impl CommandAdapter for McpAdapter {
    async fn execute(&mut self, command: &str, args: Vec<String>) -> CommandResult<String> {
        // Use anonymous authentication for basic adapter trait
        self.execute_with_auth(command, args, Auth::None).await
    }
    
    async fn get_help(&self, command: &str) -> CommandResult<String> {
        <RegistryAdapter as CommandAdapter>::get_help(&self.adapter, command).await
    }
    
    async fn list_commands(&self) -> CommandResult<Vec<String>> {
        <RegistryAdapter as CommandAdapter>::list_commands(&self.adapter).await
    }
}

/// Plugin adapter implementation
#[derive(Debug)]
pub struct PluginAdapter {
    adapter: RegistryAdapter,
    plugin_id: String,
    version: String,
}

impl PluginAdapter {
    pub fn new() -> Self {
        Self {
            adapter: RegistryAdapter::new(),
            plugin_id: "commands".to_string(),
            version: "1.0.0".to_string(),
        }
    }
    
    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }
    
    pub fn version(&self) -> &str {
        &self.version
    }
    
    pub fn register_command(&self, command: Arc<dyn Command>) -> CommandResult<()> {
        self.adapter.register(command.name(), command)
    }
    
    pub async fn get_commands(&self) -> CommandResult<Vec<String>> {
        self.adapter.list_commands()
    }
}

#[async_trait]
impl CommandAdapter for PluginAdapter {
    async fn execute(&self, command: &str, args: Vec<String>) -> CommandResult<String> {
        <RegistryAdapter as CommandAdapter>::execute(&self.adapter, command, args).await
    }
    
    async fn get_help(&self, command: &str) -> CommandResult<String> {
        <RegistryAdapter as CommandAdapter>::get_help(&self.adapter, command).await
    }
    
    async fn list_commands(&self) -> CommandResult<Vec<String>> {
        <RegistryAdapter as CommandAdapter>::list_commands(&self.adapter).await
    }
}

/// MockAdapter trait for testing and example purposes
#[async_trait]
pub trait MockAdapter: Send + Sync {
    async fn execute(&self, command: &str, args: Vec<String>) -> CommandResult<String>;
    async fn get_help(&self, command: &str) -> CommandResult<String>;
    async fn list_commands(&self) -> CommandResult<Vec<String>>;
}

#[async_trait]
impl MockAdapter for RegistryAdapter {
    async fn execute(&self, command: &str, args: Vec<String>) -> CommandResult<String> {
        <RegistryAdapter as CommandAdapter>::execute(self, command, args).await
    }
    
    async fn get_help(&self, command: &str) -> CommandResult<String> {
        <RegistryAdapter as CommandAdapter>::get_help(self, command).await
    }
    
    async fn list_commands(&self) -> CommandResult<Vec<String>> {
        <RegistryAdapter as CommandAdapter>::list_commands(self).await
    }
}

#[async_trait]
impl MockAdapter for McpAdapter {
    async fn execute(&self, command: &str, args: Vec<String>) -> CommandResult<String> {
        <McpAdapter as CommandAdapter>::execute(self, command, args).await
    }
    
    async fn get_help(&self, command: &str) -> CommandResult<String> {
        <McpAdapter as CommandAdapter>::get_help(self, command).await
    }
    
    async fn list_commands(&self) -> CommandResult<Vec<String>> {
        <McpAdapter as CommandAdapter>::list_commands(self).await
    }
}

#[async_trait]
impl MockAdapter for PluginAdapter {
    async fn execute(&self, command: &str, args: Vec<String>) -> CommandResult<String> {
        <PluginAdapter as CommandAdapter>::execute(self, command, args).await
    }
    
    async fn get_help(&self, command: &str) -> CommandResult<String> {
        <PluginAdapter as CommandAdapter>::get_help(self, command).await
    }
    
    async fn list_commands(&self) -> CommandResult<Vec<String>> {
        <PluginAdapter as CommandAdapter>::list_commands(self).await
    }
}

/// Test the adapter with different implementations
///
/// This function demonstrates how the adapter pattern allows for polymorphic
/// usage of different adapter implementations through a common interface.
///
/// # Arguments
///
/// * `adapter` - An adapter implementation that conforms to the CommandAdapter trait
/// * `command` - The command to execute
/// * `args` - The arguments to pass to the command
///
/// # Returns
///
/// The result of the command execution
///
/// # Examples
///
/// ```
/// use adapter_pattern_tests::{RegistryAdapter, McpAdapter, CommandAdapter, CommandResult};
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() -> CommandResult<()> {
///    async fn test_adapter(adapter: &dyn CommandAdapter, cmd: &str) -> CommandResult<String> {
///        adapter.execute(cmd, vec![]).await
///    }
///
///    let registry_adapter = RegistryAdapter::new();
///    let mcp_adapter = McpAdapter::new();
///    
///    // Test polymorphic usage
///    let result1 = test_adapter(&registry_adapter, "hello").await;
///    let result2 = test_adapter(&mcp_adapter, "hello").await;
///    
///    Ok(())
/// }
/// ```
pub async fn test_polymorphic_adapter<A: CommandAdapter + ?Sized>(
    adapter: &A, 
    command: &str, 
    args: Vec<String>
) -> CommandResult<String> {
    adapter.execute(command, args).await
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
        
        adapter.register(hello_cmd.name(), Arc::new(hello_cmd))?;
        adapter.register(echo_cmd.name(), Arc::new(echo_cmd))?;
        
        // Test command execution without arguments
        let result = <RegistryAdapter as CommandAdapter>::execute(&adapter, "hello", vec![]).await?;
        assert_eq!(result, "Hello, world!");
        
        // Test command execution with arguments
        let result = <RegistryAdapter as CommandAdapter>::execute(&adapter, "echo", vec!["Hello".to_string(), "there!".to_string()]).await?;
        assert_eq!(result, "Echo with args: [\"Hello\", \"there!\"]");
        
        // Test help information
        let help = <RegistryAdapter as CommandAdapter>::get_help(&adapter, "hello").await?;
        assert_eq!(help, "hello: Says hello");
        
        // Test listing commands
        let commands = <RegistryAdapter as CommandAdapter>::list_commands(&adapter).await?;
        assert_eq!(commands.len(), 2);
        assert!(commands.contains(&"hello".to_string()));
        assert!(commands.contains(&"echo".to_string()));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_mcp_adapter_authentication() -> CommandResult<()> {
        // Create an MCP adapter
        let mut adapter = McpAdapter::new();
        
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
        match result {
            Err(CommandError::AuthenticationFailed(_)) => (),
            _ => panic!("Expected authentication failure"),
        }
        
        // Test access with invalid credentials
        let invalid_auth = Auth::User("admin".to_string(), "wrong_password".to_string());
        let result = adapter.execute_with_auth("secure", vec![], invalid_auth).await;
        assert!(result.is_err());
        match result {
            Err(CommandError::AuthenticationFailed(_)) => (),
            _ => panic!("Expected authentication failure"),
        }
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_plugin_adapter() -> CommandResult<()> {
        // Create a plugin adapter
        let adapter = PluginAdapter::new();
        
        // Verify plugin metadata
        assert_eq!(adapter.plugin_id(), "commands");
        assert_eq!(adapter.version(), "1.0.0");
        
        // Register commands
        let cmd = TestCommand::new("plugin-cmd", "Plugin command", "Plugin result");
        adapter.register_command(Arc::new(cmd))?;
        
        // List commands
        let commands = adapter.get_commands().await?;
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], "plugin-cmd");
        
        // Execute command
        let result = adapter.execute("plugin-cmd", vec!["arg1".to_string(), "arg2".to_string()]).await?;
        assert_eq!(result, "Plugin result with args: [\"arg1\", \"arg2\"]");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_adapter_trait() -> CommandResult<()> {
        // Test using adapter trait with different implementations
        async fn test_adapter(adapter: &dyn CommandAdapter, cmd_name: &str) -> CommandResult<String> {
            adapter.execute(cmd_name, vec![]).await
        }
        
        let registry_adapter = RegistryAdapter::new();
        let mcp_adapter = McpAdapter::new();
        let plugin_adapter = PluginAdapter::new();
        
        // Register the same command in all adapters
        let test_cmd = TestCommand::new("test", "Test command", "Test result");
        registry_adapter.register(test_cmd.name(), Arc::new(test_cmd.clone()))?;
        mcp_adapter.register_command(Arc::new(test_cmd.clone()))?;
        plugin_adapter.register_command(Arc::new(test_cmd))?;
        
        // Test execution through trait
        let result1 = test_adapter(&registry_adapter, "test").await?;
        let result2 = test_adapter(&mcp_adapter, "test").await?;
        let result3 = test_adapter(&plugin_adapter, "test").await?;
        
        assert_eq!(result1, "Test result");
        assert_eq!(result2, "Test result");
        assert_eq!(result3, "Test result");
        
        Ok(())
    }
}
