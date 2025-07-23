//! Configuration for local AI models

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the local AI client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAIConfig {
    /// Default model to use
    pub default_model: String,

    /// Whether to enable Ollama integration
    pub enable_ollama: bool,

    /// Whether to enable native Rust implementations
    pub enable_native: bool,

    /// Ollama-specific configuration
    pub ollama: OllamaConfig,

    /// Native implementation configuration
    pub native: NativeConfig,

    /// Resource management settings
    pub resource_management: ResourceManagementConfig,
}

/// Configuration for Ollama integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// Ollama server URL
    pub base_url: String,

    /// Connection timeout in seconds
    pub timeout_seconds: u64,

    /// Models to auto-discover
    pub auto_discover_models: bool,

    /// Specific models to load
    pub models: Vec<String>,

    /// Keep models loaded in memory
    pub keep_alive: bool,
}

/// Configuration for native model implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeConfig {
    /// Directory containing model files
    pub models_directory: PathBuf,

    /// Maximum number of models to keep loaded
    pub max_loaded_models: usize,

    /// Whether to use GPU acceleration
    pub use_gpu: bool,

    /// GPU device IDs to use (empty means use all available)
    pub gpu_device_ids: Vec<u32>,

    /// Thread count for CPU inference
    pub cpu_threads: Option<usize>,
}

/// Resource management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManagementConfig {
    /// Maximum memory usage in MB
    pub max_memory_mb: u64,

    /// Maximum GPU memory usage in MB
    pub max_gpu_memory_mb: Option<u64>,

    /// Whether to automatically unload unused models
    pub auto_unload_unused: bool,

    /// Time in seconds before unloading unused models
    pub unload_timeout_seconds: u64,

    /// Whether to preload frequently used models
    pub preload_frequent_models: bool,
}

impl Default for LocalAIConfig {
    fn default() -> Self {
        Self {
            default_model: "llama3-8b".to_string(),
            enable_ollama: true,
            enable_native: false, // Disabled by default until implementations are ready
            ollama: OllamaConfig::default(),
            native: NativeConfig::default(),
            resource_management: ResourceManagementConfig::default(),
        }
    }
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: std::env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| crate::config::DefaultEndpoints::ollama_endpoint()),
            timeout_seconds: 30,
            auto_discover_models: true,
            models: vec![
                "llama3-8b".to_string(),
                "llama3-70b".to_string(),
                "codellama".to_string(),
                "mistral".to_string(),
            ],
            keep_alive: true,
        }
    }
}

impl Default for NativeConfig {
    fn default() -> Self {
        Self {
            models_directory: PathBuf::from("./models"),
            max_loaded_models: 3,
            use_gpu: true,
            gpu_device_ids: vec![], // Use all available GPUs
            cpu_threads: None,      // Use system default
        }
    }
}

impl Default for ResourceManagementConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 16384,    // 16GB default
            max_gpu_memory_mb: None, // No limit by default
            auto_unload_unused: true,
            unload_timeout_seconds: 300, // 5 minutes
            preload_frequent_models: false,
        }
    }
}
