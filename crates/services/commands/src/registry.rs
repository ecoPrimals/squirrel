// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Command registry for managing and executing commands
#![allow(
    clippy::type_complexity,
    reason = "Complex handler types are inherent to the command pattern"
)]
//!
//! This module provides the core types and interfaces for the command system.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tracing::{debug, error, info, warn};

use crate::error::CommandError;

/// Type alias for command operation results
pub type CommandResult<T> = Result<T, CommandError>;

/// Trait for commands to implement
pub trait Command: Send + Sync {
    /// Returns the name of the command
    fn name(&self) -> &str;

    /// Returns a description of what the command does
    fn description(&self) -> &str;

    /// Executes the command with the given arguments
    fn execute(&self, args: &[String]) -> CommandResult<String>;

    /// Returns help text for the command
    fn help(&self) -> String {
        format!("{}: {}", self.name(), self.description())
    }

    /// Returns a parser for the command's arguments
    fn parser(&self) -> clap::Command;

    /// Clones the command into a new box
    fn clone_box(&self) -> Box<dyn Command>;
}

/// Struct to track lock timing for registry operations
#[derive(Debug)]
struct LockTimer {
    operation: String,
    start_time: Instant,
    warn_threshold: Duration,
}

impl LockTimer {
    fn new(operation: &str) -> Self {
        debug!("Registry: Acquiring lock for operation '{}'", operation);
        Self {
            operation: operation.to_string(),
            start_time: Instant::now(),
            warn_threshold: Duration::from_millis(50), // Warn if lock held for more than 50ms
        }
    }

    fn complete(self) {
        let _duration = self.end();
    }

    fn end(self) -> Duration {
        let duration = self.start_time.elapsed();
        debug!(
            "Registry: Lock operation '{}' completed in {:?}",
            self.operation, duration
        );

        if duration > self.warn_threshold {
            warn!(
                "Registry: Lock operation '{}' took {:?} - potential contention",
                self.operation, duration
            );
        }

        duration
    }
}

/// Registry for storing and executing commands
#[derive(Clone)]
pub struct CommandRegistry {
    /// Map of command names to command implementations
    commands: Arc<Mutex<HashMap<String, Arc<dyn Command>>>>,
    /// Post-execution hooks
    post_hooks: Arc<Mutex<Vec<Arc<dyn Fn(&str, &CommandResult<String>) + Send + Sync>>>>,
    /// Resource storage for sharing data between commands
    resources: Arc<Mutex<HashMap<String, Arc<dyn std::any::Any + Send + Sync>>>>,
}

// Manual implementation of Debug for CommandRegistry
impl fmt::Debug for CommandRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Try to acquire the lock - if we can't, just show a placeholder
        match self.commands.lock() {
            Ok(commands) => {
                let command_names: Vec<_> = commands.keys().collect();
                f.debug_struct("CommandRegistry")
                    .field("commands", &command_names)
                    .finish()
            }
            Err(_) => f
                .debug_struct("CommandRegistry")
                .field("commands", &"<locked>")
                .finish(),
        }
    }
}

