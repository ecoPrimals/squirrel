//! OpenAI models information
//!
//! This module provides information about different OpenAI models.

/// Base trait for OpenAI models
pub trait OpenAIModel {
    /// Returns the model's unique identifier.
    fn id(&self) -> &str;
    /// Returns the human-readable name of the model.
    fn name(&self) -> &str;
    /// Returns the maximum context window size in tokens.
    fn context_window(&self) -> usize;
    /// Returns the cost per 1000 input tokens in USD.
    fn cost_per_1k_input_tokens(&self) -> f64;
    /// Returns the cost per 1000 output tokens in USD.
    fn cost_per_1k_output_tokens(&self) -> f64;
}

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

    /// Model ID for GPT-4.
    pub static GPT4_ID: &str = "gpt-4";
    /// Model ID for GPT-4 Turbo Preview.
    pub static GPT4_TURBO_ID: &str = "gpt-4-turbo-preview";
    /// Model ID for GPT-4 Vision Preview.
    pub static GPT4_VISION_ID: &str = "gpt-4-vision-preview";
    /// Model ID for GPT-3.5 Turbo.
    pub static GPT3_TURBO_ID: &str = "gpt-3.5-turbo";
    /// Model ID for GPT-3.5 Turbo 16K.
    pub static GPT3_TURBO_16K_ID: &str = "gpt-3.5-turbo-16k";

    /// GPT-4 model
    pub fn gpt4() -> ModelInfo {
        ModelInfo {
            id: "gpt-4".to_string(),
            max_context: 8192,
            max_tokens: 8192,
            supports_functions: true,
            supports_json: true,
        }
    }

    /// GPT-4 Turbo
    pub fn gpt4_turbo() -> ModelInfo {
        ModelInfo {
            id: "gpt-4-turbo-preview".to_string(),
            max_context: 128000,
            max_tokens: 4096,
            supports_functions: true,
            supports_json: true,
        }
    }

    /// GPT-4 Vision
    pub fn gpt4_vision() -> ModelInfo {
        ModelInfo {
            id: "gpt-4-vision-preview".to_string(),
            max_context: 128000,
            max_tokens: 4096,
            supports_functions: true,
            supports_json: true,
        }
    }
}

/// Information about GPT-3.5 models
pub mod gpt3 {
    use super::ModelInfo;

    /// GPT-3.5 Turbo
    pub fn gpt3_turbo() -> ModelInfo {
        ModelInfo {
            id: "gpt-3.5-turbo".to_string(),
            max_context: 4096,
            max_tokens: 4096,
            supports_functions: true,
            supports_json: true,
        }
    }

    /// GPT-3.5 Turbo 16k
    pub fn gpt3_turbo_16k() -> ModelInfo {
        ModelInfo {
            id: "gpt-3.5-turbo-16k".to_string(),
            max_context: 16384,
            max_tokens: 16384,
            supports_functions: true,
            supports_json: true,
        }
    }
}

/// Default model to use when no model is specified
pub const DEFAULT_MODEL: &str = "gpt-3.5-turbo";

/// Get information about a model by ID
pub fn get_model_info(model_id: &str) -> Option<ModelInfo> {
    match model_id {
        "gpt-4" => Some(gpt4::gpt4()),
        "gpt-4-turbo-preview" => Some(gpt4::gpt4_turbo()),
        "gpt-4-vision-preview" => Some(gpt4::gpt4_vision()),
        "gpt-3.5-turbo" => Some(gpt3::gpt3_turbo()),
        "gpt-3.5-turbo-16k" => Some(gpt3::gpt3_turbo_16k()),
        _ => None,
    }
}

/// GPT-4 model
#[derive(Debug, Clone)]
pub struct GPT4 {
    id: String,
    name: String,
    context_window: usize,
    cost_per_1k_input_tokens: f64,
    cost_per_1k_output_tokens: f64,
}

/// GPT-4 Turbo model
#[derive(Debug, Clone)]
pub struct GPT4Turbo {
    id: String,
    name: String,
    context_window: usize,
    cost_per_1k_input_tokens: f64,
    cost_per_1k_output_tokens: f64,
}

/// GPT-4 Vision model
#[derive(Debug, Clone)]
pub struct GPT4Vision {
    id: String,
    name: String,
    context_window: usize,
    cost_per_1k_input_tokens: f64,
    cost_per_1k_output_tokens: f64,
}

/// GPT-3.5 Turbo model
#[derive(Debug, Clone)]
pub struct GPT35Turbo {
    id: String,
    name: String,
    context_window: usize,
    cost_per_1k_input_tokens: f64,
    cost_per_1k_output_tokens: f64,
}

