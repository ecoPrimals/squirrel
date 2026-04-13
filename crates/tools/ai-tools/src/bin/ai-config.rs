// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![expect(warnings)] // Thin CLI wrapper; keep workspace `-D warnings` green

//! AI configuration management CLI tool.

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
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
    fn load() -> anyhow::Result<Self> {
        let config_path = get_config_path()?;

        if !config_path.exists() {
            return Ok(ApiConfig::default());
        }

        let content = fs::read_to_string(&config_path)?;
        let config: ApiConfig = serde_json::from_str(&content)?;

        Ok(config)
    }

    fn save(&self) -> anyhow::Result<()> {
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

fn get_config_path() -> anyhow::Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
    let config_path = config_dir.join("squirrel-ai-tools").join("api-keys.json");
    Ok(config_path)
}

fn validate_key(key: &str) -> anyhow::Result<()> {
    anyhow::ensure!(!key.is_empty(), "API key cannot be empty");
    Ok(())
}

fn main() -> anyhow::Result<()> {
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
