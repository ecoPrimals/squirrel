// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Command execution handlers for the task service.
//!
//! This module provides command execution capabilities for the task service,
//! allowing tasks to execute commands and retrieve command information.

use crate::error::MCPError;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use super::service::TaskServiceImpl;
use super::types::SimpleCommand;
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
    /// Total number of command executions
    pub total_executions: u64,
    /// Number of successful executions
    pub successful_executions: u64,
    /// Number of failed executions
    pub failed_executions: u64,
    /// Average execution duration in milliseconds
    pub average_duration_ms: f64,
    /// Timestamp of last execution
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
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
            execution_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a command
    pub fn register_command(&self, command: Box<dyn SimpleCommand>) -> Result<(), ProductionError> {
        let command_name = command.name().to_string();

        let commands_result = SafeOperation::execute(|| {
            self.commands.try_write().map_err(|e| {
                ProductionError::concurrency(
                    format!("Failed to acquire command registry write lock: {e}"),
                    "command_registration",
                    true,
                )
            })
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
    pub fn list_commands(&self) -> Result<Vec<String>, ProductionError> {
        let commands_result = SafeOperation::execute(|| {
            self.commands.try_read().map_err(|e| {
                ProductionError::concurrency(
                    format!("Failed to acquire command registry read lock: {e}"),
                    "command_listing",
                    true,
                )
            })
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
    pub fn execute_command(
        &self,
        command_name: &str,
        args: &[String],
    ) -> Result<String, ProductionError> {
        let start_time = std::time::Instant::now();

        // Get the command
        let command = {
            let commands_result = SafeOperation::execute(|| {
                self.commands.try_read().map_err(|e| {
                    ProductionError::concurrency(
                        format!("Failed to acquire command registry read lock: {e}"),
                        "command_execution",
                        true,
                    )
                })
            });

            match commands_result.result() {
                Ok(commands) => commands.get(command_name).map(|cmd| cmd.clone_box()),
                Err(e) => {
                    error!(
                        "Failed to access command registry for '{}': {}",
                        command_name, e
                    );
                    return Err(e);
                }
            }
        };

        let available = self.list_commands().unwrap_or_default().join(", ");
        let command = command.ok_or_else(|| {
            ProductionError::not_found(
                format!("Command '{command_name}' not found"),
                "command_execution",
                Some(format!("Available commands: {available}")),
            )
        })?;

        // Execute the command
        let result = SafeOperation::execute(|| {
            command.execute(args).map_err(|e| {
                ProductionError::execution(
                    format!("Command '{command_name}' execution failed: {e}"),
                    "command_execution",
                    true,
                )
            })
        });

        let execution_time = start_time.elapsed();
        let cmd_result = result.result();

        // Update statistics
        self.update_command_stats(command_name, execution_time, cmd_result.is_ok());

        match cmd_result {
            Ok(output) => {
                info!(
                    "Command '{}' executed successfully in {:?}",
                    command_name, execution_time
                );
                Ok(output)
            }
            Err(e) => {
                error!("Command '{}' execution failed: {}", command_name, e);
                Err(e)
            }
        }
    }

    /// Get help for a command
    pub fn get_command_help(&self, command_name: &str) -> Result<String, ProductionError> {
        let commands_result = SafeOperation::execute(|| {
            self.commands.try_read().map_err(|e| {
                ProductionError::concurrency(
                    format!("Failed to acquire command registry read lock: {e}"),
                    "command_help",
                    true,
                )
            })
        });

        match commands_result.result() {
            Ok(commands) => {
                if let Some(command) = commands.get(command_name) {
                    Ok(command.help())
                } else {
                    Err(ProductionError::not_found(
                        format!("Command '{command_name}' not found"),
                        "command_help",
                        Some(format!(
                            "Available commands: {}",
                            commands.keys().cloned().collect::<Vec<_>>().join(", ")
                        )),
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
    fn update_command_stats(
        &self,
        command_name: &str,
        execution_time: std::time::Duration,
        success: bool,
    ) {
        let stats_result = SafeOperation::execute(|| {
            self.execution_stats.try_write().map_err(|e| {
                ProductionError::concurrency(
                    format!("Failed to acquire stats write lock: {e}"),
                    "stats_update",
                    false, // Not retryable - stats are not critical
                )
            })
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
            command_stats.average_duration_ms = command_stats
                .average_duration_ms
                .mul_add((command_stats.total_executions - 1) as f64, duration_ms)
                / command_stats.total_executions as f64;
            command_stats.last_execution = Some(chrono::Utc::now());
        }
    }

    /// Get command statistics
    #[must_use]
    pub fn get_command_stats(&self, command_name: &str) -> Option<CommandStats> {
        let stats_result = SafeOperation::execute(|| {
            self.execution_stats.try_read().map_err(|e| {
                ProductionError::concurrency(
                    format!("Failed to acquire stats read lock: {e}"),
                    "stats_read",
                    false,
                )
            })
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
static COMMAND_REGISTRY: std::sync::OnceLock<ProductionCommandRegistry> =
    std::sync::OnceLock::new();

/// Get the global command registry instance
pub fn get_command_registry() -> &'static ProductionCommandRegistry {
    COMMAND_REGISTRY.get_or_init(|| {
        let registry = ProductionCommandRegistry::new();
        // Register built-in commands
        let _ = registry.register_command(Box::new(HelpCommand));
        registry
    })
}

/// Built-in help command
#[derive(Debug, Clone)]
pub struct HelpCommand;

impl SimpleCommand for HelpCommand {
    fn name(&self) -> &'static str {
        "help"
    }

    fn description(&self) -> &'static str {
        "Show help information for commands"
    }

    fn execute(&self, args: &[String]) -> Result<String, String> {
        if args.is_empty() {
            // Show general help
            Ok("Available commands:\n- help [command]: Show help for a specific command\n- Use 'help <command>' for detailed help".to_string())
        } else {
            // Show help for specific command
            let command_name = &args[0];
            match get_command_registry().get_command_help(command_name) {
                Ok(help) => Ok(help),
                Err(e) => Err(format!("Failed to get help for '{command_name}': {e}")),
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
                    .required(false),
            )
    }

    fn clone_box(&self) -> Box<dyn SimpleCommand> {
        Box::new(self.clone())
    }
}

impl TaskServiceImpl {
    /// List available commands
    pub fn list_available_commands(&self) -> Result<Vec<String>, String> {
        match get_command_registry().list_commands() {
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
    pub fn execute_command(
        &self,
        command_name: &str,
        args: HashMap<String, String>,
    ) -> Result<String, String> {
        // Convert HashMap to Vec<String> for execution
        let args_vec: Vec<String> = args.into_iter().map(|(k, v)| format!("{k}={v}")).collect();

        match get_command_registry().execute_command(command_name, &args_vec) {
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
    pub fn get_command_help(&self, command_name: &str) -> Result<String, String> {
        match get_command_registry().get_command_help(command_name) {
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

    /// Validates that a task ID is non-empty. Returns error if invalid.
    pub fn validate_task_id(task_id: &str) -> Result<(), MCPError> {
        if task_id.is_empty() {
            return Err(MCPError::InvalidArgument(
                "Task ID cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    /// Validates that an agent ID is non-empty. Returns error if invalid.
    pub fn validate_agent_id(agent_id: &str) -> Result<(), MCPError> {
        if agent_id.is_empty() {
            return Err(MCPError::InvalidArgument(
                "Agent ID cannot be empty".to_string(),
            ));
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
                    _ => return Err(format!("Unsupported parameter type: {value:?}")),
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
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// Get list of command names
    #[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::manager::TaskManager;
    use crate::task::server::mock::MockCommand;
    use crate::task::server::service::{TaskServerConfig, TaskServiceImpl};
    use serde_json::json;

    #[test]
    fn command_stats_default_clone_debug() {
        let a = CommandStats::default();
        let b = a.clone();
        assert_eq!(a.total_executions, b.total_executions);
        let s = format!("{a:?}");
        assert!(s.contains("CommandStats"));
    }

    #[test]
    fn production_command_registry_register_list_execute_help_and_stats() {
        let reg = ProductionCommandRegistry::new();
        reg.register_command(Box::new(MockCommand::new("echo", "echo args")))
            .unwrap();
        let names = reg.list_commands().unwrap();
        assert!(names.contains(&"echo".to_string()));
        let out = reg.execute_command("echo", &[String::from("x")]).unwrap();
        assert!(out.contains("echo"));
        let help = reg.get_command_help("echo").unwrap();
        assert!(help.contains("echo"));
        let stats = reg.get_command_stats("echo").expect("stats");
        assert!(stats.total_executions >= 1);
        assert!(stats.successful_executions >= 1);
    }

    #[test]
    fn production_command_registry_execute_unknown_command_returns_not_found() {
        let reg = ProductionCommandRegistry::new();
        let err = reg.execute_command("no_such_cmd", &[]).unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn production_command_registry_get_help_unknown_returns_not_found() {
        let reg = ProductionCommandRegistry::new();
        let err = reg.get_command_help("missing").unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn json_params_to_string_vec_accepts_array_of_primitives() {
        let v = json!(["a", 1, true]);
        let got = json_params_to_string_vec(&v).unwrap();
        assert_eq!(got, vec!["a", "1", "true"]);
    }

    #[test]
    fn json_params_to_string_vec_rejects_non_array() {
        let err = json_params_to_string_vec(&json!({"k": "v"})).unwrap_err();
        assert!(err.contains("array"));
    }

    #[test]
    fn json_params_to_string_vec_rejects_unsupported_element() {
        let err = json_params_to_string_vec(&json!([[]])).unwrap_err();
        assert!(err.contains("Unsupported"));
    }

    #[test]
    fn local_command_registry_new_default_and_empty() {
        let mut reg = LocalCommandRegistry::new();
        assert!(reg.command_names().is_empty());
        assert!(reg.get("nope").is_none());
        let reg2 = LocalCommandRegistry::default();
        assert!(reg2.command_names().is_empty());
    }

    #[test]
    fn help_command_parser_name_and_execute_general_help() {
        let h = HelpCommand;
        assert_eq!(h.parser().get_name(), "help");
        let out = h.execute(&[]).unwrap();
        assert!(out.contains("Available commands"));
    }

    #[tokio::test]
    async fn task_service_impl_validators_and_command_wrappers() {
        let tm = Arc::new(tokio::sync::Mutex::new(TaskManager::new()));
        let svc = TaskServiceImpl::new(tm, TaskServerConfig::default());
        assert!(TaskServiceImpl::validate_task_id("").is_err());
        assert!(TaskServiceImpl::validate_task_id("tid").is_ok());
        assert!(TaskServiceImpl::validate_agent_id("").is_err());
        assert!(TaskServiceImpl::validate_agent_id("aid").is_ok());
        let cmds = svc.list_available_commands().unwrap();
        assert!(cmds.iter().any(|c| c == "help"));
        let help = svc.get_command_help("help").unwrap();
        assert!(!help.is_empty());
        let exec = svc
            .execute_command("help", HashMap::new())
            .expect("execute");
        assert!(exec.contains("Available commands"));
    }
}
