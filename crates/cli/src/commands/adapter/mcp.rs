use std::sync::Arc;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use async_trait::async_trait;
use log::debug;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::commands::adapter::error::{AdapterError, AdapterResult};
use crate::commands::adapter::{CommandRegistryAdapter, CommandAdapterTrait};

/// Authentication types
#[derive(Debug, Clone)]
pub enum Auth {
    /// Username/password authentication
    User(String, String),
    
    /// Token-based authentication
    Token(String),
    
    /// API key authentication
    ApiKey(String),
    
    /// No authentication
    None,
}

/// User roles for authorization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    /// Administrator role with full access
    Admin,
    
    /// Power user role with extended privileges
    PowerUser,
    
    /// Regular user role with standard access
    RegularUser,
    
    /// Guest role with limited access
    Guest,
}

/// Authentication provider interface
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Authenticate a user using the provided auth information
    async fn authenticate(&self, auth: &Auth) -> AdapterResult<Option<String>>;
    
    /// Check if a user is authorized to execute a command
    async fn authorize(&self, command: &str, username: Option<&str>) -> AdapterResult<bool>;
}

/// Basic username/password authentication provider
#[derive(Debug, Clone)]
pub struct BasicAuthProvider {
    users: HashMap<String, String>,
    command_permissions: HashMap<String, Vec<UserRole>>,
    user_roles: HashMap<String, UserRole>,
}

impl BasicAuthProvider {
    /// Create a new basic auth provider
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            command_permissions: HashMap::new(),
            user_roles: HashMap::new(),
        }
    }
    
    /// Add a user to the provider
    pub fn add_user(&mut self, username: &str, password: &str, role: UserRole) -> &mut Self {
        self.users.insert(username.to_string(), password.to_string());
        self.user_roles.insert(username.to_string(), role);
        self
    }
    
    /// Add command permission
    pub fn add_command_permission(&mut self, command: &str, allowed_roles: Vec<UserRole>) -> &mut Self {
        self.command_permissions.insert(command.to_string(), allowed_roles);
        self
    }
}

impl Default for BasicAuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthProvider for BasicAuthProvider {
    async fn authenticate(&self, auth: &Auth) -> AdapterResult<Option<String>> {
        match auth {
            Auth::User(username, password) => {
                if let Some(stored_password) = self.users.get(username) {
                    if stored_password == password {
                        return Ok(Some(username.clone()));
                    }
                }
                Err(AdapterError::AuthenticationFailed("Invalid username or password".to_string()))
            },
            Auth::Token(_) => Err(AdapterError::AuthenticationFailed("Token authentication not supported by BasicAuthProvider".to_string())),
            Auth::ApiKey(_) => Err(AdapterError::AuthenticationFailed("API key authentication not supported by BasicAuthProvider".to_string())),
            Auth::None => Ok(None),
        }
    }
    
    async fn authorize(&self, command: &str, username: Option<&str>) -> AdapterResult<bool> {
        if let Some(username) = username {
            if let Some(user_role) = self.user_roles.get(username) {
                if let Some(allowed_roles) = self.command_permissions.get(command) {
                    let is_authorized = allowed_roles.contains(user_role);
                    if !is_authorized {
                        return Err(AdapterError::AuthorizationFailed(
                            format!("User '{}' with role {:?} is not authorized to execute command '{}'. Required roles: {:?}", 
                                    username, user_role, command, allowed_roles)
                        ));
                    }
                    return Ok(true);
                } else {
                    // If no specific permissions are defined, allow by default
                    return Ok(true);
                }
            }
            Err(AdapterError::AuthorizationFailed(format!("User '{}' not found", username)))
        } else {
            // If no username, check if command allows anonymous access
            if let Some(allowed_roles) = self.command_permissions.get(command) {
                if !allowed_roles.contains(&UserRole::Guest) {
                    return Err(AdapterError::AuthorizationFailed(
                        format!("Anonymous access is not allowed for command '{}'. Required roles: {:?}", 
                                command, allowed_roles)
                    ));
                }
                Ok(true)
            } else {
                // If no specific permissions are defined, allow by default for public commands
                Ok(true)
            }
        }
    }
}

/// Token-based authentication provider
#[derive(Debug, Clone)]
pub struct TokenAuthProvider {
    tokens: HashMap<String, String>, // token -> username
}

impl TokenAuthProvider {
    /// Create a new token authentication provider
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
        }
    }
    
    /// Add a token for a user
    pub fn add_token(&mut self, token: &str, username: &str) -> &mut Self {
        self.tokens.insert(token.to_string(), username.to_string());
        self
    }
}

