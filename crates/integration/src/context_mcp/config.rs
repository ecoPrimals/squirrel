//! Context-MCP Adapter Configuration
//!
//! This module provides configuration options for the Context-MCP adapter.

use serde::{Deserialize, Serialize};

/// Direction of synchronization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncDirection {
    /// Sync from Squirrel to MCP
    SquirrelToMcp,
    /// Sync from MCP to Squirrel
    McpToSquirrel,
    /// Sync in both directions
    Bidirectional,
}

/// Configuration for the Context-MCP adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMcpAdapterConfig {
    /// How often to sync in seconds
    pub sync_interval_secs: u64,
    /// Maximum number of retries for operations
    pub max_retries: u32,
    /// API request timeout in milliseconds
    pub timeout_ms: u64,
    /// Whether to enable automatic sync
    pub enable_auto_sync: bool,
    /// Direction of synchronization
    pub sync_direction: SyncDirection,
}

impl Default for ContextMcpAdapterConfig {
    fn default() -> Self {
        Self {
            sync_interval_secs: 60,
            max_retries: 3,
            timeout_ms: 5000,
            enable_auto_sync: true,
            sync_direction: SyncDirection::Bidirectional,
        }
    }
}

/// Type of enhancement to apply to context
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextEnhancementType {
    /// Provide insights about the context data
    Insights,
    /// Summarize the context data
    Summary,
    /// Similar to Summary, added for backward compatibility
    Summarize,
    /// Provide recommendations based on the context data
    Recommendations,
    /// Analyze trends in the context data
    TrendAnalysis,
    /// Detect anomalies in the context data
    AnomalyDetection,
    /// Custom enhancement with instructions
    Custom(String),
}

impl std::fmt::Display for ContextEnhancementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Insights => write!(f, "Insights"),
            Self::Summary => write!(f, "Summary"),
            Self::Summarize => write!(f, "Summarize"),
            Self::Recommendations => write!(f, "Recommendations"),
            Self::TrendAnalysis => write!(f, "TrendAnalysis"),
            Self::AnomalyDetection => write!(f, "AnomalyDetection"),
            Self::Custom(instruction) => write!(f, "Custom: {}", instruction),
        }
    }
}

/// Options for context enhancement operations
#[derive(Debug, Clone)]
pub struct ContextEnhancementOptions {
    /// Type of enhancement to apply
    pub enhancement_type: ContextEnhancementType,
    /// Optional custom template to use
    pub custom_template: Option<String>,
    /// Optional timeout in milliseconds
    pub timeout_ms: Option<u64>,
}

/// Options for AI enhancement
#[derive(Debug, Clone)]
pub struct ContextAiEnhancementOptions {
    /// Type of enhancement to apply
    pub enhancement_type: ContextEnhancementType,
    /// Optional custom prompt to use
    pub custom_prompt: Option<String>,
    /// Optional maximum tokens to use
    pub max_tokens: Option<usize>,
    /// Optional temperature (0.0-1.0)
    pub temperature: Option<f32>,
    /// AI provider to use (e.g., "openai", "anthropic")
    pub provider: String,
    /// API key for the AI provider
    pub api_key: String,
    /// Model to use
    pub model: Option<String>,
    /// Timeout in milliseconds
    pub timeout_ms: Option<u64>,
    /// Additional parameters as key-value pairs
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
}

impl ContextAiEnhancementOptions {
    /// Create new options with the given enhancement type, provider and API key
    pub fn new(
        enhancement_type: ContextEnhancementType,
        provider: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        let provider_str = provider.into();
        let model = default_model_for_provider(&provider_str);
        
        Self {
            enhancement_type,
            custom_prompt: None,
            max_tokens: None,
            temperature: None,
            provider: provider_str,
            api_key: api_key.into(),
            model,
            timeout_ms: None,
            parameters: std::collections::HashMap::new(),
        }
    }
    
    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
    
    /// Set the timeout in milliseconds
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Set a custom system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.custom_prompt = Some(prompt.into());
        self
    }
    
    /// Add a parameter with a value
    pub fn with_parameter<T: Into<serde_json::Value>>(mut self, key: impl Into<String>, value: T) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
}

/// Determine the default model based on provider
fn default_model_for_provider(provider: &str) -> Option<String> {
    match provider.to_lowercase().as_str() {
        "openai" => Some("gpt-4-turbo".to_string()),
        "anthropic" => Some("claude-3-sonnet".to_string()),
        "gemini" => Some("gemini-pro".to_string()),
        _ => None,
    }
} 