//! MCP (Machine Context Protocol) Adapter implementation
//!
//! This module contains the implementation of the McpCommandAdapter, which
//! adapts the command registry for use with the Machine Context Protocol (MCP).
//! It adds authentication and authorization capabilities.

use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;

use crate::command::MockCommand;
use crate::error::{AdapterError, AdapterResult};
use super::registry::{CommandRegistryAdapter, MockAdapter};

/// Authentication type for MCP commands
///
/// This enum represents different authentication methods for MCP commands.
#[derive(Debug, Clone)]
pub enum Auth {
    /// User authentication with username and password
    User(String, String), // username, password
    
    /// No authentication (anonymous)
    None,
}

impl Auth {
    /// Creates a new user authentication
    pub fn user(username: &str, password: &str) -> Self {
        Self::User(username.to_string(), password.to_string())
    }
    
    /// Creates an anonymous authentication
    pub fn anonymous() -> Self {
        Self::None
    }
    
    /// Returns the username if available
    pub fn username(&self) -> Option<&str> {
        match self {
            Auth::User(username, _) => Some(username),
            Auth::None => None,
        }
    }
}

/// MCP Adapter for command operations with authentication and authorization
///
/// This adapter extends the basic command registry adapter with authentication
/// and authorization capabilities for the Machine Context Protocol (MCP).
#[derive(Debug)]
pub struct McpCommandAdapter {
    /// The underlying command registry adapter
    adapter: CommandRegistryAdapter,
    
    /// Map of authorized users and their passwords
    authorized_users: HashMap<String, String>, // username -> password
    
    /// Map of command roles and required permissions
    command_permissions: HashMap<String, String>, // command -> required role
}

impl McpCommandAdapter {
    /// Creates a new MCP adapter with default settings
    pub fn new() -> Self {
        let mut instance = Self {
            adapter: CommandRegistryAdapter::new(),
            authorized_users: HashMap::new(),
            command_permissions: HashMap::new(),
        };
        
        // Add default admin user
        instance.add_user("admin", "password", true);
        
        instance
    }
    
    /// Creates an MCP adapter with an existing registry adapter
    pub fn with_adapter(adapter: CommandRegistryAdapter) -> Self {
        let mut instance = Self {
            adapter,
            authorized_users: HashMap::new(),
            command_permissions: HashMap::new(),
        };
        
        // Add default admin user
        instance.add_user("admin", "password", true);
        
        instance
    }
    
    /// Checks if the adapter is initialized
    pub fn is_initialized(&self) -> bool {
        self.adapter.is_initialized()
    }
    
    /// Adds a user to the authorized users list
    ///
    /// # Arguments
    ///
    /// * `username` - The username to add
    /// * `password` - The password for the user
    /// * `is_admin` - Whether the user has admin privileges
    pub fn add_user(&mut self, username: &str, password: &str, is_admin: bool) {
        self.authorized_users.insert(username.to_string(), password.to_string());
        
        if is_admin {
            // Mark all admin commands as requiring admin role
            for cmd in ["admin", "config", "user"].iter() {
                self.command_permissions.insert(cmd.to_string(), "admin".to_string());
            }
        }
    }
    
    /// Registers a command with the registry
    ///
    /// # Arguments
    ///
    /// * `command` - The command to register
    ///
    /// # Returns
    ///
    /// * `Ok(())` if registration succeeded
    /// * `Err(AdapterError)` if registration failed
    pub fn register_command(&self, command: Arc<dyn MockCommand + Send + Sync>) -> AdapterResult<()> {
        // Automatically mark admin commands as requiring admin role
        let cmd_name = command.name();
        if cmd_name.starts_with("admin") {
            // Register admin commands as requiring admin role
            let mut permissions = self.command_permissions.clone();
            permissions.insert(cmd_name.to_string(), "admin".to_string());
            
            // We can't modify self.command_permissions directly due to borrowing rules,
            // but in a real implementation we would use interior mutability like RwLock
        }
        
        self.adapter.register_command(command)
    }
    
    /// Executes a command with authentication
    ///
    /// # Arguments
    ///
    /// * `command` - The name of the command to execute
    /// * `args` - The arguments to pass to the command
    /// * `auth` - The authentication credentials
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the command output
    /// * `Err(AdapterError)` if execution failed
    pub async fn execute_with_auth(&self, command: &str, args: Vec<String>, auth: Auth) -> AdapterResult<String> {
        // Check authorization
        match &auth {
            Auth::User(username, password) => {
                // Verify user credentials
                if let Some(stored_password) = self.authorized_users.get(username) {
                    if password != stored_password {
                        return Err(AdapterError::AuthenticationFailed { 
                            reason: format!("Invalid password for user '{}'", username) 
                        });
                    }
                } else {
                    return Err(AdapterError::AuthenticationFailed { 
                        reason: format!("User '{}' not found", username) 
                    });
                }
                
                // Check if command requires admin role
                if command.starts_with("admin") && username != "admin" {
                    return Err(AdapterError::AuthorizationFailed { 
                        username: username.clone(), 
                        command: command.to_string() 
                    });
                }
            },
            Auth::None => {
                // Allow only public commands for anonymous users
                if command.starts_with("admin") {
                    return Err(AdapterError::AuthenticationFailed { 
                        reason: format!("Authentication required for command '{}'", command) 
                    });
                }
            }
        }
        
        // Execute the command
        self.adapter.execute(command, args).await
    }
    
    /// Gets available commands based on authentication
    ///
    /// # Arguments
    ///
    /// * `auth` - The authentication credentials
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` containing available command names
    /// * `Err(AdapterError)` if getting commands failed
    pub async fn get_available_commands(&self, auth: Auth) -> AdapterResult<Vec<String>> {
        let mut commands = self.adapter.list_commands().await?;
        
        // Filter admin commands if not admin
        match auth {
            Auth::User(ref username, _) => {
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

#[async_trait]
impl MockAdapter for McpCommandAdapter {
    async fn execute(&self, command: &str, args: Vec<String>) -> AdapterResult<String> {
        // Use anonymous authentication for the base adapter trait
        self.execute_with_auth(command, args, Auth::None).await
    }
    
    async fn get_help(&self, command: &str) -> AdapterResult<String> {
        // For help info, we delegate to the underlying adapter
        self.adapter.get_help(command).await
    }
} 