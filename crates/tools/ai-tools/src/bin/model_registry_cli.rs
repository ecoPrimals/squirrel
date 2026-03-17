// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::too_many_lines,
    clippy::ignored_unit_patterns,
    clippy::single_match_else,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::return_self_not_must_use,
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::manual_string_new,
    clippy::redundant_closure_for_method_calls,
    clippy::unreadable_literal
)]

//! Model registry management CLI tool.

use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

/// Model registry CLI for managing AI model capabilities
#[derive(Parser)]
#[command(author, version, about = "CLI tool for managing AI model registry")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available models
    List {
        /// Provider name (optional)
        #[arg(short, long)]
        provider: Option<String>,
    },

    /// Add a new model to the registry
    Add {
        /// Model name
        #[arg(short, long)]
        name: String,

        /// Provider name
        #[arg(short, long)]
        provider: String,

        /// Model version
        #[arg(short, long)]
        version: Option<String>,

        /// Output file to save the updated registry
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Import models from a JSON file
    Import {
        /// Input file
        #[arg(short, long)]
        input: PathBuf,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Export models to a JSON file
    Export {
        /// Output file
        #[arg(short, long)]
        output: PathBuf,

        /// Provider filter
        #[arg(short, long)]
        provider: Option<String>,
    },
}

/// Simple model registry for AI models
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ModelRegistry {
    models: HashMap<String, HashMap<String, ModelCapabilities>>,
}

/// Model capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelCapabilities {
    /// Model name
    pub name: String,

    /// Provider ID
    pub provider_id: String,

    /// Model version
    pub version: Option<String>,

    /// Supported model types
    #[serde(default)]
    pub model_types: Vec<String>,

    /// Supported task types
    #[serde(default)]
    pub task_types: Vec<String>,

    /// Maximum context size in tokens
    pub max_context_size: Option<usize>,

    /// Whether streaming is supported
    #[serde(default)]
    pub supports_streaming: bool,

    /// Whether function calling is supported
    #[serde(default)]
    pub supports_function_calling: bool,

    /// Whether tool use is supported
    #[serde(default)]
    pub supports_tool_use: bool,
}

impl ModelRegistry {
    /// Create a new empty registry
    fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    /// Add a model to the registry
    fn add_model(&mut self, capabilities: ModelCapabilities) {
        let provider_id = capabilities.provider_id.clone();
        let model_id = capabilities.name.clone();

        self.models
            .entry(provider_id)
            .or_default()
            .insert(model_id, capabilities);
    }

    /// Get all providers
    fn get_providers(&self) -> Vec<String> {
        self.models.keys().cloned().collect()
    }

