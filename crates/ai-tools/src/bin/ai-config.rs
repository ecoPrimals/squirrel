use clap::{Parser, Subcommand};
use squirrel_ai_tools::config::Config;
use secrecy::ExposeSecret;
use std::error::Error;

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
    },
    /// Show the current configuration status
    Status,
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
        Commands::SetKey { key } => {
            validate_key(&key)?;
            let mut config = Config::load()?;
            config.set_openai_api_key(key);
            config.save()?;
            println!("API key set successfully");
        }
        Commands::Status => {
            let config = Config::load()?;
            let secret = config.openai_api_key.expose_secret();
            if secret.0.is_empty() {
                println!("No API key configured");
            } else {
                println!("API key is configured");
            }
        }
    }

    Ok(())
} 