//! Command execution handlers for the task service.
//!
//! This module provides command execution capabilities for the task service,
//! allowing tasks to execute commands and retrieve command information.

use std::collections::HashMap;
use std::sync::Arc;
use serde_json;
use tonic::Status;
use tracing::{debug, info, warn, error};
use tokio::sync::RwLock;

use super::types::SimpleCommand;
use super::service::TaskServiceImpl;
use crate::error::production::{ProductionError, SafeOperation};

/// Production command registry
#[derive(Debug)]
pub struct ProductionCommandRegistry {
    /// Registered commands
    commands: Arc<RwLock<HashMap<String, Box<dyn SimpleCommand>>>>,
    /// Command execution statistics
    execution_stats: Arc<RwLock<HashMap<String, CommandStats>>>,
}

/// Command execution statistics
#[derive(Debug, Clone)]
pub struct CommandStats {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_duration_ms: f64,
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for CommandStats {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            average_duration_ms: 0.0,
            last_execution: None,
        }
    }
}

impl ProductionCommandRegistry {
    /// Create a new production command registry
    pub fn new() -> Self {
        Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
            execution_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a command
    pub async fn register_command(&self, command: Box<dyn SimpleCommand>) -> Result<(), ProductionError> {
        let command_name = command.name().to_string();
        
        let commands_result = SafeOperation::execute(|| {
            self.commands.try_write()
                .map_err(|e| ProductionError::concurrency(
                    format!("Failed to acquire command registry write lock: {}", e),
                    "command_registration",
                    true,
                ))
        });

        match commands_result.result() {
            Ok(mut commands) => {
                commands.insert(command_name.clone(), command);
                info!("Command registered: {}", command_name);
                Ok(())
            }
            Err(e) => {
                error!("Failed to register command '{}': {}", command_name, e);
                Err(e)
            }
        }
    }

    /// List all available commands
    pub async fn list_commands(&self) -> Result<Vec<String>, ProductionError> {
        let commands_result = SafeOperation::execute(|| {
            self.commands.try_read()
                .map_err(|e| ProductionError::concurrency(
                    format!("Failed to acquire command registry read lock: {}", e),
                    "command_listing",
                    true,
                ))
        });

        match commands_result.result() {
            Ok(commands) => {
                let command_names: Vec<String> = commands.keys().cloned().collect();
                debug!("Listed {} commands", command_names.len());
                Ok(command_names)
            }
            Err(e) => {
                error!("Failed to list commands: {}", e);
                Err(e)
            }
        }
    }

    /// Execute a command
    pub async fn execute_command(&self, command_name: &str, args: Vec<String>) -> Result<String, ProductionError> {
        let start_time = std::time::Instant::now();
        
        // Get the command
        let command = {
            let commands_result = SafeOperation::execute(|| {
                self.commands.try_read()
                    .map_err(|e| ProductionError::concurrency(
                        format!("Failed to acquire command registry read lock: {}", e),
                        "command_execution",
                        true,
                    ))
            });

            match commands_result.result() {
                Ok(commands) => {
                    commands.get(command_name).map(|cmd| cmd.clone_box())
                }
                Err(e) => {
                    error!("Failed to access command registry for '{}': {}", command_name, e);
                    return Err(e);
                }
            }
        };

        let command = command.ok_or_else(|| {
            ProductionError::not_found(
                format!("Command '{}' not found", command_name),
                "command_execution",
                Some(format!("Available commands: {}", self.list_commands().await.unwrap_or_default().join(", ")))
            )
        })?;

        // Execute the command
        let result = SafeOperation::execute(|| {
            command.execute(&args)
                .map_err(|e| ProductionError::execution(
                    format!("Command '{}' execution failed: {}", command_name, e),
                    "command_execution",
                    true,
                ))
        });

        let execution_time = start_time.elapsed();
        
        // Update statistics
        self.update_command_stats(command_name, execution_time, result.is_ok()).await;

        match result.result() {
            Ok(output) => {
                info!("Command '{}' executed successfully in {:?}", command_name, execution_time);
                Ok(output)
            }
            Err(e) => {
                error!("Command '{}' execution failed: {}", command_name, e);
                Err(e)
            }
        }
    }

    /// Get help for a command
    pub async fn get_command_help(&self, command_name: &str) -> Result<String, ProductionError> {
        let commands_result = SafeOperation::execute(|| {
            self.commands.try_read()
                .map_err(|e| ProductionError::concurrency(
                    format!("Failed to acquire command registry read lock: {}", e),
                    "command_help",
                    true,
                ))
        });

        match commands_result.result() {
            Ok(commands) => {
                if let Some(command) = commands.get(command_name) {
                    Ok(command.help())
                } else {
                    Err(ProductionError::not_found(
                        format!("Command '{}' not found", command_name),
                        "command_help",
                        Some(format!("Available commands: {}", commands.keys().cloned().collect::<Vec<_>>().join(", ")))
                    ))
                }
            }
            Err(e) => {
                error!("Failed to get help for command '{}': {}", command_name, e);
                Err(e)
            }
        }
    }

    /// Update command execution statistics
    async fn update_command_stats(&self, command_name: &str, execution_time: std::time::Duration, success: bool) {
        let stats_result = SafeOperation::execute(|| {
            self.execution_stats.try_write()
                .map_err(|e| ProductionError::concurrency(
                    format!("Failed to acquire stats write lock: {}", e),
                    "stats_update",
                    false, // Not retryable - stats are not critical
                ))
        });

        if let Ok(mut stats) = stats_result.result() {
            let command_stats = stats.entry(command_name.to_string()).or_default();
            
            command_stats.total_executions += 1;
            if success {
                command_stats.successful_executions += 1;
            } else {
                command_stats.failed_executions += 1;
            }
            
            let duration_ms = execution_time.as_millis() as f64;
            command_stats.average_duration_ms = (command_stats.average_duration_ms * (command_stats.total_executions - 1) as f64 + duration_ms) / command_stats.total_executions as f64;
            command_stats.last_execution = Some(chrono::Utc::now());
        }
    }

    /// Get command statistics
    pub async fn get_command_stats(&self, command_name: &str) -> Option<CommandStats> {
        let stats_result = SafeOperation::execute(|| {
            self.execution_stats.try_read()
                .map_err(|e| ProductionError::concurrency(
                    format!("Failed to acquire stats read lock: {}", e),
                    "stats_read",
                    false,
                ))
        });

        if let Ok(stats) = stats_result.result() {
            stats.get(command_name).cloned()
        } else {
            None
        }
    }
}

impl Default for ProductionCommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Global command registry instance
static COMMAND_REGISTRY: std::sync::OnceLock<ProductionCommandRegistry> = std::sync::OnceLock::new();

/// Get the global command registry instance
pub fn get_command_registry() -> &'static ProductionCommandRegistry {
    COMMAND_REGISTRY.get_or_init(|| {
        let registry = ProductionCommandRegistry::new();
        // Register built-in commands
        let _ = tokio::runtime::Handle::current().block_on(async {
            registry.register_command(Box::new(HelpCommand)).await
        });
        registry
    })
}

