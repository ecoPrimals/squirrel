// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

#![allow(
    clippy::map_unwrap_or,
    clippy::redundant_closure,
    clippy::redundant_closure_for_method_calls
)]

//! AI Rate Limiting Demo command-line tool
//!
//! This binary demonstrates how the rate limiting functionality of the AI tools works.

use std::time::Instant;

use squirrel_ai_tools::{
    common::{create_provider_client, ChatRequest},
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments for provider and rate limit
    let args: Vec<String> = std::env::args().collect();
    let provider = args.get(1).map(|s| s.as_str()).unwrap_or("openai");
    let rate_limit = args.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(5);

    println!("AI Rate Limiting Demo");
    println!("--------------------");
    println!("Using provider: {provider}");
    println!("Rate limit: {rate_limit} requests per minute");

    // Load API key from environment variables
    // Backward compatibility: using legacy error type until full migration to universal_error
    #[allow(deprecated)]
    let api_key = match provider {
        "openai" => std::env::var("OPENAI_API_KEY")
            .or_else(|_| std::env::var("OPENAI_KEY"))
            .map_err(|_| {
                squirrel_ai_tools::Error::Configuration(
                    "OpenAI API key not found. Set OPENAI_API_KEY environment variable.".into(),
                )
            })?,
        "anthropic" => std::env::var("ANTHROPIC_API_KEY")
            .or_else(|_| std::env::var("ANTHROPIC_KEY"))
            .map_err(|_| {
                squirrel_ai_tools::Error::Configuration(
                    "Anthropic API key not found. Set ANTHROPIC_API_KEY environment variable."
                        .into(),
                )
            })?,
        "gemini" => std::env::var("GEMINI_API_KEY")
            .or_else(|_| std::env::var("GOOGLE_API_KEY"))
            .map_err(|_| {
                squirrel_ai_tools::Error::Configuration(
                    "Gemini API key not found. Set GEMINI_API_KEY environment variable.".into(),
                )
            })?,
        _ => {
            return Err(squirrel_ai_tools::Error::UnsupportedProvider(format!(
                "Unsupported provider: {provider}"
            )))
        }
    };

    // Create client with rate limiting
    let client = create_provider_client(provider, &api_key)?;

    // Note: Rate limiting is configured at the client level
    // This demo shows how rate limiting works in practice
    println!("Note: Rate limiting is configured at the client level for provider: {provider}");

    // Run a series of requests and observe rate limiting
    println!("\nSending 10 requests in quick succession...\n");
    let start = Instant::now();

    for i in 1..=10 {
        let request_start = Instant::now();
        println!("Request #{i}: Starting");

        let request = ChatRequest::new()
            .add_system("You are a helpful assistant. Keep responses very short.")
            .add_user(&format!("What is {i} + {i}? Answer with just the number."));

        match client.chat(request).await {
            Ok(response) => {
                let duration = request_start.elapsed();
                println!("Request #{i}: Completed in {duration:.2?}");
                if let Some(content) = &response.choices[0].content {
                    println!("Response: {content}");
                }
            }
            Err(e) => {
                let duration = request_start.elapsed();
                println!("Request #{i}: Failed after {duration:.2?}: {e}");
            }
        }

        println!(); // Add a blank line between requests
    }

    let total_duration = start.elapsed();
    println!("All requests completed in {total_duration:.2?}");
    println!("\nNote: Requests beyond the rate limit should have waited or failed (depends on configuration).");

    Ok(())
}
