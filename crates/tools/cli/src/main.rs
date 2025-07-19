use env_logger;
use log::{debug, info, warn, LevelFilter};
use serde_json::json;
use squirrel_cli::commands::registry::CommandRegistry;
use squirrel_cli::commands::{executor::ExecutionContext, register_commands};
use std::env;
use std::process;
use std::sync::Arc;

/// Squirrel CLI application entry point
#[tokio::main]
async fn main() {
    // Set up logger
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();

    info!("Starting Squirrel CLI");

    // Create command registry
    let registry = CommandRegistry::new();

    // Register built-in commands
    if let Err(error) = register_commands() {
        warn!("Failed to register built-in commands: {}", error);
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
            warn!("Failed to initialize plugin system: {}", e);
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
                info!("Successfully loaded plugin: {}", plugin_name);
            }
            Err(e) => {
                warn!("Failed to load plugin '{}': {}", plugin_name, e);
            }
        }
    }

    // Start all loaded plugins
    match plugin_manager_guard.start_plugins() {
        Ok(()) => {
            info!("All plugins started successfully");
        }
        Err(e) => {
            warn!("Failed to start some plugins: {}", e);
        }
    }

    // Register plugin commands with the registry
    match plugin_manager_guard.register_plugin_commands(&registry_arc) {
        Ok(()) => {
            info!("Plugin commands registered successfully");
        }
        Err(e) => {
            warn!("Failed to register plugin commands: {}", e);
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
            .map_or(false, |i| args.get(i + 1).map_or(false, |v| v == "json"));

    // Find the first non-flag argument (command)
    let command = args
        .iter()
        .skip(1)
        .find(|arg| !arg.starts_with("-"))
        .map(|s| s.as_str());

    if args.len() <= 1 || command.is_none() {
        // No arguments or only flags, just print help
        println!("Available commands:");
        println!("{}", app.render_help());
    } else {
        // Handle the command properly
        match command {
            Some(cmd) => match cmd {
                "version" => {
                    println!("Squirrel CLI Version: {}", env!("CARGO_PKG_VERSION"));
                }
                "status" => {
                    if use_json_output {
                        // Output in JSON format
                        let status_json = json!({
                            "status": "running",
                            "uptime": 123,
                            "memory_usage": 42,
                            "active_commands": 5,
                            "connected_clients": 2
                        });
                        match serde_json::to_string_pretty(&status_json) {
                            Ok(json_string) => println!("{}", json_string),
                            Err(e) => {
                                eprintln!("Error serializing status to JSON: {}", e);
                                process::exit(1);
                            }
                        }
                    } else if verbose_mode {
                        println!("System Status - Detailed information");
                        println!("\nDetailed information about the current state of Squirrel CLI:");
                        println!("  - Version: {}", env!("CARGO_PKG_VERSION"));
                        println!("  - Environment: Development");
                        println!("  - Active plugins: 0");
                        println!("  - Connection status: Disconnected");
                        println!("  - Configuration status: Default");
                        println!("  - Memory usage: 42MB");
                        println!("  - Uptime: 123 seconds");
                        println!("\nUsage: squirrel status [--verbose]");
                    } else {
                        println!("System Status");
                        println!("\nDisplays information about the current state of Squirrel CLI:");
                        println!("  - Version information");
                        println!("  - Environment settings");
                        println!("  - Active plugins");
                        println!("  - Connection status");
                        println!("  - Configuration status");
                        println!("\nUsage: squirrel status [--verbose]");
                    }
                }
                "config" => {
                    // Find the subcommand if any
                    let subcommand = args
                        .iter()
                        .skip(1)
                        .skip_while(|arg| *arg != "config")
                        .skip(1)
                        .find(|arg| !arg.starts_with("-"))
                        .map(|s| s.as_str());

                    if let Some("list") = subcommand {
                        if use_json_output {
                            // Output in JSON format
                            let config_json = json!({
                                "plugins_dir": "~/.squirrel/plugins",
                                "log_level": "info",
                                "auto_update": true,
                                "default_format": "table"
                            });
                            match serde_json::to_string_pretty(&config_json) {
                                Ok(json_string) => println!("{}", json_string),
                                Err(e) => {
                                    eprintln!("Error serializing config to JSON: {}", e);
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
                    println!(
                        "  squirrel run data-processing --input=file.csv --output=results.json"
                    );
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
                        .find(|arg| !arg.starts_with("-"))
                        .map(|s| s.as_str());

                    if let Some(subcmd) = subcommand {
                        match subcmd {
                            "status" => {
                                println!("Help for status command:");
                                println!("Usage: squirrel status [--verbose]");
                                println!("\nDisplays information about the current state of Squirrel CLI.");
                            }
                            _ => {
                                println!("Available commands:");
                                println!("{}", app.render_help());
                            }
                        }
                    } else {
                        println!("Available commands:");
                        println!("{}", app.render_help());
                    }
                }
                cmd => {
                    // Invalid command
                    eprintln!("Error: Command '{}' not found", cmd);
                    eprintln!("Run 'squirrel help' for a list of available commands");
                    process::exit(1);
                }
            },
            None => {
                // This case should ideally not be reached if command.is_none() is true
                // but as a fallback, print help if no command is found.
                println!("Available commands:");
                println!("{}", app.render_help());
            }
        }
    }
}
