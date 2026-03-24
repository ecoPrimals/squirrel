// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![forbid(unsafe_code)]
#![allow(warnings)] // Interactive demo binary; keep workspace `-D warnings` green

//! AI Tools Multi-Model Dispatch Demo
//!
//! This demo showcases the multi-model dispatch system that can seamlessly
//! utilize different AI models (API-based and local) within the same workflow.

use clap::{Parser, Subcommand};
use squirrel_ai_tools::{
    Result,
    common::capability::{AITask, SecurityLevel, SecurityRequirements, TaskType},
    common::{ChatMessage, ChatRequest, MessageRole, ModelParameters},
    dispatch::{DispatcherBuilder, MultiModelDispatcher},
    router::RoutingStrategy,
    workflows,
};
use std::collections::HashMap;
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "ai-tools-demo")]
#[command(about = "Demo of multi-model AI dispatch system")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// `OpenAI` API key
    #[arg(long)]
    openai_api_key: Option<String>,

    /// Anthropic API key
    #[arg(long)]
    anthropic_api_key: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Test basic text generation with automatic model selection
    TextGeneration {
        /// The prompt to generate text for
        prompt: String,

        /// Whether the prompt contains sensitive data
        #[arg(long)]
        sensitive: bool,

        /// Preferred model to use
        #[arg(long)]
        model: Option<String>,
    },

    /// Test code generation with preference for powerful models
    CodeGeneration {
        /// The code generation prompt
        prompt: String,

        /// Programming language
        #[arg(long)]
        language: Option<String>,
    },

    /// Test multi-model workflow with different models for different tasks
    MultiModelWorkflow {
        /// Base prompt for the workflow
        prompt: String,
    },

    /// List all available models from all providers
    ListModels,

    /// Test local model integration
    TestLocal,

    /// Benchmark different models
    Benchmark {
        /// Number of requests to send
        #[arg(short, long, default_value = "5")]
        count: usize,

        /// Prompt to use for benchmarking
        #[arg(long, default_value = "Explain quantum computing in simple terms.")]
        prompt: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("ai_tools={log_level},ai_tools_demo={log_level}"))
        .init();

    info!("Starting AI Tools Multi-Model Dispatch Demo");

    // Create the dispatcher
    let dispatcher = create_dispatcher(cli.openai_api_key, cli.anthropic_api_key).await?;

    match cli.command {
        Commands::TextGeneration {
            prompt,
            sensitive,
            model,
        } => {
            test_text_generation(&dispatcher, prompt, sensitive, model).await?;
        }

        Commands::CodeGeneration { prompt, language } => {
            test_code_generation(&dispatcher, prompt, language).await?;
        }

        Commands::MultiModelWorkflow { prompt } => {
            test_multi_model_workflow(&dispatcher, prompt).await?;
        }

        Commands::ListModels => {
            list_all_models(&dispatcher).await?;
        }

        Commands::TestLocal => {
            test_local_models(&dispatcher).await?;
        }

        Commands::Benchmark { count, prompt } => {
            benchmark_models(&dispatcher, count, prompt).await?;
        }
    }

    Ok(())
}

async fn create_dispatcher(
    openai_key: Option<String>,
    anthropic_key: Option<String>,
) -> Result<MultiModelDispatcher> {
    info!("Creating multi-model dispatcher...");

    let mut builder = DispatcherBuilder::new()
        .prefer_local_for_sensitive(true)
        .prefer_api_for_complex(true)
        .with_routing_strategy(RoutingStrategy::BestFit);

    // Add API keys if provided
    if let Some(key) = openai_key {
        info!("Adding OpenAI provider");
        builder = builder.with_api_key("openai", key);
    } else {
        warn!("No OpenAI API key provided, OpenAI models will not be available");
    }

    if let Some(key) = anthropic_key {
        info!("Adding Anthropic provider");
        builder = builder.with_api_key("anthropic", key);
    } else {
        warn!("No Anthropic API key provided, Anthropic models will not be available");
    }

    // Local AI is enabled by default in the dispatcher

    let dispatcher = builder.build()?;
    info!("Multi-model dispatcher created successfully");

    Ok(dispatcher)
}

