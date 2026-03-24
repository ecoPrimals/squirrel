// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![allow(
    clippy::option_if_let_else,
    clippy::cast_possible_truncation,
    reason = "Main entry point; legacy patterns under progressive refactor"
)]

//! Squirrel AI Coordinator Main Entry Point
//!
//! `UniBin` Architecture v1.0.0 compliant entry point.
//! Modern, idiomatic async Rust with clap-based CLI.

mod cli;
mod doctor;

use anyhow::{Context, Result};
use clap::Parser;
#[cfg(feature = "monitoring")]
use squirrel::MetricsCollector;
use squirrel::ecosystem::{EcosystemConfig, EcosystemManager};
use squirrel::shutdown::ShutdownManager;
use std::process;
use std::sync::Arc;
use tracing::{error, info, warn};

use cli::{Cli, Commands, exit_codes};

#[tokio::main]
async fn main() {
    let code = run().await;
    process::exit(code);
}

/// Returns exit code (`UniBin`: 0=success, 1=error, 2=config, 3=network, 130=interrupted)
async fn run() -> i32 {
    // Parse CLI arguments using clap
    let cli = Cli::parse();

    // Initialize tracing for all commands (verbose = debug for server)
    let log_level = match &cli.command {
        Commands::Server { verbose: true, .. } => "debug",
        _ => "info",
    };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(format!("squirrel={log_level},warn"))
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .try_init();

    // Route to appropriate handler based on subcommand
    match cli.command {
        Commands::Server {
            port,
            daemon,
            socket,
            bind,
            verbose,
        } => {
            if let Err(e) = run_server(port, daemon, socket, bind, verbose).await {
                error!("Error: {e:?}");
                return exit_codes::ERROR;
            }
        }
        Commands::Client {
            socket,
            method,
            params,
            timeout,
        } => return run_client(socket, method, params, timeout).await,
        Commands::Doctor {
            comprehensive,
            format,
            subsystem,
        } => {
            if let Err(e) = doctor::run_doctor(comprehensive, format, subsystem) {
                error!("Error: {e:?}");
                return exit_codes::ERROR;
            }
        }
        Commands::Version { verbose } => {
            print_version(verbose);
        }
    }

    exit_codes::SUCCESS
}

/// Run client mode: connect to socket, send JSON-RPC, print response
async fn run_client(
    socket: Option<String>,
    method: String,
    params: String,
    timeout_ms: u64,
) -> i32 {
    use squirrel::rpc::unix_socket;
    use universal_patterns::IpcClient;

    let socket_path = socket.unwrap_or_else(|| {
        let node_id = unix_socket::get_node_id();
        unix_socket::get_socket_path(&node_id)
    });

    let params_value: serde_json::Value = match serde_json::from_str(&params) {
        Ok(v) => v,
        Err(e) => {
            error!("Invalid --params JSON: {e}");
            return exit_codes::CONFIG_ERROR;
        }
    };

    let timeout = std::time::Duration::from_millis(timeout_ms);
    let client = IpcClient::new(&socket_path)
        .with_request_timeout(timeout)
        .with_connection_timeout(timeout);

    let call_fut = client.call(&method, &params_value);
    tokio::select! {
        result = call_fut => match result {
            Ok(result) => {
                println!("{}", serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string()));
                exit_codes::SUCCESS
            }
            Err(e) => {
                error!("Error: {e:?}");
                if let Some(ipc_err) = e.downcast_ref::<universal_patterns::IpcClientError>() {
                    match ipc_err {
                        universal_patterns::IpcClientError::Connection { .. }
                        | universal_patterns::IpcClientError::NotFound(_)
                        | universal_patterns::IpcClientError::Timeout { .. } => exit_codes::NETWORK_ERROR,
                        _ => exit_codes::ERROR,
                    }
                } else {
                    exit_codes::ERROR
                }
            }
        },
        _ = tokio::signal::ctrl_c() => {
            error!("Interrupted");
            exit_codes::INTERRUPTED
        }
    }
}

