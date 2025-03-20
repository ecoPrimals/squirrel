//! Main executable for the Squirrel CLI.

use clap::Parser;
use squirrel_core::Core;
// Removed unused import: use squirrel_commands::Command;
use std::env;
use std::process;

#[tokio::main]
async fn main() {
    println!("Squirrel CLI");
    
    // Run the application
    println!("Squirrel Core v{}", squirrel_core::build_info::version());
    
    // Create a core instance
    let _core = Core::new();

    // Create a new command registry with built-in commands
    let registry = match squirrel_commands::create_command_registry() {
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
        match registry.list_commands() {
            Ok(commands) => {
                for cmd in commands {
                    if let Ok(help) = registry.get_help(&cmd) {
                        println!("  {}", help);
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
    let command_args = &args[2..].to_vec();

    match registry.execute(command_name, command_args) {
        Ok(output) => {
            println!("{}", output);
            process::exit(0);
        },
        Err(e) => {
            eprintln!("Error executing command: {}", e);
            process::exit(1);
        }
    }
} 