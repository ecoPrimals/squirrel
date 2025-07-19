//! Configuration stub for Squirrel main
//!
//! This is a minimal configuration module to satisfy imports

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub debug: bool,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            debug: false,
            log_level: "info".to_string(),
        }
    }
}

pub fn load_config() -> Result<Config, crate::error::PrimalError> {
    Ok(Config::default())
}
