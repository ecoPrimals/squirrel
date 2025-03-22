//! Entry point for the Squirrel CLI application.

use std::sync::Arc;
use std::{env, process};

use log::{debug, warn, info, error, LevelFilter};
use squirrel_commands::CommandRegistry;
use squirrel_cli::commands::{create_cli, register_commands, ExecutionContext};
use squirrel_cli::plugins::state::get_plugin_manager;

/// Squirrel CLI application entry point
#[tokio::main]
async fn main() {
    // Set up logger
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();

    // Create command registry
    let mut registry = CommandRegistry::new();

    // Register built-in commands
    register_commands(&mut registry);
    debug!("Built-in commands registered successfully");

    // Create Arc-wrapped registry for sharing
    let registry_arc = Arc::new(registry);

    // Initialize plugin system
    info!("Initializing plugin system...");
    let plugin_manager = get_plugin_manager();
    
    // Get plugin names from the list of installed plugins
    let plugin_names = {
        let plugin_manager_lock = plugin_manager.lock().unwrap();
        plugin_manager_lock.list_plugins()
            .iter()
            .map(|p| p.metadata().name.clone())
            .collect::<Vec<String>>()
    };
    
    info!("Loading {} installed plugins...", plugin_names.len());
    for plugin_name in &plugin_names {
        debug!("Loading plugin: {}", plugin_name);
        
        // Get a new lock for each operation
        let mut plugin_manager_lock = plugin_manager.lock().unwrap();
        match plugin_manager_lock.load_plugin(plugin_name) {
            Ok(_) => {
                info!("Successfully loaded plugin: {}", plugin_name);
            }
            Err(err) => {
                warn!("Failed to load plugin {}: {}", plugin_name, err);
            }
        }
    }
    
    // Register commands from loaded plugins
    debug!("Registering plugin commands...");
    {
        let plugin_manager_lock = plugin_manager.lock().unwrap();
        match plugin_manager_lock.register_plugin_commands(&registry_arc) {
            Ok(_) => {
                info!("Successfully registered plugin commands");
            }
            Err(err) => {
                warn!("Failed to register some plugin commands: {}", err);
            }
        }
    }
    
    // Start the plugins
    debug!("Starting plugins...");
    {
        let mut plugin_manager_lock = plugin_manager.lock().unwrap();
        match plugin_manager_lock.start_plugins() {
            Ok(_) => {
                info!("Successfully started plugins");
            }
            Err(err) => {
                warn!("Failed to start some plugins: {}", err);
            }
        }
    }

    // Create CLI app
    let app = create_cli();
    
    // Create execution context
    let execution_context = ExecutionContext::new(registry_arc);
    
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();
    
    // Parse command-line arguments
    let matches = match app.try_get_matches_from(args) {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to parse arguments: {}", e);
            process::exit(1);
        }
    };
    
    // Get the subcommand and execute it
    let (command_name, subcommand_matches) = matches.subcommand().unwrap();
    
    match execution_context.execute_command(command_name, subcommand_matches.clone()).await {
        Ok(_) => {
            info!("Command executed successfully");
        }
        Err(err) => {
            error!("Command execution failed: {}", err);
            process::exit(1);
        }
    }
    
    // Cleanup: Unload plugins
    debug!("Unloading plugins...");
    let _ = plugin_manager.lock().map(|mut plugin_manager| {
        if let Err(e) = plugin_manager.unload_plugins() {
            warn!("Failed to unload plugins: {}", e);
        } else {
            debug!("Plugins unloaded successfully");
        }
    });
    
    info!("Squirrel CLI execution completed");
} 