impl Default for TokenAuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthProvider for TokenAuthProvider {
    async fn authenticate(&self, auth: &Auth) -> AdapterResult<Option<String>> {
        match auth {
            Auth::Token(token) => {
                if let Some(username) = self.tokens.get(token) {
                    return Ok(Some(username.clone()));
                }
                Err(AdapterError::AuthenticationFailed("Invalid token".to_string()))
            },
            Auth::User(_, _) => Err(AdapterError::AuthenticationFailed("Username/password authentication not supported by TokenAuthProvider".to_string())),
            Auth::ApiKey(_) => Err(AdapterError::AuthenticationFailed("API key authentication not supported by TokenAuthProvider".to_string())),
            Auth::None => Ok(None),
        }
    }
    
    async fn authorize(&self, _command: &str, username: Option<&str>) -> AdapterResult<bool> {
        // Token authentication only checks if the user is authenticated, not specific permissions
        Ok(username.is_some())
    }
}

/// API key authentication provider
#[derive(Debug, Clone)]
pub struct ApiKeyAuthProvider {
    api_keys: HashMap<String, String>, // api_key -> username
}

impl ApiKeyAuthProvider {
    /// Create a new API key authentication provider
    pub fn new() -> Self {
        Self {
            api_keys: HashMap::new(),
        }
    }
    
    /// Add an API key for a user
    pub fn add_api_key(&mut self, api_key: &str, username: &str) -> &mut Self {
        self.api_keys.insert(api_key.to_string(), username.to_string());
        self
    }
}

impl Default for ApiKeyAuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthProvider for ApiKeyAuthProvider {
    async fn authenticate(&self, auth: &Auth) -> AdapterResult<Option<String>> {
        match auth {
            Auth::ApiKey(key) => {
                if let Some(username) = self.api_keys.get(key) {
                    return Ok(Some(username.clone()));
                }
                Err(AdapterError::AuthenticationFailed("Invalid API key".to_string()))
            },
            Auth::User(_, _) => Err(AdapterError::AuthenticationFailed("Username/password authentication not supported by ApiKeyAuthProvider".to_string())),
            Auth::Token(_) => Err(AdapterError::AuthenticationFailed("Token authentication not supported by ApiKeyAuthProvider".to_string())),
            Auth::None => Ok(None),
        }
    }
    
    async fn authorize(&self, _command: &str, username: Option<&str>) -> AdapterResult<bool> {
        // API key authentication only checks if the user is authenticated, not specific permissions
        Ok(username.is_some())
    }
}

/// Command log entry for audit logging
#[derive(Debug, Clone)]
pub struct CommandLogEntry {
    /// Command name
    pub command: String,
    
    /// Command arguments
    pub args: Vec<String>,
    
    /// Username (if authenticated)
    pub user: Option<String>,
    
    /// Timestamp of execution
    pub timestamp: SystemTime,
    
    /// Whether the command succeeded
    pub success: bool,
    
    /// Command output or error message
    pub message: String,
}

/// MCP command adapter with authentication and authorization
pub struct McpCommandAdapter {
    registry_adapter: Arc<CommandRegistryAdapter>,
    auth_provider: Arc<dyn AuthProvider + 'static>,
    command_log: Mutex<Vec<CommandLogEntry>>,
}

impl std::fmt::Debug for McpCommandAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpCommandAdapter")
            .field("registry_adapter", &"<CommandRegistryAdapter>")
            .field("auth_provider", &"<dyn AuthProvider>")
            .field("command_log", &self.command_log)
            .finish()
    }
}

/// Authentication credentials for MCP commands
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AuthCredentials {
    /// No authentication
    None,
    
    /// Basic username/password authentication
    Basic {
        username: String,
        password: String,
    },
    
    /// Token-based authentication
    Token(String),
    
    /// API key authentication
    ApiKey(String),
}

/// MCP user information
#[derive(Debug, Clone)]
pub struct User {
    /// User ID
    pub id: String,
    
    /// User name
    pub username: String,
    
    /// User roles
    pub roles: Vec<String>,
    
    /// Custom user attributes
    pub attributes: std::collections::HashMap<String, String>,
}

impl User {
    pub fn new(username: String) -> Self {
        Self {
            id: username.clone(),
            username,
            roles: Vec::new(),
            attributes: std::collections::HashMap::new(),
        }
    }
}

/// MCP execution context
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct McpExecutionContext {
    /// Working directory for command execution
    pub working_directory: Option<String>,
    
    /// Environment variables
    pub environment: Option<std::collections::HashMap<String, String>>,
    
    /// Session ID
    pub session_id: Option<String>,
    
    /// Timestamp
    pub timestamp: Option<i64>,
}

