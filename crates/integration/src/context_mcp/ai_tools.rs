//! AI tools for Context-MCP integration
//!
//! This module provides types and utilities for AI integration with Context-MCP.

use serde::{Deserialize, Serialize};

/// Types of context enhancements that can be performed with AI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextEnhancementType {
    /// Generate summarization of context content
    Summarize,
    
    /// Generate key insights from context
    Insights,
    
    /// Generate recommendations based on context
    Recommendations,
    
    /// Generate classifications or tags for the context
    Classify,
    
    /// Custom enhancement with specific prompt
    Custom(String),
}

impl std::fmt::Display for ContextEnhancementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Summarize => write!(f, "Summarize"),
            Self::Insights => write!(f, "Insights"),
            Self::Recommendations => write!(f, "Recommendations"),
            Self::Classify => write!(f, "Classify"),
            Self::Custom(prompt) => write!(f, "Custom({})", prompt),
        }
    }
}

/// Options for AI context enhancement
#[derive(Debug, Clone)]
pub struct ContextAiEnhancementOptions {
    /// Type of enhancement to perform
    pub enhancement_type: ContextEnhancementType,
    
    /// AI provider to use (openai, anthropic, gemini)
    pub provider: String,
    
    /// API key for the AI provider
    pub api_key: String,
    
    /// Model to use (optional, defaults to an appropriate model for the provider)
    pub model: Option<String>,
    
    /// Timeout in milliseconds
    pub timeout_ms: Option<u64>,
    
    /// Custom system prompt (overrides the default for the enhancement type)
    pub system_prompt: Option<String>,
    
    /// Additional parameters
    pub parameters: serde_json::Map<String, serde_json::Value>,
}

impl ContextAiEnhancementOptions {
    /// Create new options with the given enhancement type, provider and API key
    pub fn new<S1, S2>(
        enhancement_type: ContextEnhancementType,
        provider: S1,
        api_key: S2,
    ) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            enhancement_type,
            provider: provider.into(),
            api_key: api_key.into(),
            model: None,
            timeout_ms: None,
            system_prompt: None,
            parameters: serde_json::Map::new(),
        }
    }
    
    /// Set the model to use
    pub fn with_model<S: Into<String>>(mut self, model: S) -> Self {
        self.model = Some(model.into());
        self
    }
    
    /// Set the timeout in milliseconds
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
    
    /// Set a custom system prompt
    pub fn with_system_prompt<S: Into<String>>(mut self, prompt: S) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }
    
    /// Add a parameter
    pub fn with_parameter<S: Into<String>>(mut self, key: S, value: serde_json::Value) -> Self {
        self.parameters.insert(key.into(), value);
        self
    }
} 