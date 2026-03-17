// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)]
// CLI Application Example
//!
//! This example demonstrates a complete command-line application built using the adapter pattern,
//! featuring authentication, role-based access control, and interactive command execution.

use adapter_pattern_examples::{
    Auth, Command, CommandAdapter, CommandError, CommandResult, McpAdapter, RegistryAdapter,
    UserRole,
};
use async_trait::async_trait;
use std::env;
use std::fmt::Debug;
use std::sync::Arc;

/// Version command that displays the application version
#[derive(Debug)]
struct VersionCommand {
    name: String,
    description: String,
    version: String,
}

impl VersionCommand {
    fn new(version: &str) -> Self {
        Self {
            name: "version".to_string(),
            description: "Display the application version".to_string(),
            version: version.to_string(),
        }
    }
}

#[async_trait]
impl Command for VersionCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, _args: Vec<String>) -> CommandResult<String> {
        Ok(format!("CLI App Version {}", self.version))
    }
}

/// Help command that displays help information about available commands
#[derive(Debug)]
struct HelpCommand {
    name: String,
    description: String,
    adapter: Arc<RegistryAdapter>,
}

impl HelpCommand {
    fn new(adapter: Arc<RegistryAdapter>) -> Self {
        Self {
            name: "help".to_string(),
            description: "Display help information".to_string(),
            adapter,
        }
    }
}

#[async_trait]
impl Command for HelpCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, args: Vec<String>) -> CommandResult<String> {
        if args.is_empty() {
            // List all available commands
            let commands = self.adapter.list_commands().await?;
            let mut help_text = String::from("Available commands:\n");

            for cmd in commands {
                if let Ok(cmd_help) = self.adapter.get_help(&cmd).await {
                    use std::fmt::Write;
                    let _ = writeln!(help_text, "  {cmd_help}");
                }
            }

            help_text
                .push_str("\nUse 'help <command>' for more information about a specific command.");
            Ok(help_text)
        } else {
            // Get help for a specific command
            let command = &args[0];
            self.adapter.get_help(command).await.map_or_else(
                |_| {
                    Err(CommandError::NotFound(format!(
                        "Command '{command}' not found"
                    )))
                },
                Ok,
            )
        }
    }
}

/// Echo command that echoes back the arguments
#[derive(Debug)]
struct EchoCommand {
    name: String,
    description: String,
}

impl EchoCommand {
    fn new() -> Self {
        Self {
            name: "echo".to_string(),
            description: "Echo back the arguments".to_string(),
        }
    }
}

#[async_trait]
impl Command for EchoCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, args: Vec<String>) -> CommandResult<String> {
        if args.is_empty() {
            return Ok("Echo: No arguments provided".to_string());
        }
        Ok(format!("Echo: {}", args.join(" ")))
    }
}

/// A greeter command with different greetings
#[derive(Debug)]
struct GreetCommand {
    name: String,
    description: String,
}

impl GreetCommand {
    fn new() -> Self {
        Self {
            name: "greet".to_string(),
            description: "Greet someone (usage: greet [formal|casual] <name>)".to_string(),
        }
    }
}

#[async_trait]
impl Command for GreetCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, args: Vec<String>) -> CommandResult<String> {
        if args.is_empty() {
            return Err(CommandError::ExecutionFailed(
                "No name provided. Usage: greet [formal|casual] <name>".to_string(),
            ));
        }

        if args.len() == 1 {
            // Default to casual greeting
            return Ok(format!("Hi, {}!", args[0]));
        }

        let style = &args[0];
        let name = &args[1];

        match style.as_str() {
            "formal" => Ok(format!("Good day to you, {name}.")),
            "casual" => Ok(format!("Hey {name}! What's up?")),
            _ => Err(CommandError::ExecutionFailed(format!(
                "Unknown greeting style: {style}. Use 'formal' or 'casual'."
            ))),
        }
    }
}

// A secure command that requires authentication
#[derive(Debug)]
struct SecureCommand {
    name: String,
    description: String,
}

impl SecureCommand {
    fn new() -> Self {
        Self {
            name: "secure".to_string(),
            description: "A secure command that requires authentication".to_string(),
        }
    }
}

#[async_trait]
impl Command for SecureCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, _args: Vec<String>) -> CommandResult<String> {
        Ok("You have access to secure information!".to_string())
    }
}

// Application context
struct CliApp {
    registry_adapter: Arc<RegistryAdapter>,
    mcp_adapter: McpAdapter,
    version: String,
}

impl CliApp {
    fn new(version: &str) -> Self {
        let registry_adapter = Arc::new(RegistryAdapter::new());
        let mcp_adapter = McpAdapter::new();

        Self {
            registry_adapter,
            mcp_adapter,
            version: version.to_string(),
        }
    }

