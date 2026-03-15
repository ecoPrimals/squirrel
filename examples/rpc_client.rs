// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Example JSON-RPC client for testing Squirrel's Unix socket RPC server
//!
//! This demonstrates how biomeOS can interact with Squirrel via JSON-RPC 2.0

use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get node ID from environment or use default
    let node_id = std::env::var("SQUIRREL_NODE_ID").unwrap_or_else(|_| "squirrel".to_string());
    let socket_path = format!("/tmp/squirrel-{}.sock", node_id);

    println!("🐿️  Squirrel JSON-RPC Client Example");
    println!("   Connecting to: {}", socket_path);
    println!();

    // Connect to Unix socket
    let stream = UnixStream::connect(&socket_path).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Example 1: Health check
    println!("📋 Example 1: Health Check");
    let health_request = json!({
        "jsonrpc": "2.0",
        "method": "health_check",
        "params": {},
        "id": 1
    });

    send_request(&mut writer, &mut reader, health_request).await?;
    println!();

    // Example 2: List providers
    println!("📋 Example 2: List AI Providers");
    let list_providers_request = json!({
        "jsonrpc": "2.0",
        "method": "list_providers",
        "params": {
            "capability": null,
            "include_offline": true
        },
        "id": 2
    });

    send_request(&mut writer, &mut reader, list_providers_request).await?;
    println!();

    // Example 3: Query AI
    println!("📋 Example 3: Query AI");
    let query_ai_request = json!({
        "jsonrpc": "2.0",
        "method": "query_ai",
        "params": {
            "prompt": "Explain Rust's ownership system in one sentence",
            "provider": "auto",
            "model": null,
            "priority": 50,
            "max_tokens": 100,
            "temperature": 0.7,
            "stream": false
        },
        "id": 3
    });

    send_request(&mut writer, &mut reader, query_ai_request).await?;
    println!();

    // Example 4: Announce capabilities
    println!("📋 Example 4: Announce Capabilities");
    let announce_request = json!({
        "jsonrpc": "2.0",
        "method": "announce_capabilities",
        "params": {
            "capabilities": ["ai.inference", "ai.multi-provider", "mcp.protocol"],
            "sub_federations": null,
            "genetic_families": null
        },
        "id": 4
    });

    send_request(&mut writer, &mut reader, announce_request).await?;
    println!();

    println!("✅ All examples completed successfully!");

    Ok(())
}

async fn send_request(
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    reader: &mut BufReader<tokio::net::unix::OwnedReadHalf>,
    request: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    // Send request
    let request_str = serde_json::to_string(&request)?;
    println!("→ Request: {}", request_str);
    writer.write_all(request_str.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    // Read response
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;

    // Parse and display response
    let response: serde_json::Value = serde_json::from_str(&response_line)?;
    println!("← Response:");
    println!("{}", serde_json::to_string_pretty(&response)?);

    Ok(())
}

