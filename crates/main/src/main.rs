//! Squirrel AI Coordinator Main Entry Point
//!
//! UniBin Architecture v1.0.0 compliant entry point.
//! Modern, idiomatic async Rust with clap-based CLI.

mod cli;
mod doctor;

use anyhow::Result;
use clap::Parser;
use serde::Serialize;
use squirrel::api::ApiServer;
use squirrel::ecosystem::{EcosystemConfig, EcosystemManager};
use squirrel::shutdown::ShutdownManager;
use squirrel::MetricsCollector;
use std::collections::HashMap;
use std::sync::Arc;

use cli::{Cli, Commands};

/// Capability manifest for BiomeOS integration
#[derive(Serialize)]
struct CapabilityManifest {
    name: &'static str,
    category: &'static str,
    version: &'static str,
    api_type: &'static str,
    capabilities: Vec<&'static str>,
    endpoints: HashMap<&'static str, String>,
    discovery: DiscoveryInfo,
}

#[derive(Serialize)]
struct DiscoveryInfo {
    protocol: &'static str,
    default_port: u16,
    health_check: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments using clap
    let cli = Cli::parse();

    // Route to appropriate handler based on subcommand
    match cli.command {
        Commands::Server {
            port,
            daemon,
            socket,
            bind,
            verbose,
        } => {
            run_server(port, daemon, socket, bind, verbose).await?;
        }
        Commands::Doctor {
            comprehensive,
            format,
            subsystem,
        } => {
            doctor::run_doctor(comprehensive, format, subsystem).await?;
        }
        Commands::Version { verbose } => {
            print_version(verbose);
        }
    }

    Ok(())
}

/// Run server mode
async fn run_server(
    port: u16,
    _daemon: bool, // Reserved for future daemon mode
    _socket: Option<String>,
    _bind: String,
    verbose: bool,
) -> Result<()> {
    // Initialize tracing subscriber
    let log_level = if verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("squirrel={},debug", log_level))
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();

    println!("🐿️  Squirrel AI/MCP Primal Starting...");
    println!("   Version: {}", env!("CARGO_PKG_VERSION"));
    println!("   Mode: Server");
    println!("✅ UniBin Architecture v1.0.0");
    println!("✅ Zero-HTTP Production Mode (v1.1.0)");
    println!("✅ Modern Async Concurrent Rust");
    println!();

    // Initialize ecosystem components
    let metrics_collector = Arc::new(MetricsCollector::new());
    let ecosystem_config = EcosystemConfig::default();
    let ecosystem_manager = Arc::new(EcosystemManager::new(
        ecosystem_config,
        metrics_collector.clone(),
    ));
    let shutdown_manager = Arc::new(ShutdownManager::new());

    println!("✅ Ecosystem Manager initialized");
    println!("✅ Metrics Collector initialized");
    println!("✅ Shutdown Manager initialized");

    // Create and start API server
    let api_server = ApiServer::new(
        port,
        ecosystem_manager.clone(),
        metrics_collector.clone(),
        shutdown_manager.clone(),
    );

    println!("🚀 Starting API server on port {}", port);
    println!("   Health: http://localhost:{}/health", port);
    println!("   API: http://localhost:{}/api/v1/*", port);
    println!();

    // Start JSON-RPC server on Unix socket (for biomeOS integration)
    let node_id = std::env::var("SQUIRREL_NODE_ID")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "squirrel".to_string());

    println!("🔌 Starting JSON-RPC server...");
    println!("   Socket: /tmp/squirrel-{}.sock", node_id);
    println!();
    println!("✅ Squirrel AI/MCP Primal Ready!");

    // Start the server (this will block)
    api_server.start().await?;

    Ok(())
}

/// Print version information
fn print_version(verbose: bool) {
    if verbose {
        println!("🐿️  Squirrel - Universal AI Orchestration Primal");
        println!();
        println!("Version:        {}", env!("CARGO_PKG_VERSION"));
        println!();
        println!("Features:");
        println!("  ✅ UniBin Architecture v1.0.0");
        println!("  ✅ Zero-HTTP Production Mode (v1.1.0)");
        println!("  ✅ Capability-Based Discovery");
        println!("  ✅ Multi-Provider AI Routing");
        println!("  ✅ Universal Tool Orchestration");
        println!("  ✅ PrimalPulse AI Tools");
    } else {
        println!("squirrel {}", env!("CARGO_PKG_VERSION"));
    }
}