/// Built-in help command
#[derive(Debug, Clone)]
pub struct HelpCommand;

impl SimpleCommand for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }

    fn description(&self) -> &str {
        "Show help information for commands"
    }

    fn execute(&self, args: &[String]) -> Result<String, String> {
        if args.is_empty() {
            // Show general help
            Ok("Available commands:\n- help [command]: Show help for a specific command\n- Use 'help <command>' for detailed help".to_string())
        } else {
            // Show help for specific command
            let command_name = &args[0];
            match tokio::runtime::Handle::current().block_on(async {
                get_command_registry().get_command_help(command_name).await
            }) {
                Ok(help) => Ok(help),
                Err(e) => Err(format!("Failed to get help for '{}': {}", command_name, e)),
            }
        }
    }

    fn parser(&self) -> clap::Command {
        clap::Command::new("help")
            .about("Show help information for commands")
            .arg(
                clap::Arg::new("command")
                    .help("Command to show help for")
                    .value_name("COMMAND")
                    .required(false)
            )
    }

    fn clone_box(&self) -> Box<dyn SimpleCommand> {
        Box::new(self.clone())
    }
}

impl TaskServiceImpl {
    /// List available commands
    pub async fn list_available_commands(&self) -> Result<Vec<String>, String> {
        match get_command_registry().list_commands().await {
            Ok(commands) => {
                info!("Listed {} available commands", commands.len());
                Ok(commands)
            }
            Err(e) => {
                error!("Failed to list commands: {}", e);
                Err(e.to_string())
            }
        }
    }

