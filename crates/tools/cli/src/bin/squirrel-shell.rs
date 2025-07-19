//! Main executable for the Squirrel CLI.

use squirrel_cli::commands::registry::CommandRegistry;
use std::collections::HashMap;
use std::env;
use std::process;
use std::sync::Arc;
use std::time::{Duration, Instant};

// Add log crate for consistent logging
use log::{debug, error, info, warn};

/// Helper struct to track lock timing
struct LockTimer {
    operation: String,
    start_time: Instant,
    warn_threshold: Duration,
}

impl LockTimer {
    fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            start_time: Instant::now(),
            warn_threshold: Duration::from_millis(100), // Warn if lock held for more than 100ms
        }
    }

    fn end(self) -> Duration {
        let duration = self.start_time.elapsed();
        debug!(
            "Lock operation '{}' completed in {:?}",
            self.operation, duration
        );

        if duration > self.warn_threshold {
            warn!(
                "Lock operation '{}' took {:?} - potential contention",
                self.operation, duration
            );
        }

        duration
    }
}

#[tokio::main]
async fn main() {
    // Initialize simple logger if not already initialized
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap_or_else(|_| {
            println!("Logger already initialized or failed to initialize");
        });

    info!("Squirrel CLI starting");
    println!("Squirrel CLI");

    // Run the application
    println!("Squirrel CLI v{}", env!("CARGO_PKG_VERSION"));

    // Application is ready
    info!("CLI application initialized");

    println!("Initializing command registry...");
    // Create a new command registry with built-in commands
    let registry = Arc::new(CommandRegistry::new());
    info!("Command registry initialized successfully");

    // Get command-line arguments
    let args: Vec<String> = env::args().collect();
    debug!("Command line arguments: {:?}", args);

    // If no command is provided, show usage
    if args.len() < 2 {
        println!("Usage: {} <command> [args...]", args[0]);
        println!("\nAvailable commands:");

        // Get all available commands
        let commands = registry.list_commands();

        if commands.is_empty() {
            println!("  No commands available");
        } else {
            for cmd in commands {
                println!("  {}: Command not yet implemented", cmd);
            }
        }

        info!("Help display completed, exiting");
        process::exit(1);
    }

    // Execute the command
    let command_name = &args[1];
    let command_args = &args[2..].to_vec();

    info!(
        "Executing command: {} with args: {:?}",
        command_name, command_args
    );
    println!(
        "Executing command: {} with args: {:?}",
        command_name, command_args
    );

    // Execute the command
    let result = registry.execute(command_name, command_args);

    match result {
        Ok(output) => {
            info!("Command '{}' executed successfully", command_name);
            println!("{}", output);
            process::exit(0);
        }
        Err(e) => {
            error!("Error executing command '{}': {}", command_name, e);
            eprintln!("Error executing command: {}", e);
            process::exit(1);
        }
    }
}