/// GPT-3.5 Turbo 16K model
#[derive(Debug, Clone)]
pub struct GPT35Turbo16K {
    id: String,
    name: String,
    context_window: usize,
    cost_per_1k_input_tokens: f64,
    cost_per_1k_output_tokens: f64,
}

impl Default for GPT4 {
    fn default() -> Self {
        Self::new()
    }
}

impl GPT4 {
    /// Creates a new instance representing the GPT-4 model.
    pub fn new() -> Self {
        Self {
            id: String::from("gpt-4"),
            name: String::from("GPT-4"),
            context_window: 8192,
            cost_per_1k_input_tokens: 0.03,
            cost_per_1k_output_tokens: 0.06,
        }
    }
}

impl Default for GPT4Turbo {
    fn default() -> Self {
        Self::new()
    }
}

impl GPT4Turbo {
    /// Creates a new instance representing the GPT-4 Turbo model.
    pub fn new() -> Self {
        Self {
            id: String::from("gpt-4-turbo-preview"),
            name: String::from("GPT-4 Turbo"),
            context_window: 128000,
            cost_per_1k_input_tokens: 0.01,
            cost_per_1k_output_tokens: 0.03,
        }
    }
}

impl Default for GPT4Vision {
    fn default() -> Self {
        Self::new()
    }
}

impl GPT4Vision {
    /// Creates a new instance representing the GPT-4 Vision model.
    pub fn new() -> Self {
        Self {
            id: String::from("gpt-4-vision-preview"),
            name: String::from("GPT-4 Vision"),
            context_window: 128000,
            cost_per_1k_input_tokens: 0.01,
            cost_per_1k_output_tokens: 0.03,
        }
    }
}

impl Default for GPT35Turbo {
    fn default() -> Self {
        Self::new()
    }
}

impl GPT35Turbo {
    /// Creates a new instance representing the GPT-3.5 Turbo model.
    pub fn new() -> Self {
        Self {
            id: String::from("gpt-3.5-turbo"),
            name: String::from("GPT-3.5 Turbo"),
            context_window: 4096,
            cost_per_1k_input_tokens: 0.0015,
            cost_per_1k_output_tokens: 0.002,
        }
    }
}

impl Default for GPT35Turbo16K {
    fn default() -> Self {
        Self::new()
    }
}

impl GPT35Turbo16K {
    /// Creates a new instance representing the GPT-3.5 Turbo 16K model.
    pub fn new() -> Self {
        Self {
            id: String::from("gpt-3.5-turbo-16k"),
            name: String::from("GPT-3.5 Turbo 16K"),
            context_window: 16384,
            cost_per_1k_input_tokens: 0.003,
            cost_per_1k_output_tokens: 0.004,
        }
    }
}

impl OpenAIModel for GPT4 {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn context_window(&self) -> usize {
        self.context_window
    }
    fn cost_per_1k_input_tokens(&self) -> f64 {
        self.cost_per_1k_input_tokens
    }
    fn cost_per_1k_output_tokens(&self) -> f64 {
        self.cost_per_1k_output_tokens
    }
}

impl OpenAIModel for GPT4Turbo {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn context_window(&self) -> usize {
        self.context_window
    }
    fn cost_per_1k_input_tokens(&self) -> f64 {
        self.cost_per_1k_input_tokens
    }
    fn cost_per_1k_output_tokens(&self) -> f64 {
        self.cost_per_1k_output_tokens
    }
}

impl OpenAIModel for GPT4Vision {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn context_window(&self) -> usize {
        self.context_window
    }
    fn cost_per_1k_input_tokens(&self) -> f64 {
        self.cost_per_1k_input_tokens
    }
    fn cost_per_1k_output_tokens(&self) -> f64 {
        self.cost_per_1k_output_tokens
    }
}

impl OpenAIModel for GPT35Turbo {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn context_window(&self) -> usize {
        self.context_window
    }
    fn cost_per_1k_input_tokens(&self) -> f64 {
        self.cost_per_1k_input_tokens
    }
    fn cost_per_1k_output_tokens(&self) -> f64 {
        self.cost_per_1k_output_tokens
    }
}

impl OpenAIModel for GPT35Turbo16K {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn context_window(&self) -> usize {
        self.context_window
    }
    fn cost_per_1k_input_tokens(&self) -> f64 {
        self.cost_per_1k_input_tokens
    }
    fn cost_per_1k_output_tokens(&self) -> f64 {
        self.cost_per_1k_output_tokens
    }
}