async fn test_text_generation(
    dispatcher: &MultiModelDispatcher,
    prompt: String,
    sensitive: bool,
    preferred_model: Option<String>,
) -> Result<()> {
    info!("Testing text generation...");
    info!("Prompt: {}", prompt);
    info!("Sensitive data: {}", sensitive);
    if let Some(ref model) = preferred_model {
        info!("Preferred model: {}", model);
    }

    let start_time = std::time::Instant::now();

    let request = create_chat_request(prompt, None);
    let task = create_text_generation_task(sensitive);
    let result = if let Some(model) = preferred_model {
        // Use specific model
        dispatcher
            .process_with_model_preference(request, task, None, Some(model))
            .await?
    } else {
        // Use automatic selection
        dispatcher.process_request(request, task).await?
    };

    let duration = start_time.elapsed();

    println!("\n=== Text Generation Result ===");
    println!(
        "Generated text: {}",
        result
            .choices
            .first()
            .and_then(|c| c.content.as_ref())
            .map_or("No response", |s| s)
    );
    println!("Time taken: {duration:?}");
    println!("===============================\n");

    Ok(())
}

async fn test_code_generation(
    dispatcher: &MultiModelDispatcher,
    prompt: String,
    language: Option<String>,
) -> Result<()> {
    info!("Testing code generation...");
    info!("Prompt: {}", prompt);
    if let Some(ref lang) = language {
        info!("Language: {}", lang);
    }

    let start_time = std::time::Instant::now();
    let result = workflows::generate_text(dispatcher, &prompt, false).await?;
    let duration = start_time.elapsed();

    println!("\n=== Code Generation Result ===");
    println!("Generated code:\n{result}");
    println!("Time taken: {duration:?}");
    println!("===============================\n");

    Ok(())
}

async fn test_multi_model_workflow(
    dispatcher: &MultiModelDispatcher,
    base_prompt: String,
) -> Result<()> {
    info!("Testing multi-model workflow...");

    // Create multiple requests with different characteristics
    let requests = vec![
        // Simple summarization (can use local model)
        (
            create_chat_request(
                format!("Summarize this in one sentence: {base_prompt}"),
                None,
            ),
            create_text_generation_task(false),
            Some("llama3-8b".to_string()),
        ),
        // Complex analysis (prefer powerful API model)
        (
            create_chat_request(
                format!(
                    "Provide a detailed analysis of the implications and potential solutions for: {base_prompt}"
                ),
                None,
            ),
            AITask {
                task_type: TaskType::TextGeneration,
                complexity_score: Some(85),
                priority: 90,
                ..Default::default()
            },
            Some("gpt-4".to_string()),
        ),
        // Sensitive data processing (prefer local model)
        (
            create_chat_request(
                format!("Process this sensitive information carefully: {base_prompt}"),
                None,
            ),
            create_text_generation_task(true),
            None, // Let router decide based on sensitivity
        ),
    ];

    let start_time = std::time::Instant::now();
    // Process each request individually since process_multi_model_workflow doesn't exist yet
    let mut results = Vec::new();
    for (request, task, _preferred_model) in requests {
        let result = dispatcher.process_request(request, task).await?;
        results.push(result);
    }
    let duration = start_time.elapsed();

    println!("\n=== Multi-Model Workflow Results ===");
    for (i, response) in results.iter().enumerate() {
        let text = response
            .choices
            .first()
            .and_then(|c| c.content.as_deref())
            .unwrap_or("No response");
        println!("Response {}: {}", i + 1, text);
        println!("---");
    }
    println!("Total time taken: {duration:?}");
    println!("=====================================\n");

    Ok(())
}

async fn list_all_models(dispatcher: &MultiModelDispatcher) -> Result<()> {
    info!("Listing all available models...");

    let models = dispatcher.list_all_available_models().await?;

    println!("\n=== Available Models ===");
    for (provider, provider_models) in models {
        println!("Provider: {provider}");
        for model in provider_models {
            println!("  - {model}");
        }
        println!();
    }
    println!("========================\n");

    Ok(())
}

