// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::needless_pass_by_value,
    clippy::significant_drop_tightening,
    clippy::field_reassign_with_default,
    clippy::default_trait_access,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::option_if_let_else,
    clippy::doc_markdown,
    clippy::struct_field_names,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::trivially_copy_pass_by_ref,
    clippy::unused_self,
    clippy::unused_async,
    clippy::unnecessary_wraps,
    clippy::semicolon_if_nothing_returned,
    clippy::match_wildcard_for_single_variants,
    clippy::match_same_arms,
    clippy::explicit_iter_loop,
    clippy::uninlined_format_args,
    clippy::equatable_if_let,
    clippy::assertions_on_constants,
    missing_docs,
    unused_imports,
    unused_variables,
    dead_code,
    deprecated
)]
//! Simple biome.yaml manifest test
//!
//! This test validates the basic manifest parsing functionality.

use squirrel::biomeos_integration::manifest::BiomeManifestParser;

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
    let yaml_content = serde_yaml_ng::to_string(&template)?;
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
    invalid_manifest.metadata.name = String::new();

    let invalid_yaml = serde_yaml_ng::to_string(&invalid_manifest)?;
    match parser.parse_content(&invalid_yaml).await {
        Ok(_) => println!("   ❌ Validation should have failed"),
        Err(e) => println!("   ✅ Validation correctly failed: {e}"),
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
