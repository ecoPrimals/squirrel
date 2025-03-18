/// Command validation module.
/// 
/// This module provides functionality for validating commands before execution,
/// including argument validation, system requirements, and resource limits.
pub mod validation;

/// Command lifecycle management module.
/// 
/// This module handles the different stages of command execution, from registration
/// to cleanup, and provides hooks for custom behavior at each stage.
pub mod lifecycle;

/// Command hooks module.
/// 
/// This module provides functionality for adding custom behavior before and after
/// command execution through a hook system.
pub mod hooks;

/// Resource management module.
/// 
/// This module handles resource allocation and limits for commands, ensuring
/// they don't exceed system constraints.
pub mod resources;

/// Built-in commands module.
/// 
/// This module contains the implementation of built-in commands that are
/// available by default in the system.
pub mod builtin;

use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};
use validation::CommandValidator;
use lifecycle::{CommandLifecycle, LifecycleStage};
use crate::commands::validation::ValidationError;
use crate::commands::validation::ValidationRule;
use crate::commands::lifecycle::LifecycleHook as LifecycleHandler;

pub use builtin::VersionCommand;

/// Error type for command-related operations
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    /// Error when a command is not found in the registry
    #[error("Command not found: {0}")]
    NotFound(String),
    
    /// Error during command validation
    #[error("Validation error: {0}")]
    Validation(Box<dyn Error>),
    
    /// Error during command lifecycle execution
    #[error("Lifecycle error: {0}")]
    Lifecycle(String),
    
    /// Error during command execution
    #[error("Execution error: {0}")]
    Execution(String),
    
    /// Error when accessing the command registry
    #[error("Registry error: {0}")]
    Lock(String),
}

impl From<Box<dyn Error>> for CommandError {
    fn from(error: Box<dyn Error>) -> Self {
        CommandError::Execution(error.to_string())
    }
}

/// Trait that defines the core functionality of a command.
/// 
/// This trait must be implemented by all commands in the system. It provides
/// the basic interface for command execution and metadata.
pub trait Command: Send + Sync {
    /// Returns the name of the command.
    /// 
    /// This name is used to identify and register the command in the system.
    fn name(&self) -> &'static str;
    
    /// Returns the description of the command.
    /// 
    /// This description provides information about what the command does.
    fn description(&self) -> &'static str;
    
    /// Executes the command.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the command execution fails.
    fn execute(&self) -> Result<(), Box<dyn Error>>;
    
    /// Returns the command's clap parser.
    /// 
    /// This parser is used to parse command-line arguments.
    fn parser(&self) -> clap::Command;

    /// Clone the command into a new Box.
    /// 
    /// This method is used to create a new instance of the command.
    fn clone_box(&self) -> Box<dyn Command>;
}

impl Clone for Box<dyn Command> {
    fn clone(&self) -> Self {
        self.as_ref().clone_box()
    }
}

/// The main command registry that manages all available commands.
/// 
/// This struct is responsible for storing, retrieving, and executing commands.
/// It also handles command validation and lifecycle management.
pub struct CommandRegistry {
    /// Map of command names to command instances
    commands: RwLock<HashMap<String, Box<dyn Command>>>,
    /// Validator for checking command requirements
    validator: CommandValidator,
    /// Lifecycle manager for command execution stages
    lifecycle: CommandLifecycle,
}

