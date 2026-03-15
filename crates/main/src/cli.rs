// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! CLI structure for Squirrel UniBin architecture
//!
//! UniBin exit codes: 0=success, 1=error, 2=config, 3=network, 130=interrupted

/// UniBin standard exit codes
pub mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const ERROR: i32 = 1;
    pub const CONFIG_ERROR: i32 = 2;
    pub const NETWORK_ERROR: i32 = 3;
    pub const INTERRUPTED: i32 = 130;
}

// Modern, idiomatic Rust CLI using clap derive API.
// Implements UniBin Architecture v1.0.0 ecosystem standard.

use clap::{Parser, Subcommand};
use std::fmt;

/// 🐿️ Squirrel - Universal AI Orchestration Primal
///
/// Squirrel is the Meta-AI Orchestration Primal for the ecoPrimals ecosystem,
/// providing intelligent AI routing, universal tool orchestration, and
/// capability-based discovery.
#[derive(Parser, Debug)]
#[command(name = "squirrel")]
#[command(author = "ecoPrimals Contributors")]
#[command(version)]
#[command(about = "🐿️ Squirrel - Universal AI Orchestration Primal", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands for Squirrel
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start Squirrel in server mode
    ///
    /// Runs the AI orchestration server with HTTP and Unix socket APIs.
    /// Supports multiple AI providers (cloud APIs, local servers, model hubs) with
    /// intelligent routing based on cost, quality, and latency.
    Server {
        /// Server port for HTTP API
        #[arg(short, long, default_value = "9010")]
        port: u16,

        /// Run as background daemon
        ///
        /// When enabled, Squirrel will detach from the terminal and run
        /// as a background process.
        #[arg(short, long)]
        daemon: bool,

        /// Unix socket path for JSON-RPC API
        ///
        /// Overrides the default socket path. If not specified, uses
        /// XDG runtime directory or /tmp fallback.
        #[arg(short, long)]
        socket: Option<String>,

        /// Bind address for HTTP server
        ///
        /// Default is 0.0.0.0 (all interfaces). Use 127.0.0.1 for
        /// localhost only.
        #[arg(short, long, default_value = "0.0.0.0")]
        bind: String,

        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },

    /// Send JSON-RPC request to a running Squirrel server
    ///
    /// Connects to the Unix socket and sends a JSON-RPC request.
    /// Use for scripting and automation.
    Client {
        /// Path to the Unix socket (optional, auto-detected if not specified)
        #[arg(short, long)]
        socket: Option<String>,

        /// JSON-RPC method to call
        #[arg(short, long)]
        method: String,

        /// JSON parameters (optional, defaults to {})
        #[arg(short, long, default_value = "{}")]
        params: String,

        /// Timeout in milliseconds (optional, default 5000)
        #[arg(short, long, default_value = "5000")]
        timeout: u64,
    },

    /// Run health diagnostics
    ///
    /// Performs comprehensive health checks on all Squirrel subsystems,
    /// including AI providers, ecosystem connectivity, and configuration.
    Doctor {
        /// Run comprehensive checks (includes connectivity tests)
        #[arg(short, long)]
        comprehensive: bool,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,

        /// Check specific subsystem only
        #[arg(short = 's', long)]
        subsystem: Option<Subsystem>,
    },

    /// Show version information
    ///
    /// Displays detailed version information including build metadata,
    /// platform, and Rust version.
    Version {
        /// Show detailed build information
        #[arg(short, long)]
        verbose: bool,
    },
}

/// Output format for doctor mode
#[derive(Clone, Copy, clap::ValueEnum, Debug)]
pub enum OutputFormat {
    /// Human-readable text output (default)
    Text,
    /// JSON output for machine parsing
    Json,
}

/// Subsystem to check in doctor mode
#[derive(Clone, Copy, clap::ValueEnum, Debug)]
pub enum Subsystem {
    /// AI routing and providers
    Ai,
    /// Ecosystem connectivity (capability-based service discovery)
    Ecosystem,
    /// Configuration and environment
    Config,
    /// Unix socket API
    Socket,
    /// RPC server (tarpc)
    Rpc,
}