    /// Execute a command
    pub async fn execute_command(
        &self,
        command_name: &str,
        args: HashMap<String, String>,
    ) -> Result<String, String> {
        // Convert HashMap to Vec<String> for execution
        let args_vec: Vec<String> = args.into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        
        match get_command_registry().execute_command(command_name, args_vec).await {
            Ok(output) => {
                info!("Command '{}' executed successfully", command_name);
                Ok(output)
            }
            Err(e) => {
                error!("Command '{}' execution failed: {}", command_name, e);
                Err(e.to_string())
            }
        }
    }

    /// Get help for a command
    pub async fn get_command_help(&self, command_name: &str) -> Result<String, String> {
        match get_command_registry().get_command_help(command_name).await {
            Ok(help) => {
                debug!("Retrieved help for command '{}'", command_name);
                Ok(help)
            }
            Err(e) => {
                error!("Failed to get help for command '{}': {}", command_name, e);
                Err(e.to_string())
            }
        }
    }

    // Helper method to validate task IDs
    pub fn validate_task_id(task_id: &str) -> Result<(), Status> {
        if task_id.is_empty() {
            return Err(Status::invalid_argument("Task ID cannot be empty"));
        }
        Ok(())
    }
    
    // Helper method to validate agent IDs
    pub fn validate_agent_id(agent_id: &str) -> Result<(), Status> {
        if agent_id.is_empty() {
            return Err(Status::invalid_argument("Agent ID cannot be empty"));
        }
        Ok(())
    }
}

/// Convert JSON parameters to string vector
pub fn json_params_to_string_vec(params_value: &serde_json::Value) -> Result<Vec<String>, String> {
    match params_value {
        serde_json::Value::Array(arr) => {
            let mut string_vec = Vec::new();
            for value in arr {
                match value {
                    serde_json::Value::String(s) => string_vec.push(s.clone()),
                    serde_json::Value::Number(n) => string_vec.push(n.to_string()),
                    serde_json::Value::Bool(b) => string_vec.push(b.to_string()),
                    _ => return Err(format!("Unsupported parameter type: {:?}", value)),
                }
            }
            Ok(string_vec)
        }
        _ => Err("Parameters must be an array".to_string()),
    }
}

/// Local command registry for simple commands
pub struct LocalCommandRegistry {
    commands: HashMap<String, Box<dyn SimpleCommand>>,
}

impl LocalCommandRegistry {
    /// Create a new local command registry
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// Get list of command names
    pub fn command_names(&self) -> Vec<String> {
        self.commands.keys().cloned().collect()
    }

    /// Get a command by name
    pub fn get(&mut self, name: &str) -> Option<Box<dyn SimpleCommand>> {
        self.commands.get(name).map(|cmd| cmd.clone_box())
    }
}

impl Default for LocalCommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Helper function to convert bytes to HashMap
fn bytes_to_hashmap(data: &[u8]) -> Result<HashMap<String, serde_json::Value>, String> {
    if data.is_empty() {
        return Ok(HashMap::new());
    }
    
    match serde_json::from_slice(data) {
        Ok(map) => Ok(map),
        Err(e) => Err(format!("Failed to parse JSON: {}", e)),
    }
} 