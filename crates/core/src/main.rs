//! Main entry point for the Squirrel application

use squirrel_core::{Core, MCP};

mod commands;
use commands::CommandRegistry;

use std::env;
use std::process;

fn main() {
    // Initialize core systems
    let _core = Core::new();
    let _mcp = MCP::default();

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