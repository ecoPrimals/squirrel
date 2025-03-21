//! Factory for creating command registries.
//!
//! This module provides functionality for creating and configuring command registries.

use crate::{
    registry::CommandRegistry,
    builtin::{VersionCommand, HelpCommand, EchoCommand, ExitCommand, KillCommand, HistoryCommand},
    history::CommandHistory,
};
use std::{
    error::Error,
    fmt::Debug,
    sync::{Arc, Mutex},
    time::Instant,
};

use log::{debug, info};

/// The command registry factory trait
/// 
/// Implementations of this trait are responsible for creating and configuring
/// command registries.
pub trait CommandRegistryFactory: Debug {
    /// Create a new command registry
    /// 
    /// # Returns
    /// 
    /// A Result containing an `Arc<Mutex<CommandRegistry>>` or an error
    fn create_registry(&self) -> Result<Arc<Mutex<CommandRegistry>>, Box<dyn Error>>;

    /// Register built-in commands in the provided registry
    /// 
    /// # Arguments
    /// 
    /// * `registry` - The registry to register commands in
    /// 
    /// # Returns
    /// 
    /// A Result that is Ok if all commands were registered successfully, or an error
    fn register_builtin_commands(
        &self,
        registry: &Arc<Mutex<CommandRegistry>>,
    ) -> Result<(), Box<dyn Error>>;
}

/// Create a command registry with built-in commands
/// 
/// # Returns
/// 
/// A Result containing an `Arc<Mutex<CommandRegistry>>` or an error
pub fn create_command_registry() -> Result<Arc<Mutex<CommandRegistry>>, Box<dyn Error>> {
    debug!("Factory: Creating command registry using DefaultCommandRegistryFactory");
    let factory = DefaultCommandRegistryFactory::new();
    factory.create_registry()
}

/// The default command registry factory
/// 
/// Creates a command registry with basic built-in commands like help and version.
/// This implementation uses a deadlock-safe approach for command registration.
#[derive(Debug)]
pub struct DefaultCommandRegistryFactory;

impl DefaultCommandRegistryFactory {
    /// Creates a new instance of the default factory
    #[must_use] pub fn new() -> Self {
        debug!("Factory: Creating new DefaultCommandRegistryFactory instance");
        Self
    }
}

impl Default for DefaultCommandRegistryFactory {
    fn default() -> Self {
        debug!("Factory: Creating DefaultCommandRegistryFactory using default implementation");
        Self
    }
}

impl CommandRegistryFactory for DefaultCommandRegistryFactory {
    fn create_registry(&self) -> Result<Arc<Mutex<CommandRegistry>>, Box<dyn Error>> {
        debug!("Factory: Creating command registry");
        let start = Instant::now();
        
        let registry = Arc::new(Mutex::new(CommandRegistry::new()));
        self.register_builtin_commands(&registry)?;
        
        let duration = start.elapsed();
        info!("Factory: Command registry created in {:?}", duration);
        Ok(registry)
    }

    /// Register built-in commands in the provided registry
    /// 
    /// This implementation uses a deadlock-safe approach by:
    /// 1. Registering non-help commands first
    /// 2. Creating the HelpCommand with pre-loaded command information
    /// 3. Registering the HelpCommand last
    fn register_builtin_commands(
        &self,
        registry: &Arc<Mutex<CommandRegistry>>,
    ) -> Result<(), Box<dyn Error>> {
        debug!("Factory: Registering built-in commands");
        let start = Instant::now();
        
        // Create a history manager
        let history = Arc::new(CommandHistory::new()?);
        
        // Add the history hook to Record command executions
        {
            let _registry_guard = registry.lock().map_err(|e| {
                Box::<dyn Error>::from(format!(
                    "Failed to acquire lock on registry to register history hook: {}", e
                ))
            })?;
            
            // TODO: Implement add_post_hook in CommandRegistry
            // registry_guard.add_post_hook(create_history_hook(Arc::clone(&history)))?;
            debug!("Factory: History hook not added - function not implemented in CommandRegistry");
        }
        
        // Step 1: Register non-help commands
        {
            debug!("Factory: Registering non-help commands (acquiring lock)");
            let registry_guard = registry.lock().map_err(|e| {
                Box::<dyn Error>::from(format!(
                    "Failed to acquire lock on registry to register non-help commands: {}", e
                ))
            })?;
            
            // Register basic commands
            registry_guard.register("version", Arc::new(VersionCommand::new()))?;
            registry_guard.register("echo", Arc::new(EchoCommand::new()))?;
            registry_guard.register("exit", Arc::new(ExitCommand::new()))?;
            registry_guard.register("kill", Arc::new(KillCommand::new()))?;
            
            // Register the history command
            registry_guard.register("history", Arc::new(HistoryCommand::new(Arc::clone(&history))))?;
            
            // TODO: Implement set_resource in CommandRegistry
            // registry_guard.set_resource("command_history", Box::new(Arc::clone(&history)))?;
            debug!("Factory: Resource not set - function not implemented in CommandRegistry");
            
            debug!("Factory: Non-help commands registered (releasing lock)");
        }
        
        // Step 2: Create HelpCommand with pre-loaded command information
        let help_command = HelpCommand::new(Arc::clone(registry));
        
        // Step 3: Register the HelpCommand
        {
            debug!("Factory: Registering help command (acquiring lock)");
            let registry_guard = registry.lock().map_err(|e| {
                Box::<dyn Error>::from(format!(
                    "Failed to acquire lock on registry to register help command: {}", e
                ))
            })?;
            
            registry_guard.register("help", Arc::new(help_command))?;
            
            debug!("Factory: Help command registered (releasing lock)");
        }
        
        let duration = start.elapsed();
        info!("Factory: Built-in commands registered in {:?}", duration);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Instant, Duration};
    use log::{debug, info};
    use crate::{Command, CommandResult};
    