impl fmt::Display for Subsystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Subsystem::Ai => write!(f, "ai"),
            Subsystem::Ecosystem => write!(f, "ecosystem"),
            Subsystem::Config => write!(f, "config"),
            Subsystem::Socket => write!(f, "socket"),
            Subsystem::Rpc => write!(f, "rpc"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_parsing() {
        // Verify CLI structure parses correctly
        Cli::command().debug_assert();
    }

    #[test]
    fn test_server_defaults() {
        let cli = Cli::try_parse_from(["squirrel", "server"]).unwrap();
        if let Commands::Server {
            port,
            daemon,
            socket,
            bind,
            ..
        } = cli.command
        {
            assert_eq!(port, 9010);
            assert!(!daemon);
            assert!(socket.is_none());
            assert_eq!(bind, "0.0.0.0");
        } else {
            panic!("Expected Server command");
        }
    }

    #[test]
    fn test_server_custom_port() {
        let cli = Cli::try_parse_from(["squirrel", "server", "--port", "8080"]).unwrap();
        if let Commands::Server { port, .. } = cli.command {
            assert_eq!(port, 8080);
        } else {
            panic!("Expected Server command");
        }
    }

    #[test]
    fn test_doctor_defaults() {
        let cli = Cli::try_parse_from(["squirrel", "doctor"]).unwrap();
        if let Commands::Doctor {
            comprehensive,
            format,
            subsystem,
        } = cli.command
        {
            assert!(!comprehensive);
            assert!(matches!(format, OutputFormat::Text));
            assert!(subsystem.is_none());
        } else {
            panic!("Expected Doctor command");
        }
    }

    #[test]
    fn test_version_command() {
        let cli = Cli::try_parse_from(["squirrel", "version"]).unwrap();
        assert!(matches!(cli.command, Commands::Version { .. }));
    }

    #[test]
    fn test_client_command() {
        let cli = Cli::try_parse_from([
            "squirrel",
            "client",
            "--method",
            "system.status",
            "--params",
            "{}",
        ])
        .unwrap();
        if let Commands::Client {
            socket,
            method,
            params,
            timeout,
        } = cli.command
        {
            assert!(socket.is_none());
            assert_eq!(method, "system.status");
            assert_eq!(params, "{}");
            assert_eq!(timeout, 5000);
        } else {
            panic!("Expected Client command");
        }
    }

    #[test]
    fn test_client_command_with_socket() {
        let cli = Cli::try_parse_from([
            "squirrel",
            "client",
            "--socket",
            "/tmp/squirrel.sock",
            "--method",
            "system.health",
            "--timeout",
            "10000",
        ])
        .unwrap();
        if let Commands::Client {
            socket,
            method,
            params,
            timeout,
        } = cli.command
        {
            assert_eq!(socket, Some("/tmp/squirrel.sock".to_string()));
            assert_eq!(method, "system.health");
            assert_eq!(params, "{}"); // default
            assert_eq!(timeout, 10000);
        } else {
            panic!("Expected Client command");
        }
    }

    // ========================================================================
    // ADDITIONAL E2E TESTS
    // ========================================================================

    #[test]
    fn test_server_all_options_together() {
        let cli = Cli::try_parse_from([
            "squirrel",
            "server",
            "--port",
            "3000",
            "--daemon",
            "--socket",
            "/custom/socket.sock",
            "--bind",
            "192.168.1.1",
            "--verbose",
        ])
        .unwrap();

        if let Commands::Server {
            port,
            daemon,
            socket,
            bind,
            verbose,
        } = cli.command
        {
            assert_eq!(port, 3000);
            assert!(daemon);
            assert_eq!(socket, Some("/custom/socket.sock".to_string()));
            assert_eq!(bind, "192.168.1.1");
            assert!(verbose);
        } else {
            panic!("Expected Server command");
        }
    }

    #[test]
    fn test_doctor_comprehensive_with_subsystem() {
        let cli = Cli::try_parse_from([
            "squirrel",
            "doctor",
            "--comprehensive",
            "--subsystem",
            "ai",
            "--format",
            "json",
        ])
        .unwrap();

        if let Commands::Doctor {
            comprehensive,
            subsystem,
            format,
        } = cli.command
        {
            assert!(comprehensive);
            assert!(matches!(subsystem, Some(Subsystem::Ai)));
            assert!(matches!(format, OutputFormat::Json));
        } else {
            panic!("Expected Doctor command");
        }
    }

    #[test]
    fn test_version_verbose() {
        let cli = Cli::try_parse_from(["squirrel", "version", "--verbose"]).unwrap();
        if let Commands::Version { verbose } = cli.command {
            assert!(verbose);
        } else {
            panic!("Expected Version command");
        }
    }

    // ========================================================================
    // CHAOS TESTS - Invalid Inputs and Edge Cases
    // ========================================================================

    #[test]
    fn test_invalid_command_name() {
        let result = Cli::try_parse_from(["squirrel", "invalid"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_subcommand_provided() {
        let result = Cli::try_parse_from(["squirrel"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_port_too_large() {
        let result = Cli::try_parse_from(["squirrel", "server", "--port", "70000"]);
        assert!(result.is_err()); // Port must be <= 65535
    }

    #[test]
    fn test_invalid_port_negative() {
        let result = Cli::try_parse_from(["squirrel", "server", "--port", "-1"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_port_non_numeric() {
        let result = Cli::try_parse_from(["squirrel", "server", "--port", "abc"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_doctor_format() {
        let result = Cli::try_parse_from(["squirrel", "doctor", "--format", "xml"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_doctor_subsystem() {
        let result = Cli::try_parse_from(["squirrel", "doctor", "--subsystem", "invalid"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_flag_on_server() {
        let result = Cli::try_parse_from(["squirrel", "server", "--unknown"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_value_for_port() {
        let result = Cli::try_parse_from(["squirrel", "server", "--port"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_socket_path() {
        let result = Cli::try_parse_from(["squirrel", "server", "--socket", ""]);
        assert!(result.is_ok()); // Empty string valid at parse time, validated at runtime
    }

    #[test]
    fn test_very_long_socket_path() {
        let long_path = format!("/tmp/{}.sock", "a".repeat(500));
        let result = Cli::try_parse_from(["squirrel", "server", "--socket", &long_path]);
        assert!(result.is_ok()); // Path length validated by OS at runtime
    }

    #[test]
    fn test_unicode_in_socket_path() {
        let result =
            Cli::try_parse_from(["squirrel", "server", "--socket", "/tmp/🦀squirrel.sock"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_special_chars_in_socket_path() {
        let result =
            Cli::try_parse_from(["squirrel", "server", "--socket", "/tmp/socket!@#$.sock"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_port_boundary_min() {
        let result = Cli::try_parse_from(["squirrel", "server", "--port", "1"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_port_boundary_max() {
        let result = Cli::try_parse_from(["squirrel", "server", "--port", "65535"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_port_zero() {
        let result = Cli::try_parse_from(["squirrel", "server", "--port", "0"]);
        assert!(result.is_ok()); // Port 0 means "let OS assign"
    }

    // ========================================================================
    // FAULT TESTS - Error Handling
    // ========================================================================

    #[test]
    fn test_error_message_contains_context() {
        let result = Cli::try_parse_from(["squirrel", "nonexistent"]);
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(!err_msg.is_empty(), "Error message should not be empty");
    }

    #[test]
    fn test_help_available_for_server() {
        let result = Cli::try_parse_from(["squirrel", "server", "--help"]);
        assert!(result.is_err()); // Help causes early exit
    }

    #[test]
    fn test_help_available_for_doctor() {
        let result = Cli::try_parse_from(["squirrel", "doctor", "--help"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_help_available_for_version() {
        let result = Cli::try_parse_from(["squirrel", "version", "--help"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_case_sensitive_commands() {
        let result = Cli::try_parse_from(["squirrel", "SERVER"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_invalid_arguments() {
        let result = Cli::try_parse_from([
            "squirrel",
            "server",
            "--port",
            "999999",
            "--unknown",
            "value",
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_short_flags_work() {
        let cli = Cli::try_parse_from(["squirrel", "server", "-p", "8080", "-d", "-v"]).unwrap();
        if let Commands::Server {
            port,
            daemon,
            verbose,
            ..
        } = cli.command
        {
            assert_eq!(port, 8080);
            assert!(daemon);
            assert!(verbose);
        } else {
            panic!("Expected Server command");
        }
    }

    #[test]
    fn test_mixed_short_and_long_flags() {
        let cli = Cli::try_parse_from([
            "squirrel",
            "server",
            "-p",
            "3000",
            "--daemon",
            "-v",
            "--bind",
            "127.0.0.1",
        ])
        .unwrap();

        if let Commands::Server {
            port,
            daemon,
            verbose,
            bind,
            ..
        } = cli.command
        {
            assert_eq!(port, 3000);
            assert!(daemon);
            assert!(verbose);
            assert_eq!(bind, "127.0.0.1");
        } else {
            panic!("Expected Server command");
        }
    }

    #[test]
    fn test_all_subsystems_valid() {
        let subsystems = vec!["ai", "ecosystem", "config", "socket", "rpc"];
        for sub in subsystems {
            let result = Cli::try_parse_from(["squirrel", "doctor", "--subsystem", sub]);
            assert!(result.is_ok(), "Subsystem '{}' should be valid", sub);
        }
    }

    #[test]
    fn test_doctor_json_format() {
        let cli = Cli::try_parse_from(["squirrel", "doctor", "--format", "json"]).unwrap();
        if let Commands::Doctor { format, .. } = cli.command {
            assert!(matches!(format, OutputFormat::Json));
        } else {
            panic!("Expected Doctor command");
        }
    }

    #[test]
    fn test_doctor_text_format() {
        let cli = Cli::try_parse_from(["squirrel", "doctor", "--format", "text"]).unwrap();
        if let Commands::Doctor { format, .. } = cli.command {
            assert!(matches!(format, OutputFormat::Text));
        } else {
            panic!("Expected Doctor command");
        }
    }

    #[test]
    fn test_cli_structure_is_valid() {
        // Verify CLI structure passes clap's internal validation
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }

    // ========================================================================
    // INTEGRATION TESTS - Real-world Scenarios
    // ========================================================================

    #[test]
    fn test_production_server_startup() {
        let cli =
            Cli::try_parse_from(["squirrel", "server", "--port", "9010", "--bind", "0.0.0.0"])
                .unwrap();

        if let Commands::Server { port, bind, .. } = cli.command {
            assert_eq!(port, 9010);
            assert_eq!(bind, "0.0.0.0");
        } else {
            panic!("Expected Server command");
        }
    }

    #[test]
    fn test_development_server_startup() {
        let cli = Cli::try_parse_from([
            "squirrel",
            "server",
            "--port",
            "8080",
            "--bind",
            "127.0.0.1",
            "--verbose",
        ])
        .unwrap();

        if let Commands::Server {
            port,
            bind,
            verbose,
            ..
        } = cli.command
        {
            assert_eq!(port, 8080);
            assert_eq!(bind, "127.0.0.1");
            assert!(verbose);
        } else {
            panic!("Expected Server command");
        }
    }

    #[test]
    fn test_automated_health_check() {
        let cli = Cli::try_parse_from([
            "squirrel",
            "doctor",
            "--format",
            "json",
            "--subsystem",
            "ai",
        ])
        .unwrap();

        if let Commands::Doctor {
            format, subsystem, ..
        } = cli.command
        {
            assert!(matches!(format, OutputFormat::Json));
            assert!(matches!(subsystem, Some(Subsystem::Ai)));
        } else {
            panic!("Expected Doctor command");
        }
    }
}