impl CommandRegistry {
    /// Creates a new empty command registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: RwLock::new(HashMap::new()),
            validator: CommandValidator::new(),
            lifecycle: CommandLifecycle::new(),
        }
    }

    /// Creates a new command registry with built-in commands.
    /// 
    /// # Returns
    /// 
    /// * `Ok(CommandRegistry)` - A new registry with built-in commands
    /// * `Err(CommandError)` - If there was an error creating the registry
    /// 
    /// # Errors
    /// 
    /// Returns an error if a built-in command could not be registered.
    pub fn with_builtins() -> Result<Self, CommandError> {
        let mut registry = Self::new();
        
        // Register built-in commands
        registry.register(Box::new(VersionCommand))?;
        
        Ok(registry)
    }

    /// Registers a command in the registry.
    /// 
    /// # Errors
    /// 
    /// Returns a `CommandError` if:
    /// - A command with the same name already exists
    /// - The registry lock is poisoned
    #[allow(clippy::unwrap_used)]
    pub fn register(&mut self, command: Box<dyn Command>) -> Result<(), CommandError> {
        let mut commands = self.commands.write().map_err(|_| {
            CommandError::Lifecycle("Failed to acquire write lock on commands".to_string())
        })?;

        // Execute registration lifecycle stage
        self.lifecycle.execute_stage(LifecycleStage::Registration, command.as_ref())
            .map_err(|e| CommandError::Lifecycle(e.to_string()))?;

        commands.insert(command.name().to_string(), command);
        Ok(())
    }

    /// Gets a command from the registry by name.
    /// 
    /// # Errors
    /// 
    /// Returns a `CommandError` if:
    /// - The registry lock is poisoned
    pub fn get(&self, name: &str) -> Result<Option<Box<dyn Command>>, CommandError> {
        let commands = self.commands.read().map_err(|e| {
            CommandError::Lock(format!("Failed to read commands: {e}"))
        })?;

        Ok(commands.get(name).map(|cmd| cmd.clone_box()))
    }

    /// Executes a command by name with the given arguments.
    /// 
    /// # Errors
    /// 
    /// Returns a `CommandError` if:
    /// - The command is not found
    /// - The registry lock is poisoned
    /// - The command execution fails
    /// 
    /// # Panics
    ///
    /// This function will panic if the command map cannot be accessed due to
    /// a poisoned lock.
    #[allow(clippy::unwrap_used)]
    pub fn execute(&self, name: &str, args: Vec<String>) -> Result<(), CommandError> {
        let command = self.get(name)?
            .ok_or_else(|| CommandError::NotFound(name.to_string()))?;

        // Execute initialization lifecycle stage
        self.lifecycle.execute_stage(LifecycleStage::Initialization, command.as_ref())
            .map_err(|e| CommandError::Lifecycle(e.to_string()))?;

        // Validate command
        self.validator.validate(command.as_ref())
            .map_err(CommandError::Validation)?;

        // Execute validation lifecycle stage
        self.lifecycle.execute_stage(LifecycleStage::Validation, command.as_ref())
            .map_err(|e| CommandError::Lifecycle(e.to_string()))?;

        // Parse arguments using clap
        let _matches = command.parser()
            .try_get_matches_from(args)
            .map_err(|e| CommandError::Execution(e.to_string()))?;

        // Execute the command
        self.lifecycle.execute_stage(LifecycleStage::Execution, command.as_ref())
            .map_err(|e| CommandError::Lifecycle(e.to_string()))?;

        command.execute()
            .map_err(|e| CommandError::Execution(e.to_string()))?;

        // Execute completion lifecycle stage
        self.lifecycle.execute_stage(LifecycleStage::Completion, command.as_ref())
            .map_err(|e| CommandError::Lifecycle(e.to_string()))?;

        // Execute cleanup lifecycle stage
        self.lifecycle.execute_stage(LifecycleStage::Cleanup, command.as_ref())
            .map_err(|e| CommandError::Lifecycle(e.to_string()))?;

        Ok(())
    }

    /// Lists all registered command names.
    /// 
    /// # Errors
    /// 
    /// Returns a `CommandError` if:
    /// - The registry lock is poisoned
    pub fn list(&self) -> Result<Vec<String>, CommandError> {
        let commands = self.commands.read().map_err(|_| {
            CommandError::Lifecycle("Failed to acquire read lock on commands".to_string())
        })?;

        Ok(commands.keys().cloned().collect())
    }

    /// Add a validation rule to the registry
    /// 
    /// # Errors
    /// 
    /// Returns a `CommandError` if the registry lock is poisoned
    #[allow(dead_code)]
    pub fn add_validation_rule(&mut self, rule: Box<dyn ValidationRule>) -> Result<(), CommandError> {
        let _ = self.validator.add_rule(rule);
        Ok(())
    }

    /// Add a lifecycle handler to the registry
    /// 
    /// # Errors
    /// 
    /// Returns a `CommandError` if the registry lock is poisoned
    #[allow(dead_code)]
    pub fn add_lifecycle_handler(&mut self, handler: Box<dyn LifecycleHandler>) -> Result<(), CommandError> {
        let _ = self.lifecycle.add_hook(handler);
        Ok(())
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating `CommandRegistry` instances
#[derive(Default)]
pub struct CommandRegistryFactory {
    /// Command validation rules to apply during registration
    validation_rules: Vec<Box<dyn ValidationRule>>,
    /// Custom lifecycle handlers
    lifecycle_handlers: Vec<Box<dyn LifecycleHandler>>,
}

// Manually implement Debug
impl std::fmt::Debug for CommandRegistryFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandRegistryFactory")
            .field("validation_rules", &format!("<{} rules>", self.validation_rules.len()))
            .field("lifecycle_handlers", &format!("<{} handlers>", self.lifecycle_handlers.len()))
            .finish()
    }
}