    /// Helper structure to track lock timing in tests
    struct TestLockTimer {
        operation: String,
        start_time: Instant,
    }
    
    impl TestLockTimer {
        fn new(operation: &str) -> Self {
            debug!("Test: Acquiring lock for operation '{}'", operation);
            Self {
                operation: operation.to_string(),
                start_time: Instant::now(),
            }
        }
        
        fn end(self) -> Duration {
            let duration = self.start_time.elapsed();
            debug!("Test: Lock operation '{}' completed in {:?}", self.operation, duration);
            duration
        }
    }
    
    #[test]
    fn test_create_command_registry() -> Result<(), Box<dyn Error>> {
        // Initialize logging for tests
        let _ = simple_logger::SimpleLogger::new()
            .with_level(log::LevelFilter::Debug)
            .init();
            
        info!("Test: Starting test_create_command_registry");
        let registry = create_command_registry()?;
        
        // First, verify commands exist in the registry
        let commands = {
            let timer = TestLockTimer::new("list_commands");
            let registry_lock = registry.lock().map_err(|e| {
                let err_msg = format!("Failed to acquire lock on command registry: {}", e);
                Box::<dyn Error>::from(err_msg)
            })?;
            
            let cmds = registry_lock.list_commands()?;
            timer.end();
            cmds
        }; // Lock is released here
        
        assert!(commands.contains(&"version".to_string()), "Registry should contain version command");
        assert!(commands.contains(&"help".to_string()), "Registry should contain help command");
        assert!(commands.contains(&"echo".to_string()), "Registry should contain echo command");
        assert!(commands.contains(&"exit".to_string()), "Registry should contain exit command");
        assert!(commands.contains(&"kill".to_string()), "Registry should contain kill command");
        
        // Now, execute a command that doesn't have circular dependencies
        let version_result = {
            let timer = TestLockTimer::new("execute_version");
            
            // Get command with lock
            let command = {
                let registry_lock = registry.lock().map_err(|e| {
                    let err_msg = format!("Failed to acquire lock on command registry: {}", e);
                    Box::<dyn Error>::from(err_msg)
                })?;
                
                // Clone the command while holding the lock
                let cmd = registry_lock.get_command("version")?;
                debug!("Test: Got version command, releasing lock");
                cmd
            }; // Lock is released here
            
            // Execute command without holding the lock
            debug!("Test: Executing version command without lock");
            let result = command.execute(&[]);
            timer.end();
            result
        };
        
        assert!(version_result.is_ok(), "Version command should execute successfully");
        assert!(version_result.unwrap().contains("Version"), "Version command should return version info");
        
        // Test echo command
        let echo_result = {
            let timer = TestLockTimer::new("execute_echo");
            
            // Get command with lock
            let command = {
                let registry_lock = registry.lock().map_err(|e| {
                    let err_msg = format!("Failed to acquire lock on command registry: {}", e);
                    Box::<dyn Error>::from(err_msg)
                })?;
                
                // Clone the command while holding the lock
                let cmd = registry_lock.get_command("echo")?;
                debug!("Test: Got echo command, releasing lock");
                cmd
            }; // Lock is released here
            
            // Execute command without holding the lock
            debug!("Test: Executing echo command without lock");
            let result = command.execute(&["hello".to_string(), "world".to_string()]);
            timer.end();
            result
        };
        
        assert!(echo_result.is_ok(), "Echo command should execute successfully");
        assert_eq!(echo_result.unwrap(), "Echo: hello world", "Echo command should return input");
        
        info!("Test: Completed test_create_command_registry successfully");
        Ok(())
    }
    
