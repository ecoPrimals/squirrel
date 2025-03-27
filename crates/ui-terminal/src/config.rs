use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;
use directories::ProjectDirs;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use thiserror::Error;

/// Configuration error
#[derive(Debug, Error)]
pub enum ConfigError {
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] toml::ser::Error),
    
    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] toml::de::Error),
    
    /// Invalid configuration data
    #[error("Invalid configuration data: {0}")]
    InvalidConfig(String),
    
    /// No configuration directory available
    #[error("No configuration directory available")]
    NoConfigDir,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Data update interval in milliseconds
    #[serde(with = "duration_ms")]
    pub update_interval: Duration,
    
    /// Maximum number of history points to keep
    pub max_history_points: usize,
    
    /// Use real MCP client instead of mock
    pub use_real_mcp: bool,
    
    /// Simulate connection errors for testing
    pub simulate_errors: bool,
    
    /// UI theme
    pub theme: String,
    
    /// UI settings
    pub ui: UiConfig,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Show help by default
    pub show_help: bool,
    
    /// Default tab index
    pub default_tab: usize,
    
    /// UI refresh rate in milliseconds
    pub refresh_rate_ms: u64,
    
    /// Color scheme
    pub colors: ColorScheme,
}

/// Color scheme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    /// Primary color (hex)
    pub primary: String,
    
    /// Secondary color (hex)
    pub secondary: String,
    
    /// Background color (hex)
    pub background: String,
    
    /// Foreground color (hex)
    pub foreground: String,
    
    /// Success color (hex)
    pub success: String,
    
    /// Error color (hex)
    pub error: String,
    
    /// Warning color (hex)
    pub warning: String,
    
    /// Info color (hex)
    pub info: String,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            update_interval: Duration::from_secs(2),
            max_history_points: 100,
            use_real_mcp: false,
            simulate_errors: false,
            theme: "dark".to_string(),
            ui: UiConfig::default(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_help: true,
            default_tab: 0,
            refresh_rate_ms: 250,
            colors: ColorScheme::default(),
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            primary: "#4299e1".to_string(),   // Blue
            secondary: "#805ad5".to_string(), // Purple
            background: "#1a202c".to_string(), // Dark gray
            foreground: "#e2e8f0".to_string(), // Light gray
            success: "#68d391".to_string(),   // Green
            error: "#fc8181".to_string(),     // Red
            warning: "#f6e05e".to_string(),   // Yellow
            info: "#63b3ed".to_string(),      // Light blue
        }
    }
}

impl DashboardConfig {
    /// Get the config file path
    pub fn get_config_path() -> Result<PathBuf, ConfigError> {
        let proj_dirs = ProjectDirs::from("com", "datasciencebiolab", "mcp-dashboard")
            .ok_or(ConfigError::NoConfigDir)?;
        
        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir)?;
        
        Ok(config_dir.join("config.toml"))
    }
    
    /// Load configuration from file
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = Self::get_config_path()?;
        
        if !config_path.exists() {
            return Ok(Self::default());
        }
        
        let config_str = fs::read_to_string(config_path)?;
        let config = toml::from_str(&config_str)?;
        
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<(), ConfigError> {
        let config_path = Self::get_config_path()?;
        let config_str = toml::to_string_pretty(self)?;
        
        fs::write(config_path, config_str)?;
        
        Ok(())
    }
}

/// Helper module for serializing/deserializing Duration
mod duration_ms {
    use std::time::Duration;
    use serde::{Deserialize, Deserializer, Serializer};
    
    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let millis = duration.as_millis() as u64;
        serializer.serialize_u64(millis)
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
} 