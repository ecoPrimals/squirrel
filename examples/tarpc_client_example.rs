// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! tarpc Client Usage Example
//!
//! This example demonstrates how to use the Squirrel tarpc client
//! for type-safe, high-performance RPC calls.
//!
//! ## Running This Example
//!
//! 1. Start a Squirrel server:
//!    ```bash
//!    cargo run --bin squirrel --features tarpc-rpc
//!    ```
//!
//! 2. Run this example:
//!    ```bash
//!    cargo run --example tarpc_client_example --features tarpc-rpc
//!    ```

#[cfg(feature = "tarpc-rpc")]
use squirrel::rpc::{SquirrelClient, SquirrelClientBuilder};
use std::time::Duration;

#[cfg(feature = "tarpc-rpc")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🦀 Squirrel tarpc Client Example\n");

    // Example 1: Simple connection
    println!("📡 Example 1: Simple Connection");
    println!("Connecting to Squirrel server...");
    
    let client = SquirrelClient::connect("squirrel").await?;
    println!("✅ Connected!\n");

    // Example 2: Ping/Pong
    println!("🏓 Example 2: Ping Test");
    let pong = client.ping().await?;
    println!("Response: {}\n", pong);

    // Example 3: Health Check
    println!("💚 Example 3: Health Check");
    let health = client.health().await?;
    println!("Status: {}", health.status);
    println!("Version: {}", health.version);
    println!("Uptime: {}s", health.uptime_seconds);
    println!("Requests processed: {}", health.requests_processed);
    if let Some(avg) = health.avg_response_time_ms {
        println!("Avg response time: {:.2}ms", avg);
    }
    println!();

    // Example 4: List Providers
    println!("📋 Example 4: List AI Providers");
    let providers = client.list_providers().await?;
    println!("Total providers: {}", providers.total);
    for provider in &providers.providers {
        println!("  - {} ({}): {}", provider.name, provider.id, 
                 if provider.online { "online" } else { "offline" });
    }
    println!();

    // Example 5: Query AI (if providers available)
    if providers.total > 0 {
        println!("🤖 Example 5: Query AI");
        let result = client
            .query_ai("Hello, how are you?", None, Some(50), Some(0.7))
            .await?;
        
        if result.success {
            println!("Response: {}", result.response);
            println!("Provider: {}", result.provider);
            println!("Model: {}", result.model);
            println!("Latency: {}ms", result.latency_ms);
        } else {
            println!("Query failed: {}", result.response);
        }
        println!();
    }

    // Example 6: Custom timeout
    println!("⏱️  Example 6: Custom Timeout");
    let custom_client = SquirrelClientBuilder::new("squirrel")
        .timeout(Duration::from_secs(60))
        .connect()
        .await?;
    
    let pong = custom_client.ping().await?;
    println!("Response with custom timeout: {}\n", pong);

    // Example 7: Announce Capabilities
    println!("📢 Example 7: Announce Capabilities");
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("version".to_string(), "1.0.0".to_string());
    
    let result = client
        .announce_capabilities(
            "example-service",
            vec!["text-generation".to_string(), "analysis".to_string()],
            metadata,
        )
        .await?;
    
    println!("Success: {}", result.success);
    println!("Message: {}", result.message);
    println!();

    // Example 8: Discover Peers
    println!("🔍 Example 8: Discover Peers");
    let peers = client.discover_peers().await?;
    if peers.is_empty() {
        println!("No peers discovered");
    } else {
        println!("Discovered {} peer(s):", peers.len());
        for peer in peers {
            println!("  - {}", peer);
        }
    }
    println!();

    println!("✅ All examples completed successfully!");
    Ok(())
}

#[cfg(not(feature = "tarpc-rpc"))]
fn main() {
    eprintln!("This example requires the 'tarpc-rpc' feature.");
    eprintln!("Run with: cargo run --example tarpc_client_example --features tarpc-rpc");
    std::process::exit(1);
}