impl CommandRegistry {
    /// Creates a new command registry
    #[must_use]
    pub fn new() -> Self {
        debug!("Creating new CommandRegistry instance");
        Self {
            commands: Arc::new(Mutex::new(HashMap::new())),
            post_hooks: Arc::new(Mutex::new(Vec::new())),
            resources: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Registers a command with the registry
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the command to register
    /// * `command` - The command to register
    ///
    /// # Errors
    ///
    /// Returns an error if a command with the same name already exists
    pub fn register(&self, name: &str, command: Arc<dyn Command>) -> CommandResult<()> {
        let timer = LockTimer::new(&format!("register_{name}"));

        // Get a lock on the commands map
        let mut commands = self.commands.lock().map_err(|e| {
            error!("Registry: Failed to acquire lock for register: {}", e);
            CommandError::RegistryError(format!("Failed to acquire lock: {e}"))
        })?;

        // Check if the command already exists
        if commands.contains_key(name) {
            return Err(CommandError::CommandAlreadyExists(name.to_string()));
        }

        // Insert the command
        commands.insert(name.to_string(), command);
        info!("Registry: Command '{}' registered successfully", name);

        timer.end();
        Ok(())
    }

    /// Executes a command by name with the given arguments
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the command to execute
    /// * `args` - The arguments to pass to the command
    ///
    /// # Errors
    ///
    /// Returns an error if the command does not exist or if execution fails
    pub fn execute(&self, name: &str, args: &Vec<String>) -> CommandResult<String> {
        let timer = LockTimer::new(&format!("execute_{name}"));
        debug!(
            "Registry: Executing command '{}' with args {:?}",
            name, args
        );

        // Get the command instance
        let command = {
            // Get a lock on the commands map
            let commands = self.commands.lock().map_err(|e| {
                error!("Registry: Failed to acquire lock for execute: {}", e);
                CommandError::RegistryError(format!("Failed to acquire lock: {e}"))
            })?;

            // Check if the command exists
            if !commands.contains_key(name) {
                return Err(CommandError::CommandNotFound(name.to_string()));
            }

            // Clone the command to avoid holding the lock during execution
            commands
                .get(name)
                .ok_or_else(|| CommandError::CommandNotFound(name.to_string()))?
                .clone()
        }; // Lock is released here

        timer.end();
        debug!("Registry: Lock released before command execution");

        // Execute the command without holding the lock
        let start = Instant::now();
        let result = command.execute(args);
        let duration = start.elapsed();

        // Log the execution time
        match &result {
            Ok(_) => info!(
                "Registry: Command '{}' execution completed in {:?}",
                name, duration
            ),
            Err(e) => warn!(
                "Registry: Command '{}' execution failed in {:?}: {}",
                name, duration, e
            ),
        }

        // Execute post-hooks
        self.execute_post_hooks(name, &result);

        result
    }

    /// Returns a list of all registered command names
    ///
    /// # Errors
    ///
    /// Returns an error if the command registry cannot be locked
    pub fn list_commands(&self) -> CommandResult<Vec<String>> {
        let timer = LockTimer::new("list_commands");

        // Get a lock on the commands map
        let commands = self.commands.lock().map_err(|e| {
            error!("Registry: Failed to acquire lock for list_commands: {}", e);
            CommandError::RegistryError(format!("Failed to acquire lock: {e}"))
        })?;

        // Get the list of command names
        let result = commands.keys().cloned().collect();
        let count = commands.len();

        timer.end();
        debug!("Registry: Listed {} commands", count);

        Ok(result)
    }

    /// Returns help text for a command
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the command to get help for
    ///
    /// # Errors
    ///
    /// Returns an error if the command does not exist
    pub fn get_help(&self, name: &str) -> CommandResult<String> {
        let timer = LockTimer::new(&format!("get_help_{name}"));

        // Get the command instance
        let command = {
            // Get a lock on the commands map
            let commands = self.commands.lock().map_err(|e| {
                error!("Registry: Failed to acquire lock for get_help: {}", e);
                CommandError::RegistryError(format!("Failed to acquire lock: {e}"))
            })?;

            // Check if the command exists
            if !commands.contains_key(name) {
                return Err(CommandError::CommandNotFound(name.to_string()));
            }

            // Clone the command to avoid holding the lock during help generation
            commands
                .get(name)
                .ok_or_else(|| CommandError::CommandNotFound(name.to_string()))?
                .clone()
        }; // Lock is released here

        timer.end();
        debug!("Registry: Lock released before generating help");

        // Get the help text without holding the lock
        Ok(command.help())
    }

    /// Retrieves a command by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the command to retrieve
    ///
    /// # Errors
    ///
    /// Returns an error if the command does not exist
    pub fn get_command(&self, name: &str) -> CommandResult<Box<dyn Command>> {
        let commands = self.commands.lock().map_err(|_| {
            CommandError::ExecutionError("Failed to acquire lock on command registry".to_string())
        })?;

        // Find the command while holding the lock
        match commands.get(name) {
            Some(command) => {
                // Clone the command into a new Box
                let command_clone = command.clone_box();

                // Return the cloned command after releasing the lock
                Ok(command_clone)
            }
            None => Err(CommandError::ExecutionError(format!(
                "Command '{name}' not found"
            ))),
        }
    }

    /// Checks if a command with the given name exists in the registry
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the command to check for
    ///
    /// # Errors
    ///
    /// Returns an error if the command registry cannot be locked
    pub fn command_exists(&self, name: &str) -> CommandResult<bool> {
        let commands = self.commands.lock().map_err(|e| {
            CommandError::ExecutionError(format!("Failed to acquire lock on command registry: {e}"))
        })?;

        Ok(commands.contains_key(name))
    }

    /// Add a post-execution hook that runs after command execution
    pub fn add_post_hook(
        &self,
        hook: Arc<dyn Fn(&str, &CommandResult<String>) + Send + Sync>,
    ) -> CommandResult<()> {
        let timer = LockTimer::new("add_post_hook");

        match self.post_hooks.lock() {
            Ok(mut hooks) => {
                hooks.push(hook);
                timer.complete();
                debug!(
                    "Registry: Added post-execution hook (total: {})",
                    hooks.len()
                );
                Ok(())
            }
            Err(e) => {
                error!("Registry: Failed to acquire post_hooks lock: {}", e);
                Err(CommandError::Internal(
                    "Failed to add post-execution hook".to_string(),
                ))
            }
        }
    }

    /// Execute all registered post-hooks for a command
    fn execute_post_hooks(&self, command_name: &str, result: &CommandResult<String>) {
        if let Ok(hooks) = self.post_hooks.lock() {
            for (idx, hook) in hooks.iter().enumerate() {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    hook(command_name, result);
                })) {
                    Ok(_) => debug!(
                        "Registry: Executed post-hook {} for command '{}'",
                        idx, command_name
                    ),
                    Err(_) => warn!(
                        "Registry: Post-hook {} panicked for command '{}'",
                        idx, command_name
                    ),
                }
            }
        }
    }

    /// Set a resource in the registry for sharing between commands
    pub fn set_resource<T: std::any::Any + Send + Sync>(
        &self,
        name: &str,
        resource: T,
    ) -> CommandResult<()> {
        let timer = LockTimer::new("set_resource");

        match self.resources.lock() {
            Ok(mut resources) => {
                resources.insert(name.to_string(), Arc::new(resource));
                timer.complete();
                debug!(
                    "Registry: Set resource '{}' (total resources: {})",
                    name,
                    resources.len()
                );
                Ok(())
            }
            Err(e) => {
                error!("Registry: Failed to acquire resources lock: {}", e);
                Err(CommandError::Internal(format!(
                    "Failed to set resource '{}'",
                    name
                )))
            }
        }
    }

    /// Get a resource from the registry
    pub fn get_resource<T: std::any::Any + Send + Sync>(&self, name: &str) -> Option<Arc<T>> {
        if let Ok(resources) = self.resources.lock() {
            if let Some(resource) = resources.get(name) {
                if let Ok(typed_resource) = resource.clone().downcast::<T>() {
                    debug!("Registry: Retrieved resource '{}'", name);
                    return Some(typed_resource);
                } else {
                    warn!("Registry: Resource '{}' found but type mismatch", name);
                }
            } else {
                debug!("Registry: Resource '{}' not found", name);
            }
        } else {
            error!("Registry: Failed to acquire resources lock for get");
        }
        None
    }

    /// Remove a resource from the registry
    pub fn remove_resource(&self, name: &str) -> CommandResult<bool> {
        let timer = LockTimer::new("remove_resource");

        match self.resources.lock() {
            Ok(mut resources) => {
                let removed = resources.remove(name).is_some();
                timer.complete();
                if removed {
                    debug!(
                        "Registry: Removed resource '{}' (remaining: {})",
                        name,
                        resources.len()
                    );
                } else {
                    debug!("Registry: Resource '{}' not found for removal", name);
                }
                Ok(removed)
            }
            Err(e) => {
                error!("Registry: Failed to acquire resources lock: {}", e);
                Err(CommandError::Internal(format!(
                    "Failed to remove resource '{}'",
                    name
                )))
            }
        }
    }

    /// List all available resource names
    pub fn list_resources(&self) -> Vec<String> {
        if let Ok(resources) = self.resources.lock() {
            resources.keys().cloned().collect()
        } else {
            warn!("Registry: Failed to acquire resources lock for list");
            Vec::new()
        }
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestCommand;

    impl Command for TestCommand {
        fn name(&self) -> &str {
            "test"
        }

        fn description(&self) -> &str {
            "A test command"
        }

        fn execute(&self, _args: &[String]) -> CommandResult<String> {
            Ok("Test command executed".to_string())
        }

        fn parser(&self) -> clap::Command {
            clap::Command::new("test").about("A test command")
        }

        fn clone_box(&self) -> Box<dyn Command> {
            Box::new(self.clone())
        }
    }

    #[test]
    fn test_register_command() {
        let registry = CommandRegistry::new();
        let result = registry.register("test", Arc::new(TestCommand));
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_duplicate_command() {
        let registry = CommandRegistry::new();
        registry
            .register("test", Arc::new(TestCommand))
            .expect("should succeed");
        let result = registry.register("test", Arc::new(TestCommand));
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_command() {
        let registry = CommandRegistry::new();
        registry
            .register("test", Arc::new(TestCommand))
            .expect("should succeed");
        let result = registry.execute("test", &Vec::new());
        assert!(result.is_ok());
        assert_eq!(result.expect("should succeed"), "Test command executed");
    }

    #[test]
    fn test_execute_nonexistent_command() {
        let registry = CommandRegistry::new();
        let result = registry.execute("nonexistent", &Vec::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_list_commands() {
        let registry = CommandRegistry::new();
        registry
            .register("test", Arc::new(TestCommand))
            .expect("should succeed");
        let result = registry.list_commands();
        assert!(result.is_ok());
        let commands = result.expect("should succeed");
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], "test");
    }

    #[test]
    fn test_get_help() {
        let registry = CommandRegistry::new();
        registry
            .register("test", Arc::new(TestCommand))
            .expect("should succeed");
        let result = registry.get_help("test");
        assert!(result.is_ok());
        assert_eq!(result.expect("should succeed"), "test: A test command");
    }
}