/// Run server mode
#[allow(
    clippy::too_many_lines,
    reason = "Server orchestration; refactor planned"
)]
async fn run_server(
    port: u16,
    daemon: bool,
    socket: Option<String>,
    bind: String,
    _verbose: bool,
) -> Result<()> {
    use squirrel::rpc::JsonRpcServer;
    use squirrel::rpc::unix_socket;

    // Load configuration (file + env vars)
    let mut config =
        squirrel::config::ConfigLoader::load(None).context("Failed to load configuration")?;

    // CLI arguments override config file
    if let Some(ref s) = socket {
        config.server.socket = Some(s.clone());
    }
    if daemon {
        config.server.daemon = true;
    }
    config.server.port = port;
    config.server.bind = bind.clone();

    // Tracing already initialized in run() with verbose-based level

    info!("Squirrel AI/MCP Primal Starting");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));
    info!("Mode: Server");
    info!("UniBin Architecture v1.0.0");
    info!("Zero-HTTP Production Mode (v1.1.0)");

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

    info!("Ecosystem Manager initialized");
    info!("Metrics Collector initialized");
    info!("Shutdown Manager initialized");

    // Legacy HTTP API server REMOVED - Squirrel uses Unix sockets + JSON-RPC + tarpc!
    info!("Modern architecture: Unix sockets + JSON-RPC + tarpc");
    info!("No HTTP server - TRUE PRIMAL!");

    // Determine socket path using priority:
    // 1. CLI --socket argument (HIGHEST PRIORITY)
    // 2. config.server.socket (from config file or env)
    // 3. Environment variables (SQUIRREL_SOCKET, BIOMEOS_SOCKET_PATH)
    // 4. Default fallback (XDG or /tmp)
    let socket_path = if let Some(path) = socket.clone() {
        info!("Socket path from CLI argument: {path}");
        path
    } else if let Some(ref path) = config.server.socket {
        info!("Socket path from config: {path}");
        path.clone()
    } else {
        let node_id = unix_socket::get_node_id();
        let path = unix_socket::get_socket_path(&node_id);
        info!("Socket path from auto-detection: {path}");
        path
    };

    info!("Starting JSON-RPC server...");
    info!("Socket: {socket_path}");
    info!("Bind: {bind} (unused in Unix socket mode)");
    info!("Port: {port} (unused in Unix socket mode)");
    if daemon {
        info!("Daemon mode: enabled (FUTURE: implement background detach)");
    }
    info!("Squirrel AI/MCP Primal Ready!");

    // Write primal manifest for biomeOS manifest-based discovery
    let manifest = universal_patterns::manifest_discovery::PrimalManifest {
        primal: squirrel::niche::PRIMAL_ID.into(),
        version: env!("CARGO_PKG_VERSION").into(),
        socket: std::path::PathBuf::from(&socket_path),
        capabilities: squirrel::niche::CAPABILITIES
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
        pid: Some(std::process::id()),
        started_at: Some(chrono::Utc::now().to_rfc3339()),
        family_id: std::env::var("FAMILY_ID").ok(),
    };
    if let Err(e) = universal_patterns::manifest_discovery::write_manifest(&manifest) {
        warn!("Failed to write primal manifest: {e}");
    } else {
        info!("Primal manifest written for bootstrap discovery");
    }

    // Initialize AI router with capability-based discovery
    info!("Initializing AI router...");
    let ai_router = match squirrel::api::AiRouter::new_with_discovery(None).await {
        Ok(router) => {
            let provider_count = router.provider_count().await;
            if provider_count > 0 {
                info!("{provider_count} AI provider(s) discovered");
            } else {
                warn!("No AI providers found (query_ai will return 'not configured')");
                info!("Set AI_PROVIDER_SOCKETS env var for capability discovery");
            }
            Some(Arc::new(router))
        }
        Err(e) => {
            warn!("AI router initialization failed: {e}");
            info!("Server will start without AI capabilities");
            None
        }
    };

    let server = ai_router.map_or_else(
        || Arc::new(JsonRpcServer::new(socket_path.clone())),
        |router| Arc::new(JsonRpcServer::with_ai_router(socket_path.clone(), router)),
    );
    let server_clone = Arc::clone(&server);

    // Start server in background task
    let server_task = tokio::spawn(async move {
        if let Err(e) = server_clone.start().await {
            error!("Server error: {e}");
        }
    });

    // Install signal handlers (SIGTERM + SIGINT) for socket cleanup
    let (shutdown_tx, signal_task) =
        squirrel::capabilities::lifecycle::install_signal_handlers(socket_path.clone());
    let shutdown_rx = shutdown_tx.subscribe();

    // biomeOS lifecycle registration (healthSpring pattern)
    if let Some(biomeos_socket) = squirrel::capabilities::lifecycle::find_biomeos_socket() {
        let caps = server.capability_registry.method_names();
        let cap_refs: Vec<&str> = caps.as_slice().to_vec();
        if squirrel::capabilities::lifecycle::register_with_biomeos(
            &biomeos_socket,
            &socket_path,
            &cap_refs,
        )
        .await
        {
            info!("Registered with biomeOS");

            // Start heartbeat (30s interval)
            let _heartbeat = squirrel::capabilities::lifecycle::spawn_heartbeat(
                biomeos_socket,
                socket_path.clone(),
                std::time::Duration::from_secs(30),
                shutdown_rx,
            );
            info!("Heartbeat started (30s interval)");
        }
    } else {
        info!("No biomeOS socket found — standalone mode");
    }

    // Songbird service-mesh registration (wetSpring pattern)
    let shutdown_rx_songbird = shutdown_tx.subscribe();
    if let Some(songbird_socket) = squirrel::capabilities::songbird::discover_socket() {
        if squirrel::capabilities::songbird::register(&songbird_socket, &socket_path).await {
            info!("Registered with Songbird");

            let _songbird_heartbeat = squirrel::capabilities::songbird::start_heartbeat_loop(
                songbird_socket,
                socket_path.clone(),
                std::time::Duration::from_secs(30),
                shutdown_rx_songbird,
            );
            info!("Songbird heartbeat started (30s interval)");
        }
    } else {
        info!("No Songbird socket found — peer discovery unavailable");
    }

    info!("Press Ctrl+C to stop");

    let family_id = std::env::var("FAMILY_ID").ok();
    tokio::select! {
        _ = signal_task => {
            info!("Shutting down gracefully...");

            if let Err(e) = universal_patterns::manifest_discovery::remove_manifest(
                squirrel::niche::PRIMAL_ID,
                family_id.as_deref(),
            ) {
                warn!("Failed to remove primal manifest: {e}");
            }

            if let Err(e) = shutdown_manager.request_shutdown().await {
                warn!("Shutdown error: {e}");
            }

            info!("Shutdown complete");
        }
        _ = server_task => {
            info!("Server task completed");
        }
    }

    Ok(())
}

/// Print version information to stdout (CLI output, not logging).
fn print_version(verbose: bool) {
    println!("squirrel {}", env!("CARGO_PKG_VERSION"));
    if verbose {
        println!();
        println!("Architecture:   UniBin v1.0.0 / ecoBin v3.0");
        println!("Protocol:       JSON-RPC 2.0 + tarpc (dual)");
        println!("Transport:      Unix socket / Named pipe / TCP");
        println!("Discovery:      Capability-based runtime");
        println!("License:        AGPL-3.0-only (scyBorg)");
    }
}
