// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Infant Discovery Demo - TRUE PRIMAL Pattern
//!
//! Demonstrates deploying with ZERO knowledge and discovering capabilities at runtime.
//!
//! Run: cargo run --example infant_discovery_demo

use squirrel::capabilities::{discover_all_capabilities, discover_capability};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("╔════════════════════════════════════════════════════════╗");
    println!("║                                                        ║");
    println!("║   🐿️  SQUIRREL INFANT DISCOVERY DEMO 👶              ║");
    println!("║                                                        ║");
    println!("║   Deploy with ZERO knowledge, discover everything     ║");
    println!("║                                                        ║");
    println!("╚════════════════════════════════════════════════════════╝");
    println!();

    println!("👶 Infant Mode: Starting with ZERO knowledge...");
    println!("   (No hardcoded primal names, vendors, or ports!)");
    println!();

    // Discover all available capabilities
    println!("🔍 Scanning environment for capability providers...");
    println!();

    match discover_all_capabilities().await {
        Ok(capabilities) => {
            if capabilities.is_empty() {
                println!("❌ No capability providers found.");
                println!();
                println!("💡 To test this demo:");
                println!("   1. Start a primal that responds to 'discover_capabilities'");
                println!("   2. Or set environment variables:");
                println!("      export CRYPTO_SIGNING_PROVIDER_SOCKET=/tmp/provider.sock");
                println!();
            } else {
                println!("✅ Discovered {} capabilities:", capabilities.len());
                println!();

                for (capability, providers) in &capabilities {
                    println!("   📦 Capability: {}", capability);
                    for provider in providers {
                        println!("      └─ Provider ID: {}", provider.id);
                        println!("         Socket: {:?}", provider.socket);
                        println!("         Via: {}", provider.discovered_via);
                    }
                    println!();
                }
            }
        }
        Err(e) => {
            println!("⚠️  Discovery error: {}", e);
        }
    }

    // Try to discover specific capabilities
    println!("🎯 Attempting to discover specific capabilities...");
    println!();

    let needed_capabilities = vec![
        "crypto.signing",
        "http.request",
        "storage.object",
        "ai.text_generation",
    ];

    for capability in needed_capabilities {
        match discover_capability(capability).await {
            Ok(provider) => {
                println!("   ✅ Found: {}", capability);
                println!("      Provider: {} (via {})", provider.id, provider.discovered_via);
            }
            Err(_) => {
                println!("   ❌ Not found: {}", capability);
            }
        }
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("💡 TRUE PRIMAL Pattern Demonstrated:");
    println!("   • Zero hardcoded primal names (NO BearDog, Songbird, etc.)");
    println!("   • Zero vendor lock-in (NO Kubernetes, Consul, etc.)");
    println!("   • Runtime capability discovery");
    println!("   • Deploy like an infant - knows nothing!");
    println!();

    Ok(())
}

