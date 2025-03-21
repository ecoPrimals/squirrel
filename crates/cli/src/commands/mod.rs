//! CLI commands for the Squirrel platform
//!
//! This module contains the implementation of CLI commands that
//! provide functionality for the Squirrel command-line interface.

mod config_command;

pub use config_command::{ConfigCommand, ConfigArgs, ConfigSubcommand, register_config_commands};

use clap::{Command as ClapCommand, Arg};
use squirrel_commands::{CommandRegistry, Command};

/// Create the command-line interface
///
/// This function creates the command-line interface for the Squirrel CLI.
pub fn create_cli() -> ClapCommand {
    ClapCommand::new("squirrel")
        .about("Squirrel command-line interface")
        .version("0.1.0")
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
}

/// Register all CLI commands
///
/// This function registers all commands provided by the CLI crate.
///
/// # Arguments
///
/// * `registry` - The command registry to register commands with
pub fn register_commands(registry: &mut CommandRegistry) {
    register_config_commands(registry);
    // Register additional commands here as they are implemented
} 