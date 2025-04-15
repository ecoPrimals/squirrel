use std::fmt;

/// Supported AI model types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiModel {
    /// GPT-3.5 Turbo
    Gpt35Turbo,
    /// GPT-4
    Gpt4,
    /// GPT-4 Turbo
    Gpt4Turbo,
    /// GPT-4 Vision
    Gpt4Vision,
}

impl AiModel {
    /// Get the model name as used by the API
    pub fn to_api_name(&self) -> &'static str {
        match self {
            AiModel::Gpt35Turbo => "gpt-3.5-turbo",
            AiModel::Gpt4 => "gpt-4",
            AiModel::Gpt4Turbo => "gpt-4-turbo-preview",
            AiModel::Gpt4Vision => "gpt-4-vision-preview",
        }
    }
    
    /// Get a user-friendly display name for the model
    pub fn display_name(&self) -> &'static str {
        match self {
            AiModel::Gpt35Turbo => "GPT-3.5 Turbo",
            AiModel::Gpt4 => "GPT-4",
            AiModel::Gpt4Turbo => "GPT-4 Turbo",
            AiModel::Gpt4Vision => "GPT-4 Vision",
        }
    }
    
    /// Get all available models
    pub fn all() -> Vec<Self> {
        vec![
            Self::Gpt35Turbo,
            Self::Gpt4,
            Self::Gpt4Turbo,
            Self::Gpt4Vision,
        ]
    }
}

impl fmt::Display for AiModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
} 