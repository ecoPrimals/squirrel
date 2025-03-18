//! Main executable for the Squirrel Core library.

#[cfg(not(feature = "di-tests"))]
use squirrel_core::{Core, MCP};

#[cfg(feature = "di-tests")]
use squirrel_core::{Core, app::AppConfig};

#[cfg(not(feature = "di-tests"))]
mod commands;

#[cfg(not(feature = "di-tests"))]
use commands::CommandRegistry;

#[cfg(not(feature = "di-tests"))]
use std::env;

#[cfg(not(feature = "di-tests"))]
use std::process;

#[cfg(feature = "di-tests")]
fn main() {
    println!("Running with di-tests feature");
    
    // Create a simple app with our new structure
    let config = AppConfig::default();
    let _core = Core::new(config);
    
    println!("Core created successfully");
}

#[cfg(not(feature = "di-tests"))]
fn main() {
    println!("Running without di-tests feature");
    
    // Original implementation
    let _core = Core::new();
    println!("Core created successfully");

    // Create a new command registry with built-in commands
    let registry = match CommandRegistry::with_builtins() {
        Ok(registry) => registry,
        Err(e) => {
            eprintln!("Error initializing command registry: {}", e);
            process::exit(1);
        }
    };

    // Get command-line arguments
    let args: Vec<String> = env::args().collect();

    // If no command is provided, show usage
    if args.len() < 2 {
        println!("Usage: {} <command> [args...]", args[0]);
        println!("\nAvailable commands:");
        match registry.list() {
            Ok(commands) => {
                for cmd in commands {
                    if let Ok(Some(command)) = registry.get(&cmd) {
                        println!("  {:<15} - {}", cmd, command.description());
                    }
                }
            }
            Err(e) => {
                eprintln!("Error listing commands: {}", e);
                process::exit(1);
            }
        }
        process::exit(1);
    }

    // Execute the command
    let command_name = &args[1];
    let command_args = args[1..].to_vec();

    match registry.execute(command_name, command_args) {
        Ok(_) => process::exit(0),
        Err(e) => {
            eprintln!("Error executing command: {}", e);
            process::exit(1);
        }
    }
}