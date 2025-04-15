//! Model parameters for AI chat interfaces
//!
//! This module defines common parameters used to control AI model behavior.

use serde::{Deserialize, Serialize};

/// Parameters for controlling AI model behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParameters {
    /// Temperature controls randomness (0-2, higher means more random)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    
    /// Top-p controls diversity (0-1, lower means less diverse)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    
    /// Maximum tokens to generate in the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    
    /// Stop sequences to end generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    
    /// Frequency penalty to avoid repetition (0-2, higher means more penalty)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    
    /// Presence penalty to avoid repetition (0-2, higher means more penalty)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    
    /// Response format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    
    /// Whether to stream the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

impl ModelParameters {
    /// Create new parameters with default values
    pub fn new() -> Self {
        Self {
            temperature: None,
            top_p: None,
            max_tokens: None,
            stop: None,
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
            stream: None,
        }
    }
    
    /// Set the temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
    
    /// Set the top_p value
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }
    
    /// Set the maximum tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
    
    /// Add a stop sequence
    pub fn with_stop(mut self, stop: impl Into<String>) -> Self {
        if let Some(ref mut stops) = self.stop {
            stops.push(stop.into());
        } else {
            self.stop = Some(vec![stop.into()]);
        }
        self
    }
    
    /// Add multiple stop sequences
    pub fn with_stops(mut self, stops: Vec<String>) -> Self {
        self.stop = Some(stops);
        self
    }
    
    /// Set the frequency penalty
    pub fn with_frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.frequency_penalty = Some(frequency_penalty);
        self
    }
    
    /// Set the presence penalty
    pub fn with_presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.presence_penalty = Some(presence_penalty);
        self
    }
    
    /// Set the response format
    pub fn with_response_format(mut self, response_format: ResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }
    
    /// Set streaming mode
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }
}

impl Default for ModelParameters {
    fn default() -> Self {
        Self::new()
    }
}

/// Response format for the model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseFormat {
    /// Standard text response
    Text,
    /// JSON response
    Json,
} 