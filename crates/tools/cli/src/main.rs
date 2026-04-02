// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: CLI interaction mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Squirrel CLI — command-line interface for the Squirrel AI primal.

use serde_json::json;
use squirrel_cli::commands::registry::CommandRegistry;
use squirrel_cli::commands::{executor::ExecutionContext, register_commands};
use std::env;
use std::process;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Squirrel CLI application entry point
#[tokio::main]
#[expect(clippy::too_many_lines, reason = "CLI main; refactor planned")]
async fn main() {
    // Set up logger (tracing with env filter)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug")),
        )
        .init();

    info!("Starting Squirrel CLI");

    // Create command registry
    let registry = CommandRegistry::new();

    // Register built-in commands
    if let Err(error) = register_commands() {
        warn!("Failed to register built-in commands: {error}");
        info!("Continuing without built-in commands...");
    } else {
        debug!("Built-in commands registered successfully");
    }

    // Create Arc-wrapped registry for sharing
    let registry_arc = Arc::new(registry);

    // Initialize plugin system
    info!("Initializing plugin system...");

    // Initialize plugins properly
    match squirrel_cli::plugins::initialize_plugins().await {
        Ok(()) => {
            info!("Plugin system initialized successfully");
        }
        Err(e) => {
            warn!("Failed to initialize plugin system: {e}");
            info!("Continuing without plugins...");
        }
    }

    // Load and start plugins
    let plugin_manager = squirrel_cli::plugins::state::get_plugin_manager();
    let mut plugin_manager_guard = plugin_manager.lock().await;

    // Load all discovered plugins
    let plugin_names: Vec<String> = plugin_manager_guard
        .list_plugins()
        .iter()
        .map(|p| p.metadata().name.clone())
        .collect();

    for plugin_name in &plugin_names {
        match plugin_manager_guard.load_plugin(plugin_name) {
            Ok(()) => {
                info!("Successfully loaded plugin: {plugin_name}");
            }
            Err(e) => {
                warn!("Failed to load plugin '{plugin_name}': {e}");
            }
        }
    }

    // Start all loaded plugins
    match plugin_manager_guard.start_plugins() {
        Ok(()) => {
            info!("All plugins started successfully");
        }
        Err(e) => {
            warn!("Failed to start some plugins: {e}");
        }
    }

    // Register plugin commands with the registry
    match plugin_manager_guard.register_plugin_commands(&registry_arc) {
        Ok(()) => {
            info!("Plugin commands registered successfully");
        }
        Err(e) => {
            warn!("Failed to register plugin commands: {e}");
        }
    }

    // Release the plugin manager guard
    drop(plugin_manager_guard);

    // Create CLI app - simplified for now
    let mut app = clap::Command::new("squirrel")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Squirrel CLI - A powerful command-line tool");

    // Create execution context
    let _execution_context = Arc::new(ExecutionContext::new());

    // Get command-line arguments
    let args: Vec<String> = env::args().collect();

    // Parse flags
    let verbose_mode = args.iter().any(|arg| arg == "--verbose" || arg == "-v");

    // Check for output format flag
    let use_json_output = args.contains(&"--output".to_string())
        && args
            .iter()
            .position(|a| a == "--output")
            .is_some_and(|i| args.get(i + 1).is_some_and(|v| v == "json"));

    // Find the first non-flag argument (command)
    let command = args
        .iter()
        .skip(1)
        .find(|arg| !arg.starts_with('-'))
        .map(String::as_str);

    if args.len() <= 1 || command.is_none() {
        // No arguments or only flags, just print help
        println!("Available commands:");
        println!("{}", app.render_help());
    } else if let Some(cmd) = command {
        // Handle the command properly
        match cmd {
            "version" => {
                println!("Squirrel CLI Version: {}", env!("CARGO_PKG_VERSION"));
            }
            "status" => {
                let pid = std::process::id();
                let rss_kb = read_proc_rss_kb();
                let env_name =
                    std::env::var("SQUIRREL_ENV").unwrap_or_else(|_| "development".into());
                let socket_info = squirrel_cli::status::socket_status();

                if use_json_output {
                    let status_json = json!({
                        "version": env!("CARGO_PKG_VERSION"),
                        "pid": pid,
                        "environment": env_name,
                        "memory_rss_kb": rss_kb,
                        "socket": socket_info,
                    });
                    match serde_json::to_string_pretty(&status_json) {
                        Ok(json_string) => println!("{json_string}"),
                        Err(e) => {
                            eprintln!("Error serializing status to JSON: {e}");
                            process::exit(1);
                        }
                    }
                } else if verbose_mode {
                    println!("Squirrel Status (verbose)");
                    println!("  Version:     {}", env!("CARGO_PKG_VERSION"));
                    println!("  PID:         {pid}");
                    println!("  Environment: {env_name}");
                    println!(
                        "  Memory RSS:  {} KB",
                        rss_kb.map_or_else(|| "N/A".to_string(), |k| k.to_string())
                    );
                    println!("  Socket:      {socket_info}");
                } else {
                    println!("Squirrel v{} (pid {pid})", env!("CARGO_PKG_VERSION"));
                    println!("  Socket: {socket_info}");
                }
            }
            "config" => {
                // Find the subcommand if any
                let subcommand = args
                    .iter()
                    .skip(1)
                    .skip_while(|arg| *arg != "config")
                    .skip(1)
                    .find(|arg| !arg.starts_with('-'))
                    .map(String::as_str);

                if subcommand == Some("list") {
                    if use_json_output {
                        // Output in JSON format
                        let config_json = json!({
                            "plugins_dir": "~/.squirrel/plugins",
                            "log_level": "info",
                            "auto_update": true,
                            "default_format": "table"
                        });
                        match serde_json::to_string_pretty(&config_json) {
                            Ok(json_string) => println!("{json_string}"),
                            Err(e) => {
                                eprintln!("Error serializing config to JSON: {e}");
                                process::exit(1);
                            }
                        }
                    } else {
                        println!("Configuration settings");
                        println!("\nCurrent Squirrel CLI Configuration:");
                        println!("  - plugins_dir: ~/.squirrel/plugins");
                        println!("  - log_level: info");
                        println!("  - auto_update: true");
                        println!("  - default_format: table");
                    }
                } else {
                    println!("Squirrel CLI Configuration");
                    println!("\nManage Squirrel CLI configuration settings.");
                    println!("Available actions:");
                    println!("  - get: Get a configuration value");
                    println!("  - set: Set a configuration value");
                    println!("  - list: Show all configuration values");
                    println!("  - import: Import configuration from a file");
                    println!("  - export: Export configuration to a file");
                    println!("\nUsage: squirrel config [ACTION] [OPTIONS]");
                }
            }
            "run" => {
                println!("Run Commands with Squirrel CLI");
                println!("\nExecute predefined or custom command scripts.");
                println!("Usage: squirrel run [SCRIPT_NAME] [OPTIONS]");
                println!("\nExamples:");
                println!("  squirrel run hello-world");
                println!("  squirrel run data-processing --input=file.csv --output=results.json");
            }
            "mcp" => {
                println!("Machine Context Protocol (MCP) Client");
                println!("\nMCP is a protocol for machines to exchange context information.");
                println!("Available actions:");
                println!("  - connect: Connect to an MCP server");
                println!("  - disconnect: Disconnect from an MCP server");
                println!("  - send: Send a message to an MCP server");
                println!("  - status: Check the status of the MCP connection");
                println!("\nFor more details, use the help command: squirrel mcp --help");
            }
            "plugin" => {
                println!("Plugin Management for Squirrel CLI");
                println!("\nPlugins extend the functionality of Squirrel CLI.");
                println!("Available actions:");
                println!("  - install: Install a plugin from a URL or local path");
                println!("  - uninstall: Remove an installed plugin");
                println!("  - list: Show all installed plugins");
                println!("  - enable: Enable a disabled plugin");
                println!("  - disable: Temporarily disable a plugin");
                println!("  - update: Update a plugin to the latest version");
                println!("\nFor more details, use the help command: squirrel plugin --help");
            }
            "help" => {
                // Find the subcommand if any
                let subcommand = args
                    .iter()
                    .skip(1)
                    .skip_while(|arg| *arg != "help")
                    .skip(1)
                    .find(|arg| !arg.starts_with('-'))
                    .map(String::as_str);

                if subcommand == Some("status") {
                    println!("Help for status command:");
                    println!("Usage: squirrel status [--verbose]");
                    println!("\nDisplays information about the current state of Squirrel CLI.");
                } else {
                    println!("Available commands:");
                    println!("{}", app.render_help());
                }
            }
            cmd => {
                // Invalid command
                eprintln!("Error: Command '{cmd}' not found");
                eprintln!("Run 'squirrel help' for a list of available commands");
                process::exit(1);
            }
        }
    } else {
        // Fallback: print help if no command is found
        println!("Available commands:");
        println!("{}", app.render_help());
    }
}

/// Read RSS from `/proc/self/status` (Linux).  Returns `None` on other platforms.
fn read_proc_rss_kb() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            return rest
                .trim()
                .strip_suffix("kB")
                .and_then(|s| s.trim().parse().ok());
        }
    }
    None
}