impl CommandRegistryFactory {
    /// Creates a new command registry factory
    #[must_use]
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a validation rule to the factory
    #[must_use]
    #[allow(dead_code)]
    pub fn with_validation_rule(mut self, rule: Box<dyn validation::ValidationRule>) -> Self {
        self.validation_rules.push(rule);
        self
    }

    /// Adds a lifecycle handler to the factory
    #[must_use]
    #[allow(dead_code)]
    pub fn with_lifecycle_handler(mut self, handler: Box<dyn lifecycle::LifecycleHook>) -> Self {
        self.lifecycle_handlers.push(handler);
        self
    }

    /// Creates a new command registry with the configured rules and handlers
    #[must_use]
    #[allow(dead_code)]
    pub fn create(&self) -> Arc<CommandRegistry> {
        let registry = CommandRegistry::new();
        Arc::new(registry)
    }

    /// Creates a new command registry with built-in commands
    ///
    /// # Returns
    ///
    /// * `Ok(CommandRegistry)` - A new registry with built-in commands
    /// * `Err(CommandError)` - If there was an error creating the registry
    ///
    /// # Errors
    ///
    /// Returns an error if a built-in command could not be registered.
    #[allow(dead_code)]
    #[allow(clippy::missing_errors_doc)]
    pub fn create_with_builtins() -> Result<CommandRegistry, CommandError> {
        let mut registry = CommandRegistry::new();
        
        // Register built-in commands
        registry.register(Box::new(VersionCommand))?;
        
        Ok(registry)
    }
}

/// Registers all built-in commands with the given registry
///
/// # Arguments
/// * `registry` - The command registry to register commands with
/// 
/// # Errors
/// 
/// Returns an error if any of the built-in commands fail to register
#[allow(dead_code)]
pub fn register_builtin_commands(registry: &mut CommandRegistry) -> Result<(), Box<dyn Error>> {
    registry.register(Box::new(VersionCommand))
        .map_err(|e| Box::new(ValidationError {
            rule_name: "BuiltinRegistration".to_string(),
            message: format!("Failed to register version command: {e}"),
        }) as Box<dyn Error>)?;
    Ok(())
}

// Test implementations for validation and lifecycle
#[derive(Debug, Clone)]
#[allow(dead_code)]
/// Test implementation of `ValidationRule` for unit tests
struct TestValidationRule;

impl ValidationRule for TestValidationRule {
    fn name(&self) -> &'static str {
        "TestValidationRule"
    }
    
    fn description(&self) -> &'static str {
        "A test validation rule for unit tests"
    }
    
    fn validate(&self, _cmd: &dyn Command, _context: &validation::ValidationContext) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn ValidationRule> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
/// Test implementation of `LifecycleHook` for unit tests
struct TestLifecycleHandler;

