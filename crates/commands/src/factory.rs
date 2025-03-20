//! Factory for creating command registries.
//!
//! This module provides functionality for creating and configuring command registries.

use std::sync::Arc;
use crate::registry::{CommandRegistry, CommandResult};
use crate::builtin::{HelpCommand, VersionCommand};

/// Factory for creating command registries.
pub trait CommandRegistryFactory {
    /// Create a new command registry.
    fn create_registry(&self) -> CommandResult<Arc<CommandRegistry>>;
    
    /// Add built-in commands to the registry.
    fn add_builtin_commands(&self, registry: &Arc<CommandRegistry>) -> CommandResult<()>;
}

/// Create a command registry with the default built-in commands.
///
/// # Returns
///
/// A new command registry with the built-in commands registered.
pub fn create_command_registry() -> CommandResult<Arc<CommandRegistry>> {
    let registry = Arc::new(CommandRegistry::new());
    
    // Register built-in commands
    registry.register(Box::new(VersionCommand))?;
    
    // Create and register the help command (needs a reference to the registry itself)
    let help_command = HelpCommand::new(Arc::clone(&registry));
    registry.register(Box::new(help_command))?;
    
    Ok(registry)
}

/// Default implementation of CommandRegistryFactory.
#[derive(Debug, Default)]
pub struct DefaultCommandRegistryFactory;

impl CommandRegistryFactory for DefaultCommandRegistryFactory {
    fn create_registry(&self) -> CommandResult<Arc<CommandRegistry>> {
        create_command_registry()
    }
    
    fn add_builtin_commands(&self, registry: &Arc<CommandRegistry>) -> CommandResult<()> {
        registry.register(Box::new(VersionCommand))?;
        
        // Create and register the help command
        let help_command = HelpCommand::new(Arc::clone(registry));
        registry.register(Box::new(help_command))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_command_registry() {
        let registry = create_command_registry().unwrap();
        assert!(registry.get_command("version").is_ok());
        assert!(registry.get_command("help").is_ok());
    }
    
    #[test]
    fn test_default_factory() {
        let factory = DefaultCommandRegistryFactory::default();
        let registry = factory.create_registry().unwrap();
        assert!(registry.get_command("version").is_ok());
        assert!(registry.get_command("help").is_ok());
    }
} 