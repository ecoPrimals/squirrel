// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Simple biome.yaml manifest test
//!
//! This test validates the basic manifest parsing functionality.

use squirrel::biomeos_integration::manifest::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌱 Simple biome.yaml Manifest Test");
    println!("==================================");

    // 1. Generate a template manifest
    println!("\n📝 Generating template manifest...");
    let template = BiomeManifestParser::generate_template();
    println!("✅ Template generated: {}", template.metadata.name);
    println!("   Agents: {}", template.agents.len());
    println!(
        "   Agent names: {:?}",
        template.agents.iter().map(|a| &a.name).collect::<Vec<_>>()
    );

    // 2. Convert to YAML
    println!("\n📄 Converting to YAML...");
    let yaml_content = serde_yaml::to_string(&template)?;
    println!("✅ YAML content generated ({} bytes)", yaml_content.len());

    // 3. Parse the YAML back
    println!("\n🔍 Parsing YAML content...");
    let parser = BiomeManifestParser::new();
    let parsed = parser.parse_content(&yaml_content).await?;
    println!("✅ Successfully parsed manifest: {}", parsed.metadata.name);
    println!("   Parsed agents: {}", parsed.agents.len());

    // 4. Test validation
    println!("\n✅ Testing validation...");
    let mut invalid_manifest = parsed.clone();
    invalid_manifest.metadata.name = "".to_string();

    let invalid_yaml = serde_yaml::to_string(&invalid_manifest)?;
    match parser.parse_content(&invalid_yaml).await {
        Ok(_) => println!("   ❌ Validation should have failed"),
        Err(e) => println!("   ✅ Validation correctly failed: {}", e),
    }

    // 5. Show agent details
    println!("\n📋 Agent Details:");
    for (i, agent) in parsed.agents.iter().enumerate() {
        println!("   Agent {}: {}", i + 1, agent.name);
        println!("     Provider: {}", agent.ai_provider);
        println!("     Model: {}", agent.model);
        println!("     Environment: {:?}", agent.execution_environment);
        println!("     Memory: {} MB", agent.resource_limits.memory_mb);
        println!("     CPU: {}%", agent.resource_limits.cpu_percent);
    }

    println!("\n🎉 Simple manifest test completed successfully!");
    Ok(())
}
