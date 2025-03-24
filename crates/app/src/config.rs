//! Configuration module for the Squirrel application
//! 
//! This module provides configuration functionality for the application.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application name
    pub name: String,
    
    /// Application version
    pub version: String,
    
    /// Data directory
    pub data_dir: PathBuf,
    
    /// Plugin directory
    pub plugin_dir: PathBuf,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "Squirrel".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            data_dir: PathBuf::from("./data"),
            plugin_dir: PathBuf::from("./plugins"),
        }
    }
}

/// Load configuration from file
pub fn load_config(path: &str) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config_str = std::fs::read_to_string(path)?;
    let config = serde_json::from_str(&config_str)?;
    Ok(config)
}

/// Save configuration to file
pub fn save_config(config: &AppConfig, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config_str = serde_json::to_string_pretty(config)?;
    std::fs::write(path, config_str)?;
    Ok(())
} 
