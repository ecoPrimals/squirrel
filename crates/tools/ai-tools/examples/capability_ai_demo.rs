// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Capability-Based AI Demo
//!
//! This demo shows how to use the new capability_ai pattern for AI operations.
//! Instead of directly calling HTTP APIs (OpenAI, Anthropic, etc.), we delegate
//! all HTTP to Songbird via Unix sockets.
//!
//! **Benefits**:
//! - ✅ Zero reqwest dependency (100% Pure Rust)
//! - ✅ Zero ring C dependency
//! - ✅ TRUE ecoBin compliant
//! - ✅ Cross-compiles to all architectures
//! - ✅ Ecological - Songbird handles all HTTP/TLS
//!
//! **Pattern**: Deploy like an infant - know nothing, discover everything!

use squirrel_ai_tools::capability_ai::{AiClient, AiClientConfig, ChatMessage, ChatOptions};
use std::path::PathBuf;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Capability AI Demo - TRUE ecoBin Pattern!");
    info!("");

    // Demo 1: Basic Chat Completion
    demo_basic_chat().await?;

    // Demo 2: With Options
    demo_chat_with_options().await?;

    // Demo 3: Multi-Turn Conversation
    demo_conversation().await?;

    // Demo 4: Embeddings
    demo_embeddings().await?;

    // Demo 5: Text Generation
    demo_text_generation().await?;

    // Demo 6: Error Handling
    demo_error_handling().await;

    info!("");
    info!("✅ Demo complete! All operations used Unix sockets, zero HTTP!");
    info!("   Pattern: capability_ai → Songbird (Unix) → AI Provider (HTTP)");
    info!("");
    info!("📚 See docs/CAPABILITY_AI_MIGRATION_GUIDE.md for more examples!");

    Ok(())
}

/// Demo 1: Basic chat completion
async fn demo_basic_chat() -> anyhow::Result<()> {
    info!("📝 Demo 1: Basic Chat Completion");
    info!("   Creating AiClient from environment...");

    let client = AiClient::from_env()?;

    let messages = vec![
        ChatMessage::system("You are a helpful Rust programming assistant"),
        ChatMessage::user("Explain async/await in Rust in one sentence"),
    ];

    info!("   Sending request to GPT-4 via Songbird...");
    let response = client.chat_completion("gpt-4", messages, None).await?;

    info!("   Response: {}", response.content);
    info!(
        "   Tokens used: {}",
        response.usage.as_ref().map(|u| u.total_tokens).unwrap_or(0)
    );
    info!("");

    Ok(())
}

/// Demo 2: Chat with custom options
async fn demo_chat_with_options() -> anyhow::Result<()> {
    info!("⚙️  Demo 2: Chat with Custom Options");

    let client = AiClient::from_env()?;

    let messages = vec![
        ChatMessage::system("You are a concise technical writer"),
        ChatMessage::user("What are the three pillars of Rust?"),
    ];

    let options = ChatOptions {
        temperature: Some(0.7),
        max_tokens: Some(150),
        top_p: Some(0.9),
        ..Default::default()
    };

    info!("   Sending with temperature=0.7, max_tokens=150...");
    let response = client
        .chat_completion("gpt-3.5-turbo", messages, Some(options))
        .await?;

    info!("   Response: {}", response.content);
    info!("");

    Ok(())
}

/// Demo 3: Multi-turn conversation
async fn demo_conversation() -> anyhow::Result<()> {
    info!("💬 Demo 3: Multi-Turn Conversation");

    let client = AiClient::from_env()?;

    let mut conversation = vec![ChatMessage::system(
        "You are a helpful assistant that answers questions about Rust",
    )];

    // Turn 1
    conversation.push(ChatMessage::user("What is a trait?"));
    info!("   User: What is a trait?");

    let response1 = client
        .chat_completion("gpt-4", conversation.clone(), None)
        .await?;
    info!("   Assistant: {}", response1.content);

    conversation.push(ChatMessage::assistant(&response1.content));

    // Turn 2
    conversation.push(ChatMessage::user("Can you show an example?"));
    info!("   User: Can you show an example?");

    let response2 = client
        .chat_completion("gpt-4", conversation.clone(), None)
        .await?;
    info!("   Assistant: {}", response2.content);
    info!("");

    Ok(())
}

/// Demo 4: Generate embeddings
async fn demo_embeddings() -> anyhow::Result<()> {
    info!("🔢 Demo 4: Generate Embeddings");

    let client = AiClient::from_env()?;

    let text = "Rust is a systems programming language focused on safety and performance";
    info!("   Text: {}", text);

    let embeddings = client
        .create_embedding("text-embedding-ada-002", text)
        .await?;

    info!("   Generated {} dimensional embedding", embeddings.len());
    info!("   First 5 values: {:?}", &embeddings[..5]);
    info!("");

    Ok(())
}

/// Demo 5: Text generation
async fn demo_text_generation() -> anyhow::Result<()> {
    info!("✍️  Demo 5: Text Generation");

    let client = AiClient::from_env()?;

    let prompt = "Write a haiku about Rust programming";
    info!("   Prompt: {}", prompt);

    let text = client
        .text_generation("claude-3-opus", prompt, Some(100))
        .await?;

    info!("   Generated text:");
    info!("   {}", text);
    info!("");

    Ok(())
}

/// Demo 6: Error handling
async fn demo_error_handling() {
    info!("⚠️  Demo 6: Error Handling");

    // Example 1: Handle missing socket
    info!("   Example: Handling missing socket...");
    let config = AiClientConfig {
        socket_path: PathBuf::from("/nonexistent/socket.sock"),
        timeout_secs: 5,
        ..Default::default()
    };

    match AiClient::new(config) {
        Ok(_) => info!("   Client created"),
        Err(e) => warn!("   Expected error: {}", e),
    }

    // Example 2: Graceful timeout handling
    if let Ok(client) = AiClient::from_env() {
        let messages = vec![ChatMessage::user("test")];

        match client.chat_completion("gpt-4", messages, None).await {
            Ok(response) => info!("   Response: {}", response.content),
            Err(e) => warn!("   Error (Songbird might not be running): {}", e),
        }
    }

    info!("");
}
