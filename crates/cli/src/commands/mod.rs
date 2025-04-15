//! CLI commands for the Squirrel platform
//!
//! This module contains the implementation of CLI commands that
//! provide functionality for the Squirrel command-line interface.

pub mod config_command;
pub mod help_command;
pub mod version_command;
pub mod status_command;
pub mod plugin_command;
pub mod secrets_command;
pub mod executor;
pub mod mcp_command;
pub mod registry;
pub mod context;
pub mod run_command;
pub mod mcp;
pub mod adapter;
pub mod error;
pub mod monitoring_command;
#[cfg(any(test, feature = "testing"))]
pub mod test_command;

pub use config_command::ConfigCommand;
pub use help_command::HelpCommand;
pub use version_command::VersionCommand;
pub use status_command::StatusCommand;
pub use plugin_command::PluginCommand;
pub use secrets_command::SecretsCommand;
pub use executor::ExecutionContext;
pub use mcp_command::McpCommand;
pub use run_command::RunCommand;
pub use error::CommandError;

use clap::{Command as ClapCommand, Arg, ArgAction};
use std::sync::Arc;
use log::debug;
use std::cell::RefCell;
use crate::commands::adapter::error::AdapterResult;
use tokio::sync::Mutex;

use ::commands::Command;
use ::commands::CommandRegistry;

// Thread-local storage for execution context
thread_local! {
    static EXECUTION_CONTEXT: RefCell<Option<Arc<ExecutionContext>>> = const { RefCell::new(None) };
}

/// Register commands in the registry
pub fn register_commands(registry: &mut ::commands::CommandRegistry) -> Result<(), CommandError> {
    debug!("Registering commands");
    
    // Register built-in commands
    registry.register("help", Arc::new(HelpCommand::new()))?;
    registry.register("version", Arc::new(VersionCommand::new()))?;
    registry.register("config", Arc::new(ConfigCommand::new()))?;
    registry.register("status", Arc::new(StatusCommand::new()))?;
    registry.register("run", Arc::new(RunCommand::new()))?;
    
    // Use the cfg attribute for test_command
    #[cfg(any(test, feature = "testing"))]
    registry.register("test", Arc::new(test_command::SimpleTestCommand::new("test".to_string(), "Test command".to_string())))?;
    
    registry.register("secrets", Arc::new(SecretsCommand::new()))?;
    registry.register("mcp", Arc::new(McpCommand::new()))?;
    
    // Use the MonitoringCommand's register method which handles registration correctly
    monitoring_command::MonitoringCommand::register(registry)?;
    
    registry.register("plugin", Arc::new(PluginCommand::new()))?;
    
    debug!("Commands registered successfully");
    
    Ok(())
}

/// Create a new command registry with all commands registered
pub fn create_command_registry() -> AdapterResult<Arc<Mutex<CommandRegistry>>> {
    debug!("Creating command registry");
    
    let mut registry = CommandRegistry::new();
    register_commands(&mut registry)?;
    
    let registry = Arc::new(Mutex::new(registry));
    debug!("Command registry created successfully");
    
    Ok(registry)
}

/// Create a new CLI application
pub fn create_cli() -> ClapCommand {
    let mut app = ClapCommand::new("squirrel")
        .version(env!("CARGO_PKG_VERSION"))
        .author("DataScienceBioLab")
        .about("Squirrel CLI: The next-generation data science tool")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Print verbose output")
                .action(ArgAction::SetTrue),
        );
    
    // Add subcommands - but avoid adding a duplicate help command
    // since the help command is already built into clap
    // app = app.subcommand(HelpCommand::new().parser());
    app = app.subcommand(VersionCommand::new().parser());
    app = app.subcommand(StatusCommand::new().parser());
    app = app.subcommand(ConfigCommand::new().parser());
    app = app.subcommand(PluginCommand::new().parser());
    app = app.subcommand(SecretsCommand::new().parser());
    app = app.subcommand(McpCommand::new().parser());
    app = app.subcommand(RunCommand::new().parser());
    
    app
}

/// Create a CLI adapter for command execution
pub fn create_adapter(adapter_type: adapter::AdapterType) -> AdapterResult<Arc<dyn adapter::CommandAdapterTrait>> {
    debug!("Creating adapter of type: {:?}", adapter_type);
    
    match adapter_type {
        adapter::AdapterType::Registry => {
            let registry = create_command_registry()?;
            let adapter = adapter::CommandRegistryAdapter::new(registry);
            Ok(Arc::new(adapter))
        },
        adapter::AdapterType::Mcp => {
            let registry = create_command_registry()?;
            let registry_adapter = Arc::new(adapter::CommandRegistryAdapter::new(registry));
            
            // Create a basic auth provider
            let auth_provider = Arc::new(adapter::BasicAuthProvider::new());
            
            let adapter = adapter::McpCommandAdapter::new(auth_provider, registry_adapter);
            Ok(Arc::new(adapter))
        },
        adapter::AdapterType::Plugin => {
            let registry = create_command_registry()?;
            let adapter = adapter::CommandsPluginAdapter::new(registry);
            Ok(Arc::new(adapter))
        }
    }
}

/// Creates a CLI instance from the command registry
pub fn create_cli_from_registry(registry: &registry::CommandRegistry) -> registry::cli::Cli {
    registry::cli::Cli::new(registry)
} 
