//! CLI structure for Squirrel UniBin architecture
//!
//! Modern, idiomatic Rust CLI using clap derive API.
//! Implements UniBin Architecture v1.0.0 ecosystem standard.

use clap::{Parser, Subcommand};

/// 🐿️ Squirrel - Universal AI Orchestration Primal
///
/// Squirrel is the Meta-AI Orchestration Primal for the ecoPrimals ecosystem,
/// providing intelligent AI routing, universal tool orchestration, and
/// capability-based discovery.
#[derive(Parser)]
#[command(name = "squirrel")]
#[command(author = "DataScienceBioLab")]
#[command(version)]
#[command(about = "🐿️ Squirrel - Universal AI Orchestration Primal", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands for Squirrel
#[derive(Subcommand)]
pub enum Commands {
    /// Start Squirrel in server mode
    ///
    /// Runs the AI orchestration server with HTTP and Unix socket APIs.
    /// Supports multiple AI providers (OpenAI, HuggingFace, Ollama) with
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
#[derive(Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text output (default)
    Text,
    /// JSON output for machine parsing
    Json,
}

/// Subsystem to check in doctor mode
#[derive(Clone, Copy, clap::ValueEnum)]
pub enum Subsystem {
    /// AI routing and providers
    Ai,
    /// Ecosystem connectivity (Songbird, BearDog, etc.)
    Ecosystem,
    /// Configuration and environment
    Config,
    /// Unix socket API
    Socket,
    /// HTTP API
    Http,
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
        let cli = Cli::try_parse_from(&["squirrel", "server"]).unwrap();
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
        let cli = Cli::try_parse_from(&["squirrel", "server", "--port", "8080"]).unwrap();
        if let Commands::Server { port, .. } = cli.command {
            assert_eq!(port, 8080);
        } else {
            panic!("Expected Server command");
        }
    }

    #[test]
    fn test_doctor_defaults() {
        let cli = Cli::try_parse_from(&["squirrel", "doctor"]).unwrap();
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
        let cli = Cli::try_parse_from(&["squirrel", "version"]).unwrap();
        assert!(matches!(cli.command, Commands::Version { .. }));
    }
}