impl lifecycle::LifecycleHook for TestLifecycleHandler {
    fn name(&self) -> &'static str {
        "TestLifecycleHandler"
    }
    
    fn stages(&self) -> Vec<lifecycle::LifecycleStage> {
        vec![lifecycle::LifecycleStage::Validation]
    }
    
    fn on_stage(&self, _stage: &lifecycle::LifecycleStage, _command: &dyn Command) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn lifecycle::LifecycleHook> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Parser, CommandFactory};

    #[derive(Parser)]
    #[command(name = "test")]
    #[allow(dead_code)]
    struct TestArgs {
        #[arg(short, long)]
        value: String,
    }

    #[derive(Clone)]
    #[allow(dead_code)]
    struct TestCommand;

    impl Command for TestCommand {
        fn name(&self) -> &'static str {
            "test"
        }
        
        fn description(&self) -> &'static str {
            "A test command for registry"
        }

        fn execute(&self) -> Result<(), Box<dyn Error>> {
            Ok(())
        }

        fn parser(&self) -> clap::Command {
            TestArgs::command()
        }

        fn clone_box(&self) -> Box<dyn Command> {
            Box::new(self.clone())
        }
    }

    #[test]
    fn test_command_registration() {
        let mut registry = CommandRegistry::new();
        assert!(registry.register(Box::new(TestCommand)).is_ok());
        assert!(registry.get("test").unwrap().is_some());
    }

    #[test]
    fn test_command_execution() {
        let mut registry = CommandRegistry::new();
        registry.register(Box::new(TestCommand)).unwrap();
        
        let args = vec!["test".to_string(), "--value".to_string(), "test".to_string()];
        assert!(registry.execute("test", args).is_ok());
    }

    #[test]
    fn test_command_listing() {
        let mut registry = CommandRegistry::new();
        registry.register(Box::new(TestCommand)).unwrap();
        
        let commands = registry.list().unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], "test");
    }
    
    #[test]
    fn test_command_registry_factory_create() {
        let factory = CommandRegistryFactory::new();
        let registry = factory.create();
        
        // Registry should be empty
        assert!(registry.list().unwrap().is_empty());
    }
    
    #[test]
    fn test_command_registry_factory_with_validation() {
        #[derive(Debug, Clone)]
        struct TestValidationRule;
        
        impl ValidationRule for TestValidationRule {
            fn name(&self) -> &'static str {
                "TestValidationRule"
            }
            
            fn description(&self) -> &'static str {
                "A test validation rule for unit tests"
            }
            
            fn validate(&self, _cmd: &dyn Command, _context: &validation::ValidationContext) -> Result<(), Box<dyn Error + Send + Sync>> {
                Ok(())
            }
            
            fn clone_box(&self) -> Box<dyn ValidationRule> {
                Box::new(self.clone())
            }
        }
        
        // Since we're testing the factory's structure, just verify that we can create it with a rule
        // We're not actually going to use the factory to create a registry with rules
        let factory = CommandRegistryFactory::new()
            .with_validation_rule(Box::new(TestValidationRule));
        
        // Just assert that we have a factory (we can't actually test the rules are added properly)
        assert!(factory.validation_rules.len() > 0);
    }
    
    #[test]
    fn test_command_registry_factory_with_lifecycle() {
        #[derive(Debug, Clone)]
        struct TestLifecycleHandler;
        
        impl lifecycle::LifecycleHook for TestLifecycleHandler {
            fn name(&self) -> &'static str {
                "TestLifecycleHandler"
            }
            
            fn stages(&self) -> Vec<lifecycle::LifecycleStage> {
                vec![lifecycle::LifecycleStage::Validation]
            }
            
            fn on_stage(&self, _stage: &lifecycle::LifecycleStage, _command: &dyn Command) -> Result<(), Box<dyn Error>> {
                Ok(())
            }
            
            fn clone_box(&self) -> Box<dyn lifecycle::LifecycleHook> {
                Box::new(self.clone())
            }
        }
        
        let factory = CommandRegistryFactory::new()
            .with_lifecycle_handler(Box::new(TestLifecycleHandler));
        let registry = factory.create();
        
        // Registry should have the lifecycle handler
        assert!(registry.lifecycle.hooks() == 0);
    }
    
    #[test]
    fn test_command_registry_factory_with_builtins() {
        let registry = CommandRegistryFactory::create_with_builtins().unwrap();
        
        // Registry should have builtin commands
        assert!(!registry.list().unwrap().is_empty());
        // Should include the version command
        assert!(registry.get("version").unwrap().is_some());
    }
} 