//! CLI Commands module
//!
//! This module provides command handling for the CLI interface.

use crate::Result;
use tracing::{debug, info};

/// Register all CLI commands
pub fn register_commands() -> Result<()> {
    info!("Registering CLI built-in commands...");

    // Create a command registry from the services crate
    let services_registry = squirrel_commands::create_command_registry()
        .map_err(|e| format!("Failed to create services registry: {}", e))?;

    // Register commands successfully
    info!("Successfully registered built-in commands from services crate");

    // Log available commands safely
    match services_registry.lock() {
        Ok(registry) => match registry.list_commands() {
            Ok(commands) => {
                debug!("Available commands: {:?}", commands);
            }
            Err(e) => {
                debug!("Failed to list commands: {}", e);
            }
        },
        Err(e) => {
            debug!("Failed to access registry to list commands: {}", e);
        }
    }

    Ok(())
}

/// Command adapter module
pub mod adapter {
    use super::*;

    /// Command adapter trait
    pub trait CommandAdapterTrait {
        /// Execute a command
        fn execute(&self, command: &str, args: &[String]) -> impl std::future::Future<Output = Result<String>> + Send;
    }

    /// Command adapter implementation
    pub struct CommandAdapter {
        /// Internal state
        pub state: String,
    }

    impl Default for CommandAdapter {
        fn default() -> Self {
            Self::new()
        }
    }

    impl CommandAdapter {
        pub fn new() -> Self {
            Self {
                state: String::new(),
            }
        }
    }

    impl CommandAdapterTrait for CommandAdapter {
        fn execute(&self, _command: &str, _args: &[String]) -> impl std::future::Future<Output = Result<String>> + Send {
            async { Ok("Command executed".to_string()) }
        }
    }

    /// Registry adapter module
    pub mod registry {
        /// Command registry adapter
        pub struct CommandRegistryAdapter {
            /// Internal state
            pub state: String,
        }

        impl Default for CommandRegistryAdapter {
            fn default() -> Self {
                Self::new()
            }
        }

        impl CommandRegistryAdapter {
            pub fn new() -> Self {
                Self {
                    state: String::new(),
                }
            }
        }
    }

    /// Error handling module
    pub mod error {
        /// Adapter error type
        #[derive(Debug, thiserror::Error)]
        pub enum AdapterError {
            #[error("Command execution failed: {0}")]
            /// Command execution failed with an error message
            ExecutionFailed(String),
            #[error("Invalid command: {0}")]
            /// Invalid command was provided
            InvalidCommand(String),
        }

        /// Adapter result type
        pub type AdapterResult<T> = std::result::Result<T, AdapterError>;
    }
}

/// Command context module
pub mod context {
    /// Command execution context
    pub struct CommandContext {
        /// Context state
        pub state: String,
    }

    impl Default for CommandContext {
        fn default() -> Self {
            Self::new()
        }
    }

    impl CommandContext {
        pub fn new() -> Self {
            Self {
                state: String::new(),
            }
        }
    }
}

/// Command executor module
pub mod executor {
    /// Execution context
    pub struct ExecutionContext {
        /// Context state
        pub state: String,
    }

    impl Default for ExecutionContext {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ExecutionContext {
        pub fn new() -> Self {
            Self {
                state: String::new(),
            }
        }
    }
}

/// Command registry module
pub mod registry {
    use squirrel_commands::Command;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// Command registry
    pub struct CommandRegistry {
        /// Registered commands
        commands: Mutex<HashMap<String, Arc<dyn Command>>>,
    }

    impl Default for CommandRegistry {
        fn default() -> Self {
            Self::new()
        }
    }

    impl CommandRegistry {
        pub fn new() -> Self {
            Self {
                commands: Mutex::new(HashMap::new()),
            }
        }

        pub fn register_command(&self, name: String, command: Arc<dyn Command>) {
            if let Ok(mut commands) = self.commands.lock() {
                commands.insert(name, command);
            }
        }

        /// Register command (alias for register_command)
        pub fn register(&self, name: &str, command: Arc<dyn Command>) -> Result<(), String> {
            self.register_command(name.to_string(), command);
            Ok(())
        }

        /// Execute a command
        pub fn execute(&self, name: &str, args: &[String]) -> Result<String, String> {
            if let Ok(commands) = self.commands.lock() {
                if let Some(command) = commands.get(name) {
                    command
                        .execute(args)
                        .map_err(|e| format!("Command execution failed: {}", e))
                } else {
                    Err(format!("Command '{}' not found", name))
                }
            } else {
                Err("Failed to acquire command registry lock".to_string())
            }
        }

        pub fn get_command(&self, name: &str) -> Option<Arc<dyn Command>> {
            if let Ok(commands) = self.commands.lock() {
                commands.get(name).cloned()
            } else {
                None
            }
        }

        pub fn list_commands(&self) -> Vec<String> {
            if let Ok(commands) = self.commands.lock() {
                commands.keys().cloned().collect()
            } else {
                Vec::new()
            }
        }
    }
}

/// Command errors
pub mod error {
    /// Command error type
    #[derive(Debug, thiserror::Error)]
    pub enum CommandError {
        #[error("Command execution failed: {0}")]
        ExecutionFailed(String),
        #[error("Invalid command: {0}")]
        InvalidCommand(String),
    }
}
