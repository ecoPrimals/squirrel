// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Squirrel AI Coordinator Main Entry Point
//!
//! UniBin Architecture v1.0.0 compliant entry point.
//! Modern, idiomatic async Rust with clap-based CLI.

mod cli;
mod doctor;

use anyhow::Result;
use clap::Parser;
use squirrel::ecosystem::{EcosystemConfig, EcosystemManager};
use squirrel::shutdown::ShutdownManager;
#[cfg(feature = "monitoring")]
use squirrel::MetricsCollector;
use std::sync::Arc;

use cli::{Cli, Commands};

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
    // Load configuration (file + env vars)
    let mut config = squirrel::config::ConfigLoader::load(None)?;

    // CLI arguments override config file
    if let Some(ref s) = socket {
        config.server.socket = Some(s.clone());
    }
    if daemon {
        config.server.daemon = true;
    }
    config.server.port = port;
    config.server.bind = bind.clone();

    // Initialize tracing subscriber with config
    let log_level = if verbose {
        "debug"
    } else {
        &config.logging.level
    };

    // FUTURE: [Feature] Add JSON logging support with tracing-subscriber json feature
    // Tracking: Planned for v0.2.0 - enhanced logging features
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
    let _ecosystem_manager = Arc::new(EcosystemManager::new(
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
    // 1. CLI --socket argument (HIGHEST PRIORITY)
    // 2. config.server.socket (from config file or env)
    // 3. Environment variables (SQUIRREL_SOCKET, BIOMEOS_SOCKET_PATH)
    // 4. Default fallback (XDG or /tmp)
    use squirrel::rpc::unix_socket;

    let socket_path = if let Some(path) = socket.clone() {
        // CLI argument has highest priority
        println!("📌 Socket path from CLI argument: {}", path);
        path
    } else if let Some(ref path) = config.server.socket {
        // Config file/env override
        println!("📌 Socket path from config: {}", path);
        path.clone()
    } else {
        // Fallback to auto-detection
        let node_id = unix_socket::get_node_id();
        let path = unix_socket::get_socket_path(&node_id);
        println!("📌 Socket path from auto-detection: {}", path);
        path
    };

    println!("🔌 Starting JSON-RPC server...");
    println!("   Socket: {}", socket_path);
    println!("   Bind: {} (unused in Unix socket mode)", bind);
    println!("   Port: {} (unused in Unix socket mode)", port);
    if daemon {
        println!("   Daemon mode: enabled (FUTURE: implement background detach)");
    }
    println!();
    println!("✅ Squirrel AI/MCP Primal Ready!");

    // Initialize AI router with capability-based discovery
    println!("🤖 Initializing AI router...");
    let ai_router = match squirrel::api::AiRouter::new_with_discovery(None).await {
        Ok(router) => {
            let provider_count = router.provider_count().await;
            if provider_count > 0 {
                println!("   ✅ {} AI provider(s) discovered", provider_count);
            } else {
                println!("   ⚠️  No AI providers found (query_ai will return 'not configured')");
                println!("   💡 Set AI_PROVIDER_SOCKETS env var for capability discovery");
            }
            Some(Arc::new(router))
        }
        Err(e) => {
            println!("   ⚠️  AI router initialization failed: {}", e);
            println!("   💡 Server will start without AI capabilities");
            None
        }
    };

    // Create JSON-RPC server with or without AI router
    use squirrel::rpc::JsonRpcServer;
    let server = if let Some(router) = ai_router {
        Arc::new(JsonRpcServer::with_ai_router(socket_path.clone(), router))
    } else {
        Arc::new(JsonRpcServer::new(socket_path.clone()))
    };
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
