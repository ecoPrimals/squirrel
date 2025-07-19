//! Usage information for AI service responses
//!
//! This module defines types for tracking token usage and costs.

use serde::{Deserialize, Serialize};

/// Usage information for an AI request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    /// Number of tokens used for the prompt
    pub prompt_tokens: u32,

    /// Number of tokens generated in the response
    pub completion_tokens: u32,

    /// Total number of tokens used
    pub total_tokens: u32,

    /// Estimated cost in USD (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_cost_usd: Option<f64>,
}

impl UsageInfo {
    /// Create new usage info
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            estimated_cost_usd: None,
        }
    }

    /// Set the estimated cost
    pub fn with_cost(mut self, cost: f64) -> Self {
        self.estimated_cost_usd = Some(cost);
        self
    }

    /// Combine with another usage info
    pub fn combine(&self, other: &UsageInfo) -> Self {
        let estimated_cost_usd = match (self.estimated_cost_usd, other.estimated_cost_usd) {
            (Some(a), Some(b)) => Some(a + b),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };

        Self {
            prompt_tokens: self.prompt_tokens + other.prompt_tokens,
            completion_tokens: self.completion_tokens + other.completion_tokens,
            total_tokens: self.total_tokens + other.total_tokens,
            estimated_cost_usd,
        }
    }
}

/// A token counter for estimating token usage before making API calls
#[derive(Debug, Clone)]
pub struct TokenCounter {
    // Simple token counter without specific tokenizer
}

impl TokenCounter {
    /// Create a new token counter
    pub fn new(_tokenizer_name: impl Into<String>) -> Self {
        Self {}
    }

    /// Count tokens in a string
    pub fn count_tokens(&self, text: &str) -> u32 {
        // This is a very simple approximation
        // In a real implementation, we would use a proper tokenizer like tiktoken
        let words = text.split_whitespace().count();
        (words as f32 * 1.3) as u32
    }
}
