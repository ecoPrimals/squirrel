//! Entry point for the Squirrel CLI application.

use std::sync::{Arc, Mutex};

use log::{debug, error, warn, LevelFilter};
use squirrel_commands::CommandRegistry;
use squirrel_cli::commands::{create_cli, register_commands};
use squirrel_cli::plugins::initialize_plugins;

/// Squirrel CLI application
fn main() {
    // Set up logger
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();

    // Create command registry
    let registry = Arc::new(Mutex::new(CommandRegistry::new()));

    // Register commands
    {
        let mut reg = registry.lock().unwrap();
        register_commands(&mut reg);
        debug!("Commands registered successfully");
    }

    // Initialize plugin system
    if let Err(e) = initialize_plugins() {
        warn!("Plugin system initialization failed: {}", e);
    }

    // Run CLI
    let app = create_cli();
    match app.try_get_matches() {
        Ok(_matches) => {
            debug!("CLI execution completed successfully");
            // Process matches if needed
        },
        Err(e) => {
            error!("CLI execution failed: {}", e);
        }
    }
} 