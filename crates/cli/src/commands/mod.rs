//! CLI commands for the Squirrel platform
//!
//! This module contains the implementation of CLI commands that
//! provide functionality for the Squirrel command-line interface.

pub mod config_command;
pub mod help_command;
pub mod version_command;
pub mod status_command;
pub mod plugin_command;

pub use config_command::ConfigCommand;
pub use help_command::HelpCommand;
pub use version_command::VersionCommand;
pub use status_command::StatusCommand;
pub use plugin_command::PluginCommand;

use clap::{Command as ClapCommand, Arg, ArgAction};
use squirrel_commands::{Command, CommandRegistry};

/// Create the command-line interface
///
/// This function creates the command-line interface for the Squirrel CLI.
pub fn create_cli() -> ClapCommand {
    let output_format_args = vec![
        Arg::new("json")
            .long("json")
            .help("Output in JSON format")
            .action(ArgAction::SetTrue)
            .conflicts_with_all(["yaml", "table"]),
        Arg::new("yaml")
            .long("yaml")
            .help("Output in YAML format")
            .action(ArgAction::SetTrue)
            .conflicts_with_all(["json", "table"]),
        Arg::new("table")
            .long("table")
            .help("Output in table format")
            .action(ArgAction::SetTrue)
            .conflicts_with_all(["json", "yaml"]),
    ];

    ClapCommand::new("squirrel")
        .version(env!("CARGO_PKG_VERSION"))
        .author("DataScienceBioLab")
        .about("Squirrel CLI - Machine Context Protocol Tool")
        .subcommand(
            ClapCommand::new("help")
                .about("Show help information")
                .arg(
                    Arg::new("command")
                        .help("Command to show help for")
                        .required(false)
                )
                .args(&output_format_args)
        )
        .subcommand(
            ClapCommand::new("version")
                .about("Show version information")
                .args(&output_format_args)
                .arg(
                    Arg::new("check")
                        .long("check")
                        .help("Check if current version meets requirement")
                        .value_name("VERSION")
                )
        )
        .subcommand(
            ClapCommand::new("status")
                .about("Show system status")
                .args(&output_format_args)
                .arg(
                    Arg::new("watch")
                        .long("watch")
                        .help("Continuously monitor status")
                        .action(ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("interval")
                        .long("interval")
                        .help("Update interval in seconds when watching")
                        .value_name("SECONDS")
                        .default_value("5")
                )
        )
        .subcommand_required(false)
        .arg_required_else_help(true)
        .subcommand(
            ClapCommand::new("mcp")
                .about("Machine Context Protocol commands")
                .subcommand(
                    ClapCommand::new("server")
                        .about("Start the MCP server")
                        .arg(
                            Arg::new("host")
                                .short('h')
                                .long("host")
                                .help("Server host")
                                .default_value("localhost")
                        )
                        .arg(
                            Arg::new("port")
                                .short('p')
                                .long("port")
                                .help("Server port")
                                .default_value("7777")
                                .value_parser(clap::value_parser!(u16))
                        )
                )
        )
        .subcommand(
            config_command::ConfigCommand::new().parser()
        )
        .subcommand(
            plugin_command::PluginCommand::new().parser()
        )
}

/// Register all CLI commands
///
/// This function registers all commands provided by the CLI crate.
///
/// # Arguments
///
/// * `registry` - The command registry to register commands with
pub fn register_commands(registry: &mut CommandRegistry) {
    // Create command instances
    let help_command = HelpCommand::new();
    let version_command = VersionCommand::new();
    let status_command = StatusCommand::new();
    let config_command = ConfigCommand::new();
    let plugin_command = PluginCommand::new();
    
    // Convert to Arc<dyn Command>
    let help_arc = std::sync::Arc::new(help_command);
    let version_arc = std::sync::Arc::new(version_command);
    let status_arc = std::sync::Arc::new(status_command);
    let config_arc = std::sync::Arc::new(config_command);
    let plugin_arc = std::sync::Arc::new(plugin_command);
    
    // Register commands
    let _ = registry.register("help", help_arc);
    let _ = registry.register("version", version_arc);
    let _ = registry.register("status", status_arc);
    let _ = registry.register("config", config_arc);
    let _ = registry.register("plugin", plugin_arc);
    // Register additional commands here as they are implemented
} 