    /// Get all models for a provider
    fn get_provider_models(&self, provider_id: &str) -> Vec<String> {
        self.models
            .get(provider_id)
            .map(|models| models.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get a specific model
    fn get_model(&self, provider_id: &str, model_id: &str) -> Option<&ModelCapabilities> {
        self.models
            .get(provider_id)
            .and_then(|models| models.get(model_id))
    }

    /// Load from a file
    fn load_from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Handle JSON with "models" key
        let json: serde_json::Value = serde_json::from_str(&contents)?;

        if let Some(models_obj) = json.get("models") {
            let registry_json = serde_json::json!({ "models": models_obj });
            let registry: Self = serde_json::from_value(registry_json)?;
            return Ok(registry);
        }

        // Standard format
        let registry: Self = serde_json::from_str(&contents)?;
        Ok(registry)
    }

    /// Save to a file
    fn save_to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::json!({ "models": self.models });
        let contents = serde_json::to_string_pretty(&json)?;
        fs::write(path, contents)?;
        Ok(())
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::List { provider } => {
            // Initialize with some defaults
            let mut registry = ModelRegistry::new();

            // Add example models for demonstration
            registry.add_model(ModelCapabilities {
                name: "gpt-4".to_string(),
                provider_id: "openai".to_string(),
                version: Some("2023-03-15".to_string()),
                model_types: vec!["LargeLanguageModel".to_string()],
                task_types: vec!["TextGeneration".to_string()],
                max_context_size: Some(8192),
                supports_streaming: true,
                supports_function_calling: true,
                supports_tool_use: true,
            });

            registry.add_model(ModelCapabilities {
                name: "claude-3-opus".to_string(),
                provider_id: "anthropic".to_string(),
                version: Some("2024-02-29".to_string()),
                model_types: vec!["LargeLanguageModel".to_string(), "MultiModal".to_string()],
                task_types: vec![
                    "TextGeneration".to_string(),
                    "ImageUnderstanding".to_string(),
                ],
                max_context_size: Some(200000),
                supports_streaming: true,
                supports_function_calling: true,
                supports_tool_use: true,
            });

            if let Some(provider_id) = provider {
                // List models for specific provider
                println!("Models for provider '{provider_id}':");
                let models = registry.get_provider_models(provider_id);
                if models.is_empty() {
                    println!("  No models found");
                } else {
                    for model_id in models {
                        let model = registry.get_model(provider_id, &model_id).unwrap();
                        println!("  {model_id}");
                        println!(
                            "    Version: {}",
                            model.version.as_deref().unwrap_or("unknown")
                        );
                        println!("    Context size: {}", model.max_context_size.unwrap_or(0));
                        println!("    Model types: {}", model.model_types.join(", "));
                        println!("    Task types: {}", model.task_types.join(", "));
                    }
                }
            } else {
                // List all providers and their models
                println!("Available providers:");
                for provider_id in registry.get_providers() {
                    println!("  {provider_id}");
                    let models = registry.get_provider_models(&provider_id);
                    for model_id in models {
                        println!("    {model_id}");
                    }
                }
            }
        }
        Commands::Add {
            name,
            provider,
            version,
            output,
        } => {
            let mut registry = if output.exists() {
                match ModelRegistry::load_from_file(output) {
                    Ok(reg) => reg,
                    Err(e) => {
                        eprintln!("Error loading registry: {e}");
                        process::exit(1);
                    }
                }
            } else {
                ModelRegistry::new()
            };

            let capabilities = ModelCapabilities {
                name: name.clone(),
                provider_id: provider.clone(),
                version: version.clone(),
                model_types: vec!["LargeLanguageModel".to_string()],
                task_types: vec!["TextGeneration".to_string()],
                max_context_size: Some(8192),
                supports_streaming: true,
                supports_function_calling: false,
                supports_tool_use: false,
            };

            registry.add_model(capabilities);

            match registry.save_to_file(output) {
                Ok(_) => println!("Model '{name}' added to registry"),
                Err(e) => {
                    eprintln!("Error saving registry: {e}");
                    process::exit(1);
                }
            }
        }
        Commands::Import { input, output } => {
            // Load registry from input file
            let registry = match ModelRegistry::load_from_file(input) {
                Ok(reg) => reg,
                Err(e) => {
                    eprintln!("Error loading registry from {}: {}", input.display(), e);
                    process::exit(1);
                }
            };

            // Save to output file (or stdout if none specified)
            match output {
                Some(output_path) => {
                    if let Err(e) = registry.save_to_file(output_path) {
                        eprintln!("Error saving registry to {}: {}", output_path.display(), e);
                        process::exit(1);
                    }
                    println!(
                        "Registry imported from {} and saved to {}",
                        input.display(),
                        output_path.display()
                    );
                }
                None => {
                    // Output to stdout
                    let json = serde_json::to_string_pretty(&registry).unwrap();
                    println!("{json}");
                }
            }
        }
        Commands::Export { output, provider } => {
            // Create a registry with some example models
            let mut registry = ModelRegistry::new();

            // Add example models
            registry.add_model(ModelCapabilities {
                name: "gpt-4".to_string(),
                provider_id: "openai".to_string(),
                version: Some("2023-03-15".to_string()),
                model_types: vec!["LargeLanguageModel".to_string()],
                task_types: vec!["TextGeneration".to_string()],
                max_context_size: Some(8192),
                supports_streaming: true,
                supports_function_calling: true,
                supports_tool_use: true,
            });

            registry.add_model(ModelCapabilities {
                name: "gemini-1.5-pro".to_string(),
                provider_id: "gemini".to_string(),
                version: Some("2024-05".to_string()),
                model_types: vec!["LargeLanguageModel".to_string(), "MultiModal".to_string()],
                task_types: vec![
                    "TextGeneration".to_string(),
                    "ImageUnderstanding".to_string(),
                ],
                max_context_size: Some(1000000),
                supports_streaming: true,
                supports_function_calling: true,
                supports_tool_use: false,
            });

            // Filter by provider if specified
            if let Some(provider_id) = provider {
                let mut filtered_registry = ModelRegistry::new();
                if let Some(models) = registry.models.get(provider_id) {
                    filtered_registry
                        .models
                        .insert(provider_id.clone(), models.clone());
                    registry = filtered_registry;
                } else {
                    eprintln!("Provider '{provider_id}' not found");
                    process::exit(1);
                }
            }

            match registry.save_to_file(output) {
                Ok(_) => println!("Registry exported to {}", output.display()),
                Err(e) => {
                    eprintln!("Error exporting registry: {e}");
                    process::exit(1);
                }
            }
        }
    }
}