async fn test_local_models(dispatcher: &MultiModelDispatcher) -> Result<()> {
    info!("Testing local model integration...");

    let request = create_chat_request("Hello! Can you tell me about yourself?".to_string(), None);

    let task = AITask {
        task_type: TaskType::TextGeneration,
        security_requirements: SecurityRequirements {
            contains_sensitive_data: true,
            requires_encryption: true,
            requires_audit_logging: true,
            security_level: if true {
                SecurityLevel::High
            } else {
                SecurityLevel::Medium
            },
            geo_restrictions: None,
        },
        ..Default::default()
    };

    match dispatcher.process_request(request, task).await {
        Ok(response) => {
            println!("\n=== Local Model Test Result ===");
            let text = response
                .choices
                .first()
                .and_then(|c| c.content.as_deref())
                .unwrap_or("No response");
            println!("Response: {text}");
            println!("===============================\n");
        }
        Err(e) => {
            warn!("Local model test failed: {}", e);
            println!("\n=== Local Model Test ===");
            println!("Local models may not be available. Error: {e}");
            println!(
                "To use local models, ensure a local AI server is running (Ollama, llama.cpp, vLLM, etc.)."
            );
            println!("========================\n");
        }
    }

    Ok(())
}

async fn benchmark_models(
    dispatcher: &MultiModelDispatcher,
    count: usize,
    prompt: String,
) -> Result<()> {
    info!("Benchmarking models with {} requests...", count);

    let mut results = HashMap::new();

    // Get available models
    let available_models = dispatcher.list_all_available_models().await?;

    for (provider, models) in available_models {
        for model in models.iter().take(2) {
            // Limit to 2 models per provider for demo
            info!("Benchmarking model: {} from provider: {}", model, provider);

            let mut times = Vec::new();
            let mut success_count = 0;

            for i in 0..count {
                let request = create_chat_request(format!("{} (request {})", prompt, i + 1), None);
                let task = create_text_generation_task(false);

                let start = std::time::Instant::now();
                match dispatcher
                    .process_with_model_preference(
                        request,
                        task,
                        Some(provider.clone()),
                        Some(model.clone()),
                    )
                    .await
                {
                    Ok(_) => {
                        times.push(start.elapsed());
                        success_count += 1;
                    }
                    Err(e) => {
                        warn!("Request failed for {}/{}: {}", provider, model, e);
                    }
                }
            }

            if !times.is_empty() {
                let n = u32::try_from(times.len()).unwrap_or(1);
                let avg_time = times.iter().sum::<std::time::Duration>() / n.max(1);
                #[allow(
                    clippy::cast_lossless,
                    clippy::cast_precision_loss,
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss
                )] // Metrics/benchmark float conversions are intentional
                let success_rate =
                    f64::from(success_count as u32) / f64::from(count as u32) * 100.0;

                results.insert(
                    format!("{provider}/{model}"),
                    (avg_time, success_rate, success_count),
                );
            }
        }
    }

    println!("\n=== Benchmark Results ===");
    for (model, (avg_time, success_rate, success_count)) in results {
        println!("Model: {model}");
        println!("  Average time: {avg_time:?}");
        println!("  Success rate: {success_rate:.1}% ({success_count}/{count})");
        println!();
    }
    println!("=========================\n");

    Ok(())
}

fn create_chat_request(prompt: String, model: Option<String>) -> ChatRequest {
    ChatRequest {
        model,
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: Some(prompt),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }],
        parameters: Some(ModelParameters {
            temperature: Some(0.7),
            max_tokens: Some(1000),
            top_p: None,
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            stream: Some(false),
            tool_choice: None,
        }),
        tools: None,
    }
}

const fn create_text_generation_task(sensitive: bool) -> AITask {
    AITask {
        task_type: TaskType::TextGeneration,
        required_model_type: None,
        min_context_size: None,
        requires_streaming: false,
        requires_function_calling: false,
        requires_tool_use: false,
        security_requirements: SecurityRequirements {
            contains_sensitive_data: sensitive,
            requires_encryption: sensitive,
            requires_audit_logging: sensitive,
            security_level: if sensitive {
                SecurityLevel::High
            } else {
                SecurityLevel::Medium
            },
            geo_restrictions: None,
        },
        complexity_score: Some(30),
        priority: 50,
    }
}
