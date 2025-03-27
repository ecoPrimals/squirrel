//! Command executor for CLI commands
//!
//! This module provides functionality for executing commands in the CLI.

use clap::ArgMatches;
use std::sync::Arc;
use log::{debug, error, info, warn, trace};
use std::cell::RefCell;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use commands::{CommandError, CommandRegistry};
use crate::formatter::Factory as FormatterFactory;
use crate::commands::context::CommandContext;

thread_local! {
    static CURRENT_EXECUTION_CONTEXT: RefCell<Option<Arc<ExecutionContext>>> = const { RefCell::new(None) };
}

/// Context for command execution
#[derive(Debug)]
pub struct ExecutionContext {
    /// Registry of available commands
    registry: Arc<CommandRegistry>,
}

/// A timer for tracking lock acquisition time
pub struct LockTimer {
    operation: String,
    start_time: Instant,
    warn_threshold: Duration,
}

impl LockTimer {
    /// Creates a new LockTimer for the given operation
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            start_time: Instant::now(),
            warn_threshold: Duration::from_millis(50), // Warn if lock held for more than 50ms
        }
    }
    
    /// Creates a new LockTimer with a custom warning threshold
    pub fn with_threshold(operation: &str, warn_threshold: Duration) -> Self {
        Self {
            operation: operation.to_string(),
            start_time: Instant::now(),
            warn_threshold,
        }
    }
}

impl Drop for LockTimer {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        trace!("Lock for '{}' held for {:?}", self.operation, duration);
        
        if duration > self.warn_threshold {
            warn!(
                "Lock for '{}' held for {:?}, exceeding threshold of {:?}",
                self.operation, duration, self.warn_threshold
            );
        }
    }
}

/// Executes a command registry operation with minimal lock time
/// 
/// This function acquires the lock, performs the data access operation,
/// then releases the lock before executing any additional operations.
pub async fn execute_with_minimal_lock<T, F, Fut, R>(
    registry: &Arc<Mutex<T>>,
    operation_name: &str,
    access_fn: F,
) -> R
where
    F: FnOnce(&mut T) -> Fut,
    Fut: std::future::Future<Output = R>,
{
    let timer = LockTimer::new(operation_name);
    let mut registry_guard = registry.lock().await;
    let result = access_fn(&mut *registry_guard).await;
    drop(registry_guard); // Explicitly release the lock
    drop(timer); // Log the lock timing
    result
}

/// Helper function to perform a registry operation without holding the lock during computation
pub async fn with_registry<T, F, R>(
    registry: &Arc<Mutex<T>>,
    operation_name: &str,
    f: F,
) -> R
where
    F: FnOnce(&mut T) -> R,
{
    let timer = LockTimer::new(operation_name);
    let mut registry_guard = registry.lock().await;
    let result = f(&mut *registry_guard);
    drop(registry_guard); // Explicitly release the lock
    drop(timer); // Log the lock timing
    result
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(registry: Arc<CommandRegistry>) -> Self {
        Self {
            registry,
        }
    }
    
    /// Get the execution context from thread-local storage
    pub fn get_from_thread_local() -> Option<Arc<Self>> {
        let mut result = None;
        CURRENT_EXECUTION_CONTEXT.with(|ctx| {
            result = ctx.borrow().clone();
        });
        result
    }
    
    /// Set the execution context in thread-local storage
    pub fn set_in_thread_local(context: Arc<Self>) {
        CURRENT_EXECUTION_CONTEXT.with(|ctx| {
            *ctx.borrow_mut() = Some(context);
        });
    }
    
    /// Clear the execution context from thread-local storage
    pub fn clear_from_thread_local() {
        CURRENT_EXECUTION_CONTEXT.with(|ctx| {
            *ctx.borrow_mut() = None;
        });
    }

    /// Execute a command with the given arguments
    ///
    /// # Arguments
    ///
    /// * `command_name` - The name of the command to execute
    /// * `matches` - The parsed command-line arguments
    pub async fn execute_command(&self, command_name: &str, matches: ArgMatches) -> Result<(), CommandError> {
        debug!("Executing command: {}", command_name);
        
        // Create command context
        let context = CommandContext::new(matches);
        
        // Get the command from the registry
        let command = self.registry.get_command(command_name)?;
        
        // Extract args for the base Command trait
        let args: Vec<String> = context.matches().get_many("args")
            .map(|v| v.cloned().collect())
            .unwrap_or_default();
            
        // Execute the command
        match command.execute(&args) {
            Ok(output) => {
                // Determine output format from context flags
                let format = if context.matches().get_flag("json") {
                    "json"
                } else if context.matches().get_flag("yaml") {
                    "yaml"
                } else if context.matches().get_flag("table") {
                    "table"
                } else {
                    "text"
                };

                // Create formatter and format output
                let formatter = FormatterFactory::create_formatter(format)
                    .map_err(|e| CommandError::ExecutionError(e.to_string()))?;
                
                // Print the output
                println!("{}", formatter.format(&output).map_err(|e| CommandError::ExecutionError(e.to_string()))?);
                
                info!("Command '{}' executed successfully", command_name);
                Ok(())
            }
            Err(err) => {
                error!("Command '{}' execution failed: {}", command_name, err);
                Err(err)
            }
        }
    }
    
    /// Execute a command with the given string arguments
    ///
    /// # Arguments
    ///
    /// * `command_name` - The name of the command to execute
    /// * `args` - The command arguments as strings
    pub async fn execute_command_with_args(&self, command_name: &str, args: Vec<String>) -> Result<String, CommandError> {
        debug!("Executing command with string args: {} {:?}", command_name, args);
        
        // Get the command from the registry
        let command = self.registry.get_command(command_name)?;
        
        // Execute the command
        match command.execute(&args) {
            Ok(output) => {
                info!("Command '{}' executed successfully", command_name);
                Ok(output)
            }
            Err(err) => {
                error!("Command '{}' execution failed: {}", command_name, err);
                Err(err)
            }
        }
    }
    
    /// Get help for a command
    ///
    /// # Arguments
    ///
    /// * `command_name` - The name of the command to get help for
    pub async fn get_command_help(&self, command_name: &str) -> Result<String, CommandError> {
        debug!("Getting help for command: {}", command_name);
        
        // Get help through the adapter
        match self.registry.get_command(command_name) {
            Ok(command) => {
                debug!("Got help for command '{}'", command_name);
                Ok(command.help())
            }
            Err(err) => {
                error!("Failed to get help for command '{}': {}", command_name, err);
                Err(err)
            }
        }
    }
} 
