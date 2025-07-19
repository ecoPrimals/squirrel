//! Main executable for the Squirrel CLI.

use squirrel_cli::commands::registry::CommandRegistry;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use std::env;
use std::process;
use std::sync::Arc;

// Add log crate for consistent logging
use log::{debug, error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();

    // Initialize command registry with built-in commands
    let mut registry = CommandRegistry::new();
    
    // Register built-in commands to make registry useful
    registry.register_builtin_commands().await.unwrap_or_else(|e| {
        eprintln!("Warning: Failed to register built-in commands: {}", e);
    });
    
    // Register additional shell-specific commands
    registry.register_shell_commands().await;

    // Print interactive shell banner with command count
    println!("🐿️ Squirrel Interactive Shell");
    println!("Loaded {} commands. Type 'help' for available commands, 'exit' to quit", 
             registry.command_count());
    println!();

    loop {
        // Print prompt
        print!("squirrel> ");
        io::stdout().flush()?;

        // Read user input
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();
                
                // Handle built-in shell commands first
                match input {
                    "exit" | "quit" => {
                        println!("Goodbye! 🐿️");
                        break;
                    }
                    "help" => {
                        println!("Available commands:");
                        println!("  help     - Show this help message");
                        println!("  exit     - Exit the shell");
                        println!("  clear    - Clear the screen");
                        println!("  status   - Show registry status");
                        println!();
                        
                        // List registered commands from registry
                        let commands = registry.list_commands().await;
                        if !commands.is_empty() {
                            println!("Registered commands:");
                            for command in commands {
                                println!("  {}     - {}", command.name, command.description);
                            }
                        }
                    }
                    "clear" => {
                        // Clear screen
                        print!("\x1B[2J\x1B[1;1H");
                        io::stdout().flush()?;
                    }
                    "status" => {
                        // Show registry status using the registry
                        let status = registry.get_status().await;
                        println!("Registry Status:");
                        println!("  Commands loaded: {}", status.command_count);
                        println!("  Memory usage: {} KB", status.memory_usage_kb);
                        println!("  Uptime: {} seconds", status.uptime_seconds);
                    }
                    "" => continue,
                    _ => {
                        // Try to execute command through registry
                        match registry.execute_command(input).await {
                            Ok(result) => {
                                println!("Command output: {}", result.output);
                                if let Some(error) = result.error {
                                    eprintln!("Warning: {}", error);
                                }
                            }
                            Err(e) => {
                                println!("Unknown command: {}", input);
                                println!("Error: {}", e);
                                println!("Type 'help' for available commands");
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }

    Ok(())
}
