//! Squirrel AI Coordinator Main Entry Point

use anyhow::Result;
use serde::Serialize;
use squirrel::api::ApiServer;
use squirrel::ecosystem::{EcosystemConfig, EcosystemManager};
use squirrel::shutdown::ShutdownManager;
use squirrel::MetricsCollector;
use std::collections::HashMap;
use std::sync::Arc;

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
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    // Handle --version flag
    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        println!("squirrel {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Handle --capability flag
    if args.iter().any(|arg| arg == "--capability") {
        let port = std::env::var("PORT")
            .or_else(|_| std::env::var("SQUIRREL_PORT"))
            .unwrap_or_else(|_| "9010".to_string());

        let mut endpoints = HashMap::new();
        endpoints.insert("health", format!("http://localhost:{}/health", port));
        endpoints.insert("api", format!("http://localhost:{}/api/v1", port));
        endpoints.insert("metrics", format!("http://localhost:{}/metrics", port));

        let manifest = CapabilityManifest {
            name: "squirrel",
            category: "configuration",
            version: env!("CARGO_PKG_VERSION"),
            api_type: "REST",
            capabilities: vec![
                "universal-ai-coordination",
                "config-management",
                "capability-discovery",
                "mcp-protocol",
                "ecosystem-integration",
                "zero-copy-optimization",
            ],
            endpoints,
            discovery: DiscoveryInfo {
                protocol: "HTTP/REST",
                default_port: 9010,
                health_check: format!("http://localhost:{}/health", port),
            },
        };

        println!("{}", serde_json::to_string_pretty(&manifest)?);
        return Ok(());
    }

    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter("squirrel=info,debug")
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();

    println!("🐿️  Squirrel AI/MCP Primal Starting...");
    println!("   Version: {}", env!("CARGO_PKG_VERSION"));
    println!("✅ Arc<str> Modernization Complete");
    println!("✅ Performance Optimized with Zero-Copy Patterns");

    // Get configuration from environment
    let port = std::env::var("PORT")
        .or_else(|_| std::env::var("SQUIRREL_PORT"))
        .unwrap_or_else(|_| "9010".to_string())
        .parse::<u16>()?;

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
    println!("✅ Squirrel AI/MCP Primal Ready!");

    // Start the server (this will block)
    api_server.start().await?;

    Ok(())
}