/// MCP command request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpCommandRequest {
    /// Command to execute
    pub command: String,
    
    /// Command arguments
    pub arguments: Vec<String>,
    
    /// Authentication credentials
    pub credentials: Option<AuthCredentials>,
    
    /// Execution context
    pub context: McpExecutionContext,
}

/// MCP command response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpCommandResponse {
    /// Whether the command succeeded
    pub success: bool,
    
    /// Command output if successful
    pub output: Option<String>,
    
    /// Error message if failed
    pub error: Option<String>,
    
    /// Duration of command execution in milliseconds
    pub duration_ms: Option<u64>,
    
    /// Timestamp of response
    pub timestamp: i64,
}

/// Result type for MCP operations
pub type McpResult<T> = Result<T, AdapterError>;

/// Authentication manager with AuthProvider implementation
pub struct AuthManager {
    auth_provider: Arc<dyn AuthProvider>,
}

impl std::fmt::Debug for AuthManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthManager")
            .field("auth_provider", &"<dyn AuthProvider>")
            .finish()
    }
}

impl AuthManager {
    /// Create a new auth manager with the given provider
    pub fn with_provider(provider: Arc<dyn AuthProvider>) -> Self {
        Self {
            auth_provider: provider,
        }
    }
}

#[async_trait]
impl AuthProvider for AuthManager {
    async fn authenticate(&self, auth: &Auth) -> AdapterResult<Option<String>> {
        self.auth_provider.authenticate(auth).await
    }
    
    async fn authorize(&self, command: &str, username: Option<&str>) -> AdapterResult<bool> {
        self.auth_provider.authorize(command, username).await
    }
}

impl McpCommandAdapter {
    /// Create a new McpCommandAdapter
    pub fn new(
        auth_provider: Arc<dyn AuthProvider + 'static>,
        registry_adapter: Arc<CommandRegistryAdapter>
    ) -> Self {
        Self {
            registry_adapter,
            auth_provider,
            command_log: Mutex::new(Vec::new()),
        }
    }
    
    /// Convert AuthCredentials to Auth
    fn credentials_to_auth(credentials: &AuthCredentials) -> Auth {
        match credentials {
            AuthCredentials::None => Auth::None,
            AuthCredentials::Basic { username, password } => Auth::User(username.clone(), password.clone()),
            AuthCredentials::Token(token) => Auth::Token(token.clone()),
            AuthCredentials::ApiKey(key) => Auth::ApiKey(key.clone()),
        }
    }
    
    /// Authenticate a user with the given credentials
    async fn authenticate(&self, credentials: &AuthCredentials) -> AdapterResult<Option<String>> {
        let auth = Self::credentials_to_auth(credentials);
        self.auth_provider.authenticate(&auth).await
    }
    
    /// Add a command execution to the log
    async fn log_command(&self, cmd: &str, args: &[String], user: Option<&str>, success: bool, message: &str) {
        let mut log = self.command_log.lock().await;
        log.push(CommandLogEntry {
            command: cmd.to_string(),
            args: args.to_vec(),
            user: user.map(|u| u.to_string()),
            timestamp: SystemTime::now(),
            success,
            message: message.to_string(),
        });
        
        // Trim log if it gets too large
        if log.len() > 1000 {
            log.drain(0..500);
        }
    }

    /// Handle an MCP command request
    pub async fn handle_command(&self, request: &McpCommandRequest) -> McpCommandResponse {
        let start = SystemTime::now();
        let mut user = None;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // Handle authentication if credentials are provided
        if let Some(ref credentials) = request.credentials {
            match self.authenticate(credentials).await {
                Ok(Some(username)) => {
                    debug!("User '{}' authenticated successfully", username);
                    user = Some(username);
                }
                Ok(None) => {
                    let error_msg = "Authentication failed: Invalid credentials".to_string();
                    debug!("{}", error_msg);
                    self.log_command(
                        &request.command,
                        &request.arguments,
                        None,
                        false,
                        &error_msg,
                    ).await;

                    return McpCommandResponse {
                        success: false,
                        output: None,
                        error: Some(error_msg),
                        duration_ms: Some(
                            SystemTime::now()
                                .duration_since(start)
                                .map(|d| d.as_millis() as u64)
                                .unwrap_or(0),
                        ),
                        timestamp,
                    };
                }
                Err(e) => {
                    let error_msg = format!("Authentication error: {}", e);
                    debug!("{}", error_msg);
                    self.log_command(
                        &request.command,
                        &request.arguments,
                        None,
                        false,
                        &error_msg,
                    ).await;

                    return McpCommandResponse {
                        success: false,
                        output: None,
                        error: Some(error_msg),
                        duration_ms: Some(
                            SystemTime::now()
                                .duration_since(start)
                                .map(|d| d.as_millis() as u64)
                                .unwrap_or(0),
                        ),
                        timestamp,
                    };
                }
            }
        }

        // Check authorization for the command
        if let Err(e) = self.auth_provider.authorize(&request.command, user.as_deref()).await {
            let error_msg = format!("Authorization error: {}", e);
            debug!("{}", error_msg);
            self.log_command(
                &request.command,
                &request.arguments,
                user.as_deref(),
                false,
                &error_msg,
            ).await;

            return McpCommandResponse {
                success: false,
                output: None,
                error: Some(error_msg),
                duration_ms: Some(
                    SystemTime::now()
                        .duration_since(start)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0),
                ),
                timestamp,
            };
        }

