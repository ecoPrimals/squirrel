//! AI Tools for the Squirrel system
//!
//! This crate provides integrations with various AI services like OpenAI, Anthropic, and Google Gemini.
//! It handles authentication, rate limiting, prompt management, and response processing.

#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]

pub mod common;
pub mod error;
pub mod openai;
pub mod anthropic;
pub mod gemini;
pub mod config;

pub use error::Error;

/// Common result type for AI operations
pub type Result<T> = std::result::Result<T, Error>;

/// Re-export of key traits and structures that define the AI client interface
pub mod prelude {
    pub use crate::common::{AIClient, ChatMessage, ChatRequest, ChatResponse, ModelParameters, ToolCall};
    pub use crate::error::Error;
    pub use crate::Result;
}

/// Version of the AI tools crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Factory functions for creating AI clients
pub mod clients {
    use std::sync::Arc;
    
    use crate::common::AIClient;
    use crate::openai::OpenAIClient;
    
    /// Create a new OpenAI client
    #[cfg(feature = "openai")]
    pub fn openai(api_key: impl Into<String>) -> Arc<dyn AIClient> {
        Arc::new(OpenAIClient::new(api_key))
    }
    
    /// Create a new Anthropic client
    #[cfg(feature = "anthropic")]
    pub fn anthropic(api_key: impl Into<String>) -> Arc<dyn AIClient> {
        Arc::new(crate::anthropic::AnthropicClient::new(api_key))
    }
    
    /// Create a new Google Gemini client
    #[cfg(feature = "gemini")]
    pub fn gemini(api_key: impl Into<String>) -> Arc<dyn AIClient> {
        Arc::new(crate::gemini::GeminiClient::new(api_key))
    }
} 