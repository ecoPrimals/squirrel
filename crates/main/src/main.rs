//! Squirrel AI Coordinator Main Entry Point
//!
//! UniBin Architecture v1.0.0 compliant entry point.
//! Modern, idiomatic async Rust with clap-based CLI.

mod cli;
mod doctor;

use anyhow::Result;
use clap::Parser;
use serde::Serialize;
// ApiServer REMOVED - HTTP API deleted, use JSON-RPC instead
// use squirrel::api::ApiServer; // DELETED
use squirrel::ecosystem::{EcosystemConfig, EcosystemManager};
use squirrel::shutdown::ShutdownManager;
#[cfg(feature = "monitoring")]
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
    daemon: bool,
    socket: Option<String>,
    bind: String,
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
    #[cfg(feature = "monitoring")]
    let metrics_collector = Arc::new(MetricsCollector::new());
    #[cfg(not(feature = "monitoring"))]
    let metrics_collector = Arc::new(squirrel::monitoring::metrics::MetricsCollector::new());

    let ecosystem_config = EcosystemConfig::default();
    let ecosystem_manager = Arc::new(EcosystemManager::new(
        ecosystem_config,
        metrics_collector.clone(),
    ));
    let shutdown_manager = Arc::new(ShutdownManager::new());

    println!("✅ Ecosystem Manager initialized");
    println!("✅ Metrics Collector initialized");
    println!("✅ Shutdown Manager initialized");

    // Legacy HTTP API server REMOVED - Squirrel uses Unix sockets + JSON-RPC + tarpc!
    println!("✅ Modern architecture: Unix sockets + JSON-RPC + tarpc");
    println!("   (No HTTP server - TRUE PRIMAL!)");

    // Determine socket path using priority:
    // 1. CLI --socket argument
    // 2. Environment variables (SQUIRREL_SOCKET, BIOMEOS_SOCKET_PATH, SQUIRREL_FAMILY_ID)
    // 3. Default fallback
    use squirrel::rpc::unix_socket;

    let socket_path = if let Some(path) = socket {
        path
    } else {
        let node_id = unix_socket::get_node_id();
        unix_socket::get_socket_path(&node_id)
    };

    println!("🔌 Starting JSON-RPC server...");
    println!("   Socket: {}", socket_path);
    println!("   Bind: {} (unused in Unix socket mode)", bind);
    println!("   Port: {} (unused in Unix socket mode)", port);
    if daemon {
        println!("   Daemon mode: enabled (TODO: implement background detach)");
    }
    println!();
    println!("✅ Squirrel AI/MCP Primal Ready!");

    // Initialize AI router (optional, for actual AI calls)
    // TODO: Initialize AI router from config
    // For now, server will start without AI router (health checks work, AI calls return "not configured")

    // Create and start JSON-RPC server
    use squirrel::rpc::JsonRpcServer;
    let server = Arc::new(JsonRpcServer::new(socket_path.clone()));
    let server_clone = Arc::clone(&server);

    // Start server in background task
    let server_task = tokio::spawn(async move {
        if let Err(e) = server_clone.start().await {
            eprintln!("❌ Server error: {}", e);
        }
    });

    // Setup graceful shutdown on Ctrl+C
    println!("   Press Ctrl+C to stop");
    println!();

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("\n👋 Shutting down gracefully...");

            // Cleanup socket file
            unix_socket::cleanup_socket(&socket_path);

            // Request shutdown
            if let Err(e) = shutdown_manager.request_shutdown().await {
                eprintln!("⚠️ Shutdown error: {}", e);
            }

            println!("✅ Shutdown complete");
        }
        _ = server_task => {
            println!("Server task completed");
        }
    }

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