        // Execute the command
        let result = self.registry_adapter
            .execute_command(&request.command, request.arguments.clone())
            .await;

        let duration = SystemTime::now()
            .duration_since(start)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        match result {
            Ok(output) => {
                debug!("Command '{}' executed successfully", request.command);
                self.log_command(
                    &request.command,
                    &request.arguments,
                    user.as_deref(),
                    true,
                    &output,
                ).await;

                McpCommandResponse {
                    success: true,
                    output: Some(output),
                    error: None,
                    duration_ms: Some(duration),
                    timestamp,
                }
            }
            Err(e) => {
                let error_msg = format!("Command execution error: {}", e);
                debug!("{}", error_msg);
                self.log_command(
                    &request.command,
                    &request.arguments,
                    user.as_deref(),
                    false,
                    &error_msg,
                ).await;

                McpCommandResponse {
                    success: false,
                    output: None,
                    error: Some(error_msg),
                    duration_ms: Some(duration),
                    timestamp,
                }
            }
        }
    }
}

#[async_trait]
impl CommandAdapterTrait for McpCommandAdapter {
    async fn execute_command(&self, command: &str, args: Vec<String>) -> AdapterResult<String> {
        debug!("Executing command via MCP adapter: {} with args: {:?}", command, args);
        
        // Execute the command through the registry adapter
        match self.registry_adapter.execute_command(command, args.clone()).await {
            Ok(result) => {
                self.log_command(command, &args, None, true, &result).await;
                Ok(result)
            },
            Err(e) => {
                self.log_command(command, &args, None, false, &e.to_string()).await;
                Err(e)
            }
        }
    }
    
    async fn get_help(&self, command: &str) -> AdapterResult<String> {
        debug!("Getting help for command via MCP adapter: {}", command);
        self.registry_adapter.get_help(command).await
    }
    
    async fn list_commands(&self) -> AdapterResult<Vec<String>> {
        debug!("Listing commands via MCP adapter");
        self.registry_adapter.list_commands().await
    }
}

/// McpCommandAdapter with authentication for APIs
impl McpCommandAdapter {
    /// Execute a command with authentication
    pub async fn execute_authenticated(
        &self,
        command: &str,
        args: Vec<String>,
        credentials: &AuthCredentials
    ) -> AdapterResult<String> {
        let username = self.authenticate(credentials).await?;
        // Check authorization
        if let Some(username_str) = username.as_deref() {
            if !self.auth_provider.authorize(command, Some(username_str)).await? {
                let error = AdapterError::AuthorizationFailed(format!("User '{}' not authorized to execute command '{}'", username_str, command));
                self.log_command(command, &args, Some(username_str), false, &error.to_string()).await;
                return Err(error);
            }
        }
        
        // Execute command
        match self.registry_adapter.execute_command(command, args.clone()).await {
            Ok(result) => {
                self.log_command(command, &args, username.as_deref(), true, &result).await;
                Ok(result)
            }
            Err(error) => {
                self.log_command(command, &args, username.as_deref(), false, &error.to_string()).await;
                Err(error)
            }
        }
    }
    
