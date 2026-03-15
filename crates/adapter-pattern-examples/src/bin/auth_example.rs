// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

// Authentication Example
//!
//! This example demonstrates authentication and authorization patterns using the MCP adapter,
//! including role-based access control and command permissions.

use adapter_pattern_examples::{
    Auth, Command, CommandError, CommandResult, McpAdapter, TestCommand, UserRole,
};
use async_trait::async_trait;
use std::fmt::Debug;
use std::io::{self, Write};
use std::sync::Arc;

// Administrative command that requires admin privileges
#[derive(Debug)]
struct AdminCommand {
    name: String,
    description: String,
}

impl AdminCommand {
    fn new() -> Self {
        Self {
            name: "admin-config".to_string(),
            description: "Configure system settings (admin only)".to_string(),
        }
    }
}

#[async_trait]
impl Command for AdminCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, args: Vec<String>) -> CommandResult<String> {
        if args.is_empty() {
            return Ok(
                "Current configuration: debug=false, verbose=false, timeout=30s".to_string(),
            );
        }

        let setting = &args[0];
        let value = args.get(1);

        match setting.as_str() {
            "debug" => value.map_or_else(
                || Ok("Debug mode is currently: false".to_string()),
                |v| Ok(format!("Debug mode set to: {v}")),
            ),
            "verbose" => value.map_or_else(
                || Ok("Verbose logging is currently: false".to_string()),
                |v| Ok(format!("Verbose logging set to: {v}")),
            ),
            "timeout" => value.map_or_else(
                || Ok("Timeout is currently: 30 seconds".to_string()),
                |v| Ok(format!("Timeout set to: {v} seconds")),
            ),
            _ => Err(CommandError::ExecutionFailed(format!(
                "Unknown setting: {setting}"
            ))),
        }
    }
}

// User-level command that requires user privileges
#[derive(Debug)]
struct UserProfileCommand {
    name: String,
    description: String,
}

impl UserProfileCommand {
    fn new() -> Self {
        Self {
            name: "profile".to_string(),
            description: "Manage user profile (user level access)".to_string(),
        }
    }
}

#[async_trait]
impl Command for UserProfileCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, args: Vec<String>) -> CommandResult<String> {
        if args.is_empty() {
            return Ok("Profile commands: view, update, delete".to_string());
        }

        let action = &args[0];

        match action.as_str() {
            "view" => Ok("User profile: Name=John Doe, Email=john@example.com".to_string()),
            "update" => {
                if args.len() < 3 {
                    return Err(CommandError::ExecutionFailed(
                        "Usage: profile update <field> <value>".to_string(),
                    ));
                }
                Ok(format!(
                    "Updated profile field '{}' to '{}'",
                    args[1], args[2]
                ))
            }
            "delete" => Ok("Profile deleted. This cannot be undone.".to_string()),
            _ => Err(CommandError::ExecutionFailed(format!(
                "Unknown profile action: {action}"
            ))),
        }
    }
}

// Public command that anyone can access
#[derive(Debug)]
struct PublicInfoCommand {
    name: String,
    description: String,
}

impl PublicInfoCommand {
    fn new() -> Self {
        Self {
            name: "info".to_string(),
            description: "Get public information (available to everyone)".to_string(),
        }
    }
}

#[async_trait]
impl Command for PublicInfoCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, args: Vec<String>) -> CommandResult<String> {
        if args.is_empty() {
            return Ok("Available info topics: help, about, version".to_string());
        }

        let topic = &args[0];

        match topic.as_str() {
            "help" => Ok("For more detailed help, type 'help <command>'".to_string()),
            "about" => Ok("This is a demo of the Adapter Pattern for command systems.".to_string()),
            "version" => Ok("Version 1.0.0".to_string()),
            _ => Err(CommandError::ExecutionFailed(format!(
                "Unknown info topic: {topic}"
            ))),
        }
    }
}

// Simple helper for getting user input
fn prompt(message: &str) -> String {
    print!("{message}");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Authentication and Authorization Example ===\n");

    // Create MCP adapter
    let mut mcp_adapter = McpAdapter::new();

    // Register commands with different privilege levels
    mcp_adapter.register_command(Arc::new(AdminCommand::new()))?;
    mcp_adapter.register_command(Arc::new(UserProfileCommand::new()))?;
    mcp_adapter.register_command(Arc::new(PublicInfoCommand::new()))?;
    mcp_adapter.register_command(Arc::new(TestCommand::new(
        "test",
        "Test command",
        "Test result",
    )))?;

    // Setup permissions
    mcp_adapter.add_command_permissions("profile", vec![UserRole::User, UserRole::Admin]);

    // Add some users
    mcp_adapter.add_user("regularuser", "userpass", vec![UserRole::User]);
    mcp_adapter.add_user("guest", "guestpass", vec![UserRole::Guest]);

    // Simple interactive command loop
    let mut current_auth = Auth::None;
    let mut user_type = String::from("anonymous");

    println!("Commands are protected by different permission levels.");
    println!("- 'admin-config' requires admin privileges");
    println!("- 'profile' requires user privileges");
    println!("- 'info' and 'test' are public commands\n");

    loop {
        println!("\nLogged in as: {user_type}");

        // Show available commands
        let available_commands = mcp_adapter
            .get_available_commands(current_auth.clone())
            .await?;
        println!("Available commands: {available_commands:?}");

        let input = prompt("\nEnter command (or 'login', 'logout', 'quit'): ");

        match input.as_str() {
            "login" => {
                let username = prompt("Username: ");
                let password = prompt("Password: ");

                current_auth = Auth::User(username.clone(), password);
                user_type = username;

                println!("Logged in as {user_type}");
            }
            "logout" => {
                current_auth = Auth::None;
                user_type = String::from("anonymous");
                println!("Logged out");
            }
            "quit" => {
                println!("Goodbye!");
                break;
            }
            _ => {
                let parts: Vec<String> = input
                    .split_whitespace()
                    .map(std::string::ToString::to_string)
                    .collect();

                if parts.is_empty() {
                    continue;
                }

                let command = parts[0].clone();
                let args = if parts.len() > 1 {
                    parts[1..].to_vec()
                } else {
                    vec![]
                };

                match mcp_adapter
                    .execute_with_auth(&command, args, current_auth.clone())
                    .await
                {
                    Ok(result) => println!("Result: {result}"),
                    Err(e) => println!("Error: {e}"),
                }
            }
        }
    }

    Ok(())
}
