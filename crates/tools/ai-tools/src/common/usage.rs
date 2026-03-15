// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_info_new() {
        let usage = UsageInfo::new(100, 50);
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
        assert!(usage.estimated_cost_usd.is_none());
    }

    #[test]
    fn test_usage_info_with_cost() {
        let usage = UsageInfo::new(100, 50).with_cost(0.05);
        assert!((usage.estimated_cost_usd.unwrap() - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_usage_info_combine() {
        let a = UsageInfo::new(100, 50).with_cost(0.05);
        let b = UsageInfo::new(200, 100).with_cost(0.10);
        let combined = a.combine(&b);
        assert_eq!(combined.prompt_tokens, 300);
        assert_eq!(combined.completion_tokens, 150);
        assert_eq!(combined.total_tokens, 450); // 150 + 300
        assert!((combined.estimated_cost_usd.unwrap() - 0.15).abs() < f64::EPSILON);
    }

    #[test]
    fn test_usage_info_combine_partial_cost() {
        let a = UsageInfo::new(100, 50).with_cost(0.05);
        let b = UsageInfo::new(200, 100);
        let combined = a.combine(&b);
        assert!((combined.estimated_cost_usd.unwrap() - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_usage_info_combine_no_cost() {
        let a = UsageInfo::new(100, 50);
        let b = UsageInfo::new(200, 100);
        let combined = a.combine(&b);
        assert!(combined.estimated_cost_usd.is_none());
    }

    #[test]
    fn test_usage_info_serde() {
        let usage = UsageInfo::new(100, 50).with_cost(0.05);
        let json = serde_json::to_string(&usage).expect("serialize");
        let deser: UsageInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.prompt_tokens, 100);
        assert_eq!(deser.completion_tokens, 50);
        assert_eq!(deser.total_tokens, 150);
    }

    #[test]
    fn test_usage_info_serde_skip_none_cost() {
        let usage = UsageInfo::new(10, 20);
        let json = serde_json::to_string(&usage).expect("serialize");
        assert!(!json.contains("estimated_cost_usd"));
    }

    #[test]
    fn test_token_counter_new() {
        let counter = TokenCounter::new("gpt-4");
        assert!(counter.count_tokens("hello world") > 0);
    }

    #[test]
    fn test_token_counter_empty_string() {
        let counter = TokenCounter::new("test");
        assert_eq!(counter.count_tokens(""), 0);
    }

    #[test]
    fn test_token_counter_approximation() {
        let counter = TokenCounter::new("test");
        let count = counter.count_tokens("one two three four five");
        // 5 words * 1.3 = 6.5 → 6
        assert_eq!(count, 6);
    }
}
