//! OpenAI models information
//!
//! This module provides information about different OpenAI models.

/// Describes an OpenAI model
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// The model ID
    pub id: String,
    /// Max context length
    pub max_context: usize,
    /// Max tokens per request
    pub max_tokens: usize,
    /// Whether the model supports function calling
    pub supports_functions: bool,
    /// Whether the model supports JSON mode
    pub supports_json: bool,
}

/// Information about GPT-4 and GPT-4 Turbo models
pub mod gpt4 {
    use super::ModelInfo;
    
    /// GPT-4 model
    pub const GPT4: ModelInfo = ModelInfo {
        id: "gpt-4",
        max_context: 8_192,
        max_tokens: 4_096,
        supports_functions: true,
        supports_json: true,
    };
    
    /// GPT-4 Turbo
    pub const GPT4_TURBO: ModelInfo = ModelInfo {
        id: "gpt-4-turbo-preview",
        max_context: 128_000,
        max_tokens: 4_096,
        supports_functions: true,
        supports_json: true,
    };
    
    /// GPT-4 Vision
    pub const GPT4_VISION: ModelInfo = ModelInfo {
        id: "gpt-4-vision-preview",
        max_context: 128_000,
        max_tokens: 4_096,
        supports_functions: true,
        supports_json: true,
    };
}

/// Information about GPT-3.5 models
pub mod gpt3 {
    use super::ModelInfo;
    
    /// GPT-3.5 Turbo
    pub const GPT3_TURBO: ModelInfo = ModelInfo {
        id: "gpt-3.5-turbo",
        max_context: 16_385,
        max_tokens: 4_096,
        supports_functions: true,
        supports_json: true,
    };
    
    /// GPT-3.5 Turbo 16k
    pub const GPT3_TURBO_16K: ModelInfo = ModelInfo {
        id: "gpt-3.5-turbo-16k",
        max_context: 16_385,
        max_tokens: 4_096,
        supports_functions: true,
        supports_json: true,
    };
}

/// Default model to use when no model is specified
pub const DEFAULT_MODEL: &str = "gpt-3.5-turbo";

/// Get information about a model by ID
pub fn get_model_info(model_id: &str) -> Option<ModelInfo> {
    match model_id {
        "gpt-4" => Some(gpt4::GPT4.clone()),
        "gpt-4-turbo-preview" => Some(gpt4::GPT4_TURBO.clone()),
        "gpt-4-vision-preview" => Some(gpt4::GPT4_VISION.clone()),
        "gpt-3.5-turbo" => Some(gpt3::GPT3_TURBO.clone()),
        "gpt-3.5-turbo-16k" => Some(gpt3::GPT3_TURBO_16K.clone()),
        _ => None,
    }
} 