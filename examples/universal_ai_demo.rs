// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Universal AI Provider Demo
//!
//! This demo shows how the universal AI provider works with pure capability-based
//! discovery. Any primal or AI service can integrate by simply announcing their
//! capabilities - no hardcoded names or identities needed.
//!
//! ## How to run this demo:
//!
//! 1. **Set up capability announcements** (any service can do this):
//!    ```bash
//!    # A local Ollama instance announces text generation capability
//!    export CAPABILITY_TEXT_GENERATION_1_ENDPOINT=http://localhost:11434
//!    export CAPABILITY_TEXT_GENERATION_1_FORMAT=openai
//!    export CAPABILITY_TEXT_GENERATION_1_AUTH=none
//!    export CAPABILITY_TEXT_GENERATION_1_COST=0.0
//!    
//!    # A Toadstool instance announces code generation capability
//!    export CAPABILITY_CODE_GENERATION_1_ENDPOINT=http://localhost:8080
//!    export CAPABILITY_CODE_GENERATION_1_FORMAT=openai
//!    export CAPABILITY_CODE_GENERATION_1_AUTH=none
//!    export CAPABILITY_CODE_GENERATION_1_COST=0.0
//!    
//!    # An OpenRouter service announces text generation (paid)
//!    export CAPABILITY_TEXT_GENERATION_2_ENDPOINT=https://openrouter.ai/api/v1
//!    export CAPABILITY_TEXT_GENERATION_2_FORMAT=openai
//!    export CAPABILITY_TEXT_GENERATION_2_AUTH=bearer
//!    export CAPABILITY_TEXT_GENERATION_2_COST=0.001
//!    export AI_BEARER_TOKEN=your_openrouter_key
//!    
//!    # A HuggingFace model via PyO3 (hypothetical community primal)
//!    export CAPABILITY_CODE_GENERATION_2_ENDPOINT=http://localhost:8081
//!    export CAPABILITY_CODE_GENERATION_2_FORMAT=openai
//!    export CAPABILITY_CODE_GENERATION_2_AUTH=none
//!    export CAPABILITY_CODE_GENERATION_2_COST=0.0
//!    ```
//!
//! 2. **Run the demo**:
//!    ```bash
//!    cargo run --example universal_ai_demo
//!    ```
//!
//! The system will:
//! - Discover all announced capabilities automatically
//! - Route requests to the best capability provider based on requirements
//! - Work with any primal that announces AI capabilities
//! - Support new providers added by the community

