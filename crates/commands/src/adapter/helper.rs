use std::sync::{Arc, Mutex};
use crate::{CommandRegistry, Command};
use crate::factory::{DefaultCommandRegistryFactory, CommandRegistryFactory};
use thiserror::Error;

/// Error types for adapter helpers
#[derive(Debug, Error)]
pub enum AdapterHelperError {
    /// Registry creation error
    #[error("Failed to create registry: {0}")]
    RegistryCreationError(String),
    
    /// Command registration error
    #[error("Failed to register command: {0}")]
    CommandRegistrationError(String),
}

/// Result type for adapter helpers
pub type AdapterHelperResult<T> = std::result::Result<T, AdapterHelperError>;

/// A registry adapter that provides a consistent interface for command operations
#[derive(Clone)]
pub struct CommandRegistryAdapter {
    /// The wrapped command registry
    registry: Arc<Mutex<CommandRegistry>>,
}

impl CommandRegistryAdapter {
    /// Creates a new command registry adapter
    pub fn new(registry: Arc<Mutex<CommandRegistry>>) -> Self {
        Self { registry }
    }
    
    /// Gets the underlying registry
    pub fn get_registry(&self) -> AdapterHelperResult<Arc<Mutex<CommandRegistry>>> {
        Ok(self.registry.clone())
    }
    
    /// Registers a command with the registry
    pub fn register_command(&self, command: Box<dyn Command>) -> AdapterHelperResult<()> {
        let registry = self.registry.lock().map_err(|e| {
            AdapterHelperError::CommandRegistrationError(format!("Failed to acquire lock: {}", e))
        })?;
        
        let command_name = command.name().to_string();
        let command_arc: Arc<dyn Command> = Arc::from(command);
        registry.register(&command_name, command_arc).map_err(|e| {
            AdapterHelperError::CommandRegistrationError(format!("Failed to register command: {}", e))
        })?;
        
        Ok(())
    }
    
    /// Executes a command with the given arguments
    pub fn execute_command(&self, command: &str, args: Vec<String>) -> AdapterHelperResult<String> {
        let registry = self.registry.lock().map_err(|e| {
            AdapterHelperError::CommandRegistrationError(format!("Failed to acquire lock: {}", e))
        })?;
        
        registry.execute(command, &args).map_err(|e| {
            AdapterHelperError::CommandRegistrationError(format!("Failed to execute command: {}", e))
        })
    }
    
    /// Lists all available commands
    pub fn list_commands(&self) -> AdapterHelperResult<Vec<String>> {
        let registry = self.registry.lock().map_err(|e| {
            AdapterHelperError::CommandRegistrationError(format!("Failed to acquire lock: {}", e))
        })?;
        
        registry.list_commands().map_err(|e| {
            AdapterHelperError::CommandRegistrationError(format!("Failed to list commands: {}", e))
        })
    }
}

/// Creates a registry adapter with an initialized registry containing built-in commands
pub fn create_initialized_registry_adapter() -> AdapterHelperResult<Arc<CommandRegistryAdapter>> {
    // Create factory
    let factory = DefaultCommandRegistryFactory::new();
    
    // Create registry with built-in commands
    let registry = factory.create_registry().map_err(|e| {
        AdapterHelperError::RegistryCreationError(format!("Failed to create registry: {}", e))
    })?;
    
    // Create adapter
    let adapter = CommandRegistryAdapter::new(registry);
    
    Ok(Arc::new(adapter))
}

/// Creates an empty registry adapter without any commands
pub fn create_empty_registry_adapter() -> AdapterHelperResult<Arc<CommandRegistryAdapter>> {
    // Create empty registry
    let registry = Arc::new(Mutex::new(CommandRegistry::new()));
    
    // Create adapter
    let adapter = CommandRegistryAdapter::new(registry);
    
    Ok(Arc::new(adapter))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Test command implementation
    #[derive(Debug, Clone)]
    struct TestCommand;
    
    impl Command for TestCommand {
        fn name(&self) -> &str {
            "test_adapter_command"
        }
        
        fn description(&self) -> &str {
            "Test command for adapter"
        }
        
        fn execute(&self, _args: &[String]) -> crate::CommandResult<String> {
            Ok("Test adapter command executed".to_string())
        }
        
        fn parser(&self) -> clap::Command {
            clap::Command::new("test_adapter_command")
                .about("Test command for adapter")
        }
        
        fn clone_box(&self) -> Box<dyn Command> {
            Box::new(self.clone())
        }
    }
    
    #[test]
    fn test_registry_adapter_creation() {
        let adapter = create_initialized_registry_adapter().unwrap();
        
        // List commands to verify built-ins are available
        let commands = adapter.list_commands().unwrap();
        
        // Check for common built-in commands
        assert!(commands.contains(&"help".to_string()));
        assert!(commands.contains(&"version".to_string()));
    }
    
    #[test]
    fn test_registry_adapter_command_registration() {
        let adapter = create_empty_registry_adapter().unwrap();
        
        // Register a test command
        adapter.register_command(Box::new(TestCommand)).unwrap();
        
        // Verify command was registered
        let commands = adapter.list_commands().unwrap();
        assert!(commands.contains(&"test_adapter_command".to_string()));
        
        // Execute the command
        let result = adapter.execute_command("test_adapter_command", vec![]).unwrap();
        assert_eq!(result, "Test adapter command executed");
    }
} 