    /// Get the command execution log
    pub async fn get_log(&self) -> AdapterResult<Vec<CommandLogEntry>> {
        let log = self.command_log.lock().await;
        Ok(log.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::adapter::registry::CommandRegistryAdapter;
    use commands::{Command, CommandRegistry, CommandResult};
    use clap::Command as ClapCommand;
    
    #[derive(Debug, Clone)]
    struct TestCommand;
    
    impl Command for TestCommand {
        fn name(&self) -> &str {
            "test_command"
        }
        
        fn description(&self) -> &str {
            "Test command for unit tests"
        }
        
        fn parser(&self) -> ClapCommand {
            ClapCommand::new("test_command")
                .about("Test command for unit tests")
        }
        
        fn execute(&self, args: &[String]) -> CommandResult<String> {
            if args.is_empty() {
                Ok("Test command executed successfully".to_string())
            } else {
                Ok(format!("Test command executed with args: {:?}", args))
            }
        }
        
        fn clone_box(&self) -> Box<dyn Command> {
            Box::new(self.clone())
        }
    }
    
    #[derive(Debug, Clone)]
    struct AdminCommand;
    
    impl Command for AdminCommand {
        fn name(&self) -> &str {
            "admin"
        }
        
        fn description(&self) -> &str {
            "Admin command requiring special permissions"
        }
        
        fn parser(&self) -> ClapCommand {
            ClapCommand::new("admin")
                .about("Admin command requiring special permissions")
        }
        
        fn execute(&self, _args: &[String]) -> CommandResult<String> {
            Ok("Admin command executed successfully".to_string())
        }
        
        fn clone_box(&self) -> Box<dyn Command> {
            Box::new(self.clone())
        }
    }
    
    #[tokio::test]
    async fn test_mcp_command_adapter() {
        // Create registry and register commands
        let registry = Arc::new(Mutex::new(CommandRegistry::new()));
        let registry_adapter = Arc::new(CommandRegistryAdapter::new(registry));
        
        // Use string literals for command names
        registry_adapter.register_command("test_command", Arc::new(TestCommand)).await.unwrap();
        registry_adapter.register_command("admin", Arc::new(AdminCommand)).await.unwrap();
        
        // Create auth provider and manager with proper command permissions
        let mut auth_provider = BasicAuthProvider::new();
        auth_provider.add_user("testuser", "testpass", UserRole::RegularUser)
            // Allow all user types including Guest for test_command
            .add_command_permission("test_command", vec![UserRole::Guest, UserRole::RegularUser, UserRole::PowerUser, UserRole::Admin])
            // Only allow admin for admin command
            .add_command_permission("admin", vec![UserRole::Admin]);
        
        let auth_manager = Arc::new(AuthManager::with_provider(Arc::new(auth_provider)));
        
        // Create MCP adapter
        let mcp_adapter = McpCommandAdapter::new(auth_manager, registry_adapter);
        
        // Test unauthenticated execution
        let request = McpCommandRequest {
            command: "test_command".to_string(),
            arguments: vec![],
            credentials: None,
            context: McpExecutionContext::default(),
        };
        
        let response = mcp_adapter.handle_command(&request).await;
        assert!(response.success, "Unauthenticated execution failed: {:?}", response.error);
        assert_eq!(response.output, Some("Test command executed successfully".to_string()));
        
        // Test authenticated execution
        let request = McpCommandRequest {
            command: "test_command".to_string(),
            arguments: vec!["arg1".to_string(), "arg2".to_string()],
            credentials: Some(AuthCredentials::Basic {
                username: "testuser".to_string(),
                password: "testpass".to_string(),
            }),
            context: McpExecutionContext::default(),
        };
        
        let response = mcp_adapter.handle_command(&request).await;
        assert!(response.success, "Authenticated execution failed: {:?}", response.error);
        assert_eq!(response.output, Some("Test command executed with args: [\"arg1\", \"arg2\"]".to_string()));
        
        // Test unauthorized execution
        let request = McpCommandRequest {
            command: "admin".to_string(),
            arguments: vec![],
            credentials: Some(AuthCredentials::Basic {
                username: "testuser".to_string(),
                password: "testpass".to_string(),
            }),
            context: McpExecutionContext::default(),
        };
        
        let response = mcp_adapter.handle_command(&request).await;
        assert!(!response.success, "Unauthorized execution unexpectedly succeeded");
        let error_msg = response.error.clone().unwrap();
        assert!(error_msg.contains("not authorized"), "Unexpected error message: {:?}", response.error);
        
        // Test nonexistent command
        let request = McpCommandRequest {
            command: "nonexistent".to_string(),
            arguments: vec![],
            credentials: None,
            context: McpExecutionContext::default(),
        };
        
        let response = mcp_adapter.handle_command(&request).await;
        assert!(!response.success, "Nonexistent command execution unexpectedly succeeded");
        let error_msg = response.error.clone().unwrap();
        assert!(error_msg.contains("not found"), "Unexpected error message: {:?}", response.error);
    }
} 