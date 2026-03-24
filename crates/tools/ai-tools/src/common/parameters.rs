// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Model parameters for AI chat interfaces
//!
//! This module defines common parameters used to control AI model behavior.

use serde::{Deserialize, Serialize};

/// Model parameters for API requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParameters {
    /// Temperature for response generation
    pub temperature: Option<f32>,
    /// Maximum tokens in response
    pub max_tokens: Option<u32>,
    /// Top-p sampling parameter
    pub top_p: Option<f32>,
    /// Top-k sampling parameter
    pub top_k: Option<f32>,
    /// Frequency penalty
    pub frequency_penalty: Option<f32>,
    /// Presence penalty
    pub presence_penalty: Option<f32>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
    /// Whether to stream the response
    pub stream: Option<bool>,
    /// Tool choice preference
    pub tool_choice: Option<ToolChoice>,
}

/// Tool choice options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolChoice {
    /// Let the model decide whether to use tools.
    Auto,
    /// Do not use any tools.
    None,
    /// Force the model to use a tool.
    Required,
    /// Use a specific tool by name.
    Specific(String),
}

impl ModelParameters {
    /// Create new parameters with default values
    #[must_use]
    pub const fn new() -> Self {
        Self {
            temperature: None,
            top_p: None,
            top_k: None,
            max_tokens: None,
            stop: None,
            frequency_penalty: None,
            presence_penalty: None,
            stream: Some(false),
            tool_choice: None,
        }
    }

    /// Set the temperature
    #[must_use]
    pub const fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the `top_p` value
    #[must_use]
    pub const fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Set the `top_k` value
    #[must_use]
    pub const fn with_top_k(mut self, top_k: f32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Set the maximum tokens
    #[must_use]
    pub const fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Add a stop sequence
    #[must_use]
    pub fn with_stop(mut self, stop: impl Into<String>) -> Self {
        if let Some(ref mut stops) = self.stop {
            stops.push(stop.into());
        } else {
            self.stop = Some(vec![stop.into()]);
        }
        self
    }

    /// Add multiple stop sequences
    #[must_use]
    pub fn with_stops(mut self, stops: Vec<String>) -> Self {
        self.stop = Some(stops);
        self
    }

    /// Set the frequency penalty
    #[must_use]
    pub const fn with_frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.frequency_penalty = Some(frequency_penalty);
        self
    }

    /// Set the presence penalty
    #[must_use]
    pub const fn with_presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.presence_penalty = Some(presence_penalty);
        self
    }

    /// Set streaming mode
    #[must_use]
    pub const fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Set the tool choice
    #[must_use]
    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_parameters_new() {
        let params = ModelParameters::new();
        assert!(params.temperature.is_none());
        assert!(params.max_tokens.is_none());
        assert!(params.top_p.is_none());
        assert!(params.top_k.is_none());
        assert!(params.frequency_penalty.is_none());
        assert!(params.presence_penalty.is_none());
        assert!(params.stop.is_none());
        assert_eq!(params.stream, Some(false));
        assert!(params.tool_choice.is_none());
    }

    #[test]
    fn test_model_parameters_default() {
        let params = ModelParameters::default();
        assert_eq!(params.stream, Some(false));
    }

    #[test]
    fn test_model_parameters_builders() {
        let params = ModelParameters::new()
            .with_temperature(0.7)
            .with_top_p(0.9)
            .with_top_k(40.0)
            .with_max_tokens(1000)
            .with_frequency_penalty(0.5)
            .with_presence_penalty(0.5)
            .with_stream(true)
            .with_tool_choice(ToolChoice::Auto);
        assert!((params.temperature.unwrap() - 0.7).abs() < f32::EPSILON);
        assert!((params.top_p.unwrap() - 0.9).abs() < f32::EPSILON);
        assert!((params.top_k.unwrap() - 40.0).abs() < f32::EPSILON);
        assert_eq!(params.max_tokens, Some(1000));
        assert_eq!(params.stream, Some(true));
    }

    #[test]
    fn test_model_parameters_with_stop() {
        let params = ModelParameters::new().with_stop("STOP").with_stop("END");
        let stops = params.stop.unwrap();
        assert_eq!(stops.len(), 2);
        assert_eq!(stops[0], "STOP");
        assert_eq!(stops[1], "END");
    }

    #[test]
    fn test_model_parameters_with_stops() {
        let params = ModelParameters::new().with_stops(vec!["A".to_string(), "B".to_string()]);
        let stops = params.stop.unwrap();
        assert_eq!(stops.len(), 2);
    }

    #[test]
    fn test_model_parameters_serde() {
        let params = ModelParameters::new()
            .with_temperature(0.7)
            .with_max_tokens(512);
        let json = serde_json::to_string(&params).expect("serialize");
        let deser: ModelParameters = serde_json::from_str(&json).expect("deserialize");
        assert!((deser.temperature.unwrap() - 0.7).abs() < f32::EPSILON);
        assert_eq!(deser.max_tokens, Some(512));
    }

    #[test]
    fn test_tool_choice_serde() {
        for tc in [
            ToolChoice::Auto,
            ToolChoice::None,
            ToolChoice::Required,
            ToolChoice::Specific("my_tool".to_string()),
        ] {
            let json = serde_json::to_string(&tc).expect("serialize");
            let _deser: ToolChoice = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_response_format_serde() {
        for fmt in [ResponseFormat::Text, ResponseFormat::Json] {
            let json = serde_json::to_string(&fmt).expect("serialize");
            let deser: ResponseFormat = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(deser, fmt);
        }
    }
}
