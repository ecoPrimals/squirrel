// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

#![allow(
    clippy::unnested_or_patterns,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::return_self_not_must_use,
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::similar_names,
    clippy::uninlined_format_args,
    clippy::struct_field_names,
    clippy::use_self
)]

//! AI configuration management CLI tool.

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Set the OpenAI API key
    SetKey {
        /// The API key to set
        key: String,

        /// The provider (openai, anthropic, gemini)
        #[arg(default_value = "openai")]
        provider: String,
    },
    /// Show the current configuration status
    Status,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct ApiConfig {
    openai_api_key: Option<String>,
    anthropic_api_key: Option<String>,
    gemini_api_key: Option<String>,
}

impl ApiConfig {
    fn load() -> Result<Self, Box<dyn Error>> {
        let config_path = get_config_path()?;

        if !config_path.exists() {
            return Ok(ApiConfig::default());
        }

        let content = fs::read_to_string(&config_path)?;
        let config: ApiConfig = serde_json::from_str(&content)?;

        Ok(config)
    }

    fn save(&self) -> Result<(), Box<dyn Error>> {
        let config_path = get_config_path()?;

        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, content)?;

        Ok(())
    }

    fn set_api_key(&mut self, provider: &str, key: String) {
        match provider.to_lowercase().as_str() {
            "openai" => self.openai_api_key = Some(key),
            "anthropic" => self.anthropic_api_key = Some(key),
            "gemini" => self.gemini_api_key = Some(key),
            _ => eprintln!("Warning: Unknown provider {provider}, key not saved"),
        }
    }

    fn get_api_key(&self, provider: &str) -> Option<&String> {
        match provider.to_lowercase().as_str() {
            "openai" => self.openai_api_key.as_ref(),
            "anthropic" => self.anthropic_api_key.as_ref(),
            "gemini" => self.gemini_api_key.as_ref(),
            _ => None,
        }
    }
}

fn get_config_path() -> Result<PathBuf, Box<dyn Error>> {
    let config_dir = dirs::config_dir().ok_or("Could not determine config directory")?;
    let config_path = config_dir.join("squirrel-ai-tools").join("api-keys.json");
    Ok(config_path)
}

fn validate_key(key: &str) -> Result<(), Box<dyn Error>> {
    if key.is_empty() {
        return Err("API key cannot be empty".into());
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::SetKey { key, provider } => {
            validate_key(&key)?;
            let mut config = ApiConfig::load()?;
            config.set_api_key(&provider, key);
            config.save()?;
            println!("API key for {provider} set successfully");
        }
        Commands::Status => {
            let config = ApiConfig::load()?;

            let providers = ["openai", "anthropic", "gemini"];
            let mut has_any = false;

            for provider in providers {
                match config.get_api_key(provider) {
                    Some(_) => {
                        println!("{provider} API key: Configured");
                        has_any = true;
                    }
                    None => {
                        println!("{provider} API key: Not configured");
                    }
                }
            }

            if !has_any {
                println!("\nNo API keys configured. Use the 'set-key' command to configure.");
            }
        }
    }

    Ok(())
}
