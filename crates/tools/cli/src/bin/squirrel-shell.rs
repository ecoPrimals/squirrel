// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

#![allow(
    clippy::needless_continue,
    clippy::if_not_else,
    clippy::uninlined_format_args,
    clippy::redundant_closure_for_method_calls
)]

//! Main executable for the Squirrel CLI.

use squirrel_cli::commands::registry::CommandRegistry;
use std::io::{self, Write};

// Add log crate for consistent logging
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();

    // Initialize command registry with built-in commands
    let registry = CommandRegistry::new();

    // Register some basic commands
    info!(
        "Squirrel shell initialized with {} commands",
        registry.list_commands().len()
    );

    // Print interactive shell banner with command count
    println!("🐿️ Squirrel Interactive Shell");
    println!(
        "Type 'help' for available commands or 'exit' to quit. ({} commands loaded)",
        registry.list_commands().len()
    );
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
                        // List available commands
                        let commands = registry.list_commands();
                        if !commands.is_empty() {
                            println!("Available commands:");
                            for command_name in commands {
                                println!("  {}", command_name);
                            }
                        } else {
                            println!("No commands are currently registered.");
                        }
                    }
                    "clear" => {
                        // Clear screen
                        print!("\x1B[2J\x1B[1;1H");
                        io::stdout().flush()?;
                    }
                    "status" => {
                        // Show registry status
                        let command_count = registry.list_commands().len();
                        println!("Registry Status:");
                        println!("  Commands loaded: {}", command_count);
                        println!("  Available commands: {:?}", registry.list_commands());
                    }
                    "" => continue,
                    _ => {
                        // Try to get command from registry
                        let parts: Vec<&str> = input.split_whitespace().collect();
                        if let Some(command_name) = parts.first() {
                            if let Some(command) = registry.get_command(command_name) {
                                // Execute the command with arguments
                                let args: Vec<String> =
                                    parts[1..].iter().map(|s| s.to_string()).collect();
                                match command.execute(&args) {
                                    Ok(result) => {
                                        println!("Command output: {}", result);
                                    }
                                    Err(e) => {
                                        eprintln!("Command failed: {}", e);
                                    }
                                }
                            } else {
                                println!("Unknown command: {}", command_name);
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