    fn initialize(&mut self) -> CommandResult<()> {
        // Register basic commands
        let version_cmd = VersionCommand::new(&self.version);
        let echo_cmd = EchoCommand::new();
        let greet_cmd = GreetCommand::new();
        let secure_cmd = SecureCommand::new();

        // Register with registry adapter
        self.registry_adapter
            .register_command(Arc::new(version_cmd))?;
        self.registry_adapter.register_command(Arc::new(echo_cmd))?;
        self.registry_adapter
            .register_command(Arc::new(greet_cmd))?;

        // Use the adapter for the help command
        let help_cmd = HelpCommand::new(self.registry_adapter.clone());
        self.registry_adapter.register_command(Arc::new(help_cmd))?;

        // Since we're having issues with the MCP adapter, register the secure command directly in the registry
        // but we'll still protect it with authentication in the execute method
        self.registry_adapter
            .register_command(Arc::new(secure_cmd))?;

        // Add users
        let mut mcp_adapter = self.mcp_adapter.clone();
        mcp_adapter.add_user("user", "password", vec![UserRole::User]);
        self.mcp_adapter = mcp_adapter;

        Ok(())
    }

    async fn execute(&self, command: &str, args: Vec<String>) -> CommandResult<String> {
        // Special case for secure command - require authentication
        if command == "secure" {
            return Err(CommandError::AuthenticationFailed(
                "Authentication required for this command. Use 'login <username> <password> secure'".to_string()
            ));
        }

        // Normal execution for other commands
        match self
            .registry_adapter
            .execute_command(command, args.clone())
            .await
        {
            Ok(result) => Ok(result),
            Err(e) => Err(e),
        }
    }

    async fn execute_with_auth(
        &self,
        command: &str,
        args: Vec<String>,
        auth: Auth,
    ) -> CommandResult<String> {
        println!("Debug - execute_with_auth called with command: {command} and auth");

        // Special case for secure command - check authentication
        if command == "secure" {
            match &auth {
                Auth::User(username, password) => {
                    // Try to authenticate
                    // We'll implement a simple check here
                    if (username == "admin" || username == "user") && password == "password" {
                        // User is authenticated, execute the command
                        return self.registry_adapter.execute_command(command, args).await;
                    }
                    return Err(CommandError::AuthenticationFailed(
                        "Invalid username or password".to_string(),
                    ));
                }
                _ => {
                    return Err(CommandError::AuthenticationFailed(
                        "Authentication required for this command".to_string(),
                    ));
                }
            }
        }

        // For other commands, just execute normally
        self.registry_adapter.execute_command(command, args).await
    }

    async fn available_commands(&self, auth: Auth) -> CommandResult<Vec<String>> {
        // Get commands from both adapters
        let registry_commands = self.registry_adapter.list_commands().await?;
        let mcp_commands = self.mcp_adapter.get_available_commands(auth).await?;

        // Combine them
        let mut all_commands = registry_commands;
        all_commands.extend(mcp_commands);
        all_commands.sort();

        Ok(all_commands)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    println!("Raw arguments: {args:?}");

    // Create and initialize the application
    let mut app = CliApp::new("1.0.0");
    app.initialize()?;

    // Print all available commands
    let all_commands = app.available_commands(Auth::None).await?;
    println!("Public commands: {all_commands:?}");
    let user_commands = app
        .available_commands(Auth::User("user".to_string(), "password".to_string()))
        .await?;
    println!("User commands: {user_commands:?}");

    if args.len() <= 1 {
        // No command specified, show help
        println!("Usage: {} <command> [args...]", args[0]);
        println!("Use '{} help' for a list of available commands", args[0]);
        return Ok(());
    }

    let command = &args[1];
    let command_args = if args.len() > 2 {
        args[2..].to_vec()
    } else {
        vec![]
    };

    // Special case for login
    if command == "login" {
        println!("Login command detected with args: {command_args:?}");

        if command_args.len() < 2 {
            println!("Usage: {} login <username> <password>", args[0]);
            return Ok(());
        }

        let username = &command_args[0];
        let password = &command_args[1];
        let auth = Auth::User(username.clone(), password.clone());

        // List available commands with this authentication
        let available = app.available_commands(auth.clone()).await?;
        println!("Logged in as: {username}");
        println!("Available commands: {available:?}");

        // If further command args are present, execute that command
        if command_args.len() > 2 {
            let subcommand = &command_args[2];
            println!("Executing subcommand: {subcommand} with auth");

            let subcommand_args = if command_args.len() > 3 {
                command_args[3..].to_vec()
            } else {
                vec![]
            };

            match app
                .execute_with_auth(subcommand, subcommand_args, auth)
                .await
            {
                Ok(result) => println!("{result}"),
                Err(e) => println!("Error: {e}"),
            }
        }

        return Ok(());
    }

    // Execute the command without authentication
    println!("Executing command: {command} with args: {command_args:?}");
    match app.execute(command, command_args).await {
        Ok(result) => println!("{result}"),
        Err(e) => println!("Error: {e}"),
    }

    Ok(())
}