    #[test]
    fn test_default_factory() -> Result<(), Box<dyn Error>> {
        // Initialize logging for tests
        let _ = simple_logger::SimpleLogger::new()
            .with_level(log::LevelFilter::Debug)
            .init();
            
        info!("Test: Starting test_default_factory");
        let factory = DefaultCommandRegistryFactory;
        let registry = factory.create_registry()?;
        
        // First, verify commands exist in the registry
        let commands = {
            let timer = TestLockTimer::new("list_commands");
            let registry_lock = registry.lock().map_err(|e| {
                let err_msg = format!("Failed to acquire lock on command registry: {}", e);
                Box::<dyn Error>::from(err_msg)
            })?;
            
            let cmds = registry_lock.list_commands()?;
            timer.end();
            cmds
        }; // Lock is released here
        
        assert!(commands.contains(&"version".to_string()), "Registry should contain version command");
        assert!(commands.contains(&"help".to_string()), "Registry should contain help command");
        assert!(commands.contains(&"echo".to_string()), "Registry should contain echo command");
        assert!(commands.contains(&"exit".to_string()), "Registry should contain exit command");
        assert!(commands.contains(&"kill".to_string()), "Registry should contain kill command");
        
        // Get all command help for multiple commands at once (batched operation)
        let help_texts = {
            let timer = TestLockTimer::new("get_command_help");
            let registry_lock = registry.lock().map_err(|e| {
                let err_msg = format!("Failed to acquire lock on command registry: {}", e);
                Box::<dyn Error>::from(err_msg)
            })?;
            
            // Get help for multiple commands while holding the lock
            let mut help_map = std::collections::HashMap::new();
            for cmd_name in ["version", "echo", "help"] {
                match registry_lock.get_help(cmd_name) {
                    Ok(help) => { help_map.insert(cmd_name.to_string(), help); },
                    Err(e) => { debug!("Test: Error getting help for {}: {}", cmd_name, e); }
                }
            }
            
            timer.end();
            help_map
        }; // Lock is released here
        
        // Verify help content without holding locks
        assert!(help_texts.contains_key("version"), "Should have help for version command");
        assert!(help_texts.contains_key("echo"), "Should have help for echo command");
        assert!(help_texts.contains_key("help"), "Should have help for help command");
        
        info!("Test: Completed test_default_factory successfully");
        Ok(())
    }
    
    // New comprehensive test that checks factory creation with custom commands
    #[test]
    fn test_factory_with_custom_commands() -> Result<(), Box<dyn Error>> {
        // Initialize logging for tests
        let _ = simple_logger::SimpleLogger::new()
            .with_level(log::LevelFilter::Debug)
            .init();
            
        info!("Test: Starting test_factory_with_custom_commands");
        
        // Create a custom command
        #[derive(Debug, Clone)]
        struct CustomCommand;
        
        impl Command for CustomCommand {
            fn name(&self) -> &str {
                "custom"
            }
            
            fn description(&self) -> &str {
                "A custom test command"
            }
            
            fn execute(&self, _args: &[String]) -> CommandResult<String> {
                Ok("Custom command executed".to_string())
            }
            
            fn parser(&self) -> clap::Command {
                clap::Command::new("custom")
                    .about("A custom test command")
            }
            
            fn clone_box(&self) -> Box<dyn Command> {
                Box::new(self.clone())
            }
        }
        
        // Create registry with factory
        let factory = DefaultCommandRegistryFactory;
        let registry = factory.create_registry()?;
        
        // Register custom command
        {
            let timer = TestLockTimer::new("register_custom_command");
            let registry_lock = registry.lock().map_err(|e| {
                let err_msg = format!("Failed to acquire lock on command registry: {}", e);
                Box::<dyn Error>::from(err_msg)
            })?;
            
            registry_lock.register("custom", Arc::new(CustomCommand))?;
            timer.end();
        } // Lock is released here
        
        // Execute the custom command
        let custom_result = {
            let timer = TestLockTimer::new("execute_custom_command");
            
            // Get command with lock
            let command = {
                let registry_lock = registry.lock().map_err(|e| {
                    let err_msg = format!("Failed to acquire lock on command registry: {}", e);
                    Box::<dyn Error>::from(err_msg)
                })?;
                
                // Clone the command while holding the lock
                let cmd = registry_lock.get_command("custom")?;
                debug!("Test: Got custom command, releasing lock");
                cmd
            }; // Lock is released here
            
            // Execute command without holding the lock
            debug!("Test: Executing custom command without lock");
            let result = command.execute(&[]);
            timer.end();
            result
        };
        
        assert!(custom_result.is_ok(), "Custom command should execute successfully");
        assert_eq!(custom_result.unwrap(), "Custom command executed", "Custom command should return expected output");
        
        info!("Test: Completed test_factory_with_custom_commands successfully");
        Ok(())
    }
} 