use squirrel_ai_tools::local::{create_universal_ai_provider, setup_development_capabilities, UniversalAIConfig};
use squirrel_ai_tools::common::{ChatRequest, ChatMessage, MessageRole};
use tokio;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🌟 Starting Universal AI Provider Demo");
    info!("📡 This demo shows capability-based discovery without hardcoding provider names");

    // Set up development capabilities (normally done by each primal/service)
    setup_development_capabilities();
    info!("✅ Development capabilities configured");

    // Create universal AI provider with default config
    info!("🔧 Creating Universal AI Provider...");
    let provider = create_universal_ai_provider(None).await?;
    info!("✅ Universal AI Provider created and discovery completed");

    // Demo 1: Text generation request
    info!("\n🤖 Demo 1: Text Generation Request");
    let text_request = ChatRequest {
        model: None, // No specific model - let capability system choose
        messages: vec![
            ChatMessage {
                role: MessageRole::User,
                content: Some("Explain how capability-based discovery works in distributed systems".to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }
        ],
        parameters: None,
        tools: None,
    };

    match provider.process_chat(text_request).await {
        Ok(response) => {
            info!("✅ Text generation successful!");
            if let Some(choice) = response.choices.first() {
                if let Some(content) = &choice.message.content {
                    info!("📝 Response: {}", content);
                }
            }
            info!("📊 Used model: {}", response.model);
        }
        Err(e) => {
            info!("❌ Text generation failed: {}", e);
        }
    }

    // Demo 2: Code generation request  
    info!("\n💻 Demo 2: Code Generation Request");
    let code_request = ChatRequest {
        model: Some("code-model".to_string()), // Hint for code generation
        messages: vec![
            ChatMessage {
                role: MessageRole::User,
                content: Some("Write a Rust function that demonstrates capability-based service discovery".to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }
        ],
        parameters: None,
        tools: None,
    };

    match provider.process_chat(code_request).await {
        Ok(response) => {
            info!("✅ Code generation successful!");
            if let Some(choice) = response.choices.first() {
                if let Some(content) = &choice.message.content {
                    info!("💻 Generated code: {}", content);
                }
            }
            info!("📊 Used model: {}", response.model);
        }
        Err(e) => {
            info!("❌ Code generation failed: {}", e);
        }
    }

    // Demo 3: Show discovered capabilities
    info!("\n📋 Demo 3: Discovered Ecosystem Capabilities");
    provider.refresh_capabilities().await?;
    info!("✅ Capability discovery refresh completed");

    // Demo 4: Test with different request types
    info!("\n🔍 Demo 4: Testing Different Request Types");
    
    let analysis_request = ChatRequest {
        model: None,
        messages: vec![
            ChatMessage {
                role: MessageRole::User,
                content: Some("Analyze the benefits of ecosystem-based AI routing".to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }
        ],
        parameters: None,
        tools: None,
    };

    match provider.process_chat(analysis_request).await {
        Ok(response) => {
            info!("✅ Analysis request successful!");
            info!("📊 Used model: {}", response.model);
        }
        Err(e) => {
            info!("❌ Analysis request failed: {}", e);
        }
    }

    info!("\n🎉 Universal AI Provider Demo Complete!");
    info!("🌟 Key Features Demonstrated:");
    info!("   • No hardcoded provider names - pure capability discovery");
    info!("   • Works with any primal that announces AI capabilities");
    info!("   • Automatic routing based on request type and requirements");
    info!("   • Community extensible - new primals integrate automatically");
    info!("   • Supports all AI providers: PyO3, llama.cpp, Ollama, OpenRouter, HuggingFace");
    info!("   • Network effects: leverages Toadstool compute, any primal's capabilities");

    info!("\n💡 To integrate a new primal:");
    info!("   1. Set environment variables announcing your capabilities");
    info!("   2. Run your service on the announced endpoint"); 
    info!("   3. The universal provider discovers and uses it automatically");
    info!("   4. No code changes needed in Squirrel - pure capability-based integration!");

    Ok(())
}

/// Example of how a hypothetical new community primal would integrate
pub fn example_community_primal_integration() {
    // A new "Dolphin" primal emerges from the community with specialized math capabilities
    std::env::set_var("CAPABILITY_MATHEMATICAL_ANALYSIS_1_ENDPOINT", "http://localhost:9000");
    std::env::set_var("CAPABILITY_MATHEMATICAL_ANALYSIS_1_FORMAT", "openai");
    std::env::set_var("CAPABILITY_MATHEMATICAL_ANALYSIS_1_AUTH", "none");
    std::env::set_var("CAPABILITY_MATHEMATICAL_ANALYSIS_1_COST", "0.0");
    
    // Squirrel's universal provider will discover this automatically!
    // No code changes needed in Squirrel itself.
    info!("🐬 Hypothetical 'Dolphin' primal would integrate automatically!");
}

/// Example of how Toadstool compute capabilities would be discovered
pub fn example_toadstool_integration() {
    // Toadstool announces its compute capabilities for hosting AI models
    std::env::set_var("CAPABILITY_MODEL_HOSTING_1_ENDPOINT", "http://localhost:8080");
    std::env::set_var("CAPABILITY_MODEL_HOSTING_1_FORMAT", "custom");
    std::env::set_var("CAPABILITY_MODEL_HOSTING_1_AUTH", "none");
    std::env::set_var("CAPABILITY_MODEL_HOSTING_1_COST", "0.0");
    
    // The universal provider can deploy models to Toadstool automatically
    info!("🍄 Toadstool compute integration works through capabilities!");
}

/// Example of how any AI provider integrates without hardcoding
pub fn example_arbitrary_ai_provider() {
    // Someone runs a local llama.cpp instance
    std::env::set_var("CAPABILITY_LOCAL_INFERENCE_1_ENDPOINT", "http://localhost:8080");
    std::env::set_var("CAPABILITY_LOCAL_INFERENCE_1_FORMAT", "openai");  
    std::env::set_var("CAPABILITY_LOCAL_INFERENCE_1_AUTH", "none");
    std::env::set_var("CAPABILITY_LOCAL_INFERENCE_1_COST", "0.0");
    
    // Someone runs a HuggingFace model via PyO3
    std::env::set_var("CAPABILITY_HUGGINGFACE_INFERENCE_1_ENDPOINT", "http://localhost:8081");
    std::env::set_var("CAPABILITY_HUGGINGFACE_INFERENCE_1_FORMAT", "openai");
    std::env::set_var("CAPABILITY_HUGGINGFACE_INFERENCE_1_AUTH", "none"); 
    std::env::set_var("CAPABILITY_HUGGINGFACE_INFERENCE_1_COST", "0.0");
    
    // Someone uses OpenRouter for cloud models
    std::env::set_var("CAPABILITY_CLOUD_INFERENCE_1_ENDPOINT", "https://openrouter.ai/api/v1");
    std::env::set_var("CAPABILITY_CLOUD_INFERENCE_1_FORMAT", "openai");
    std::env::set_var("CAPABILITY_CLOUD_INFERENCE_1_AUTH", "bearer");
    std::env::set_var("CAPABILITY_CLOUD_INFERENCE_1_COST", "0.002");
    
    info!("🔄 All AI providers integrate through capability announcements!");
} 