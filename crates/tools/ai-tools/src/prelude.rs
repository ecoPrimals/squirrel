//! Prelude module for AI tools
//!
//! This module re-exports commonly used types and traits for convenience.

// Common imports for AI tools
pub use crate::common::*;
pub use crate::error::{AIError, Result};

// Remove orchestrator adapter imports - moved to Songbird project
// pub use crate::orchestrator_adapter::{
//     AIToolsOrchestratorAdapter,
//     OrchestratorAIProvider,
//     AIAdapterError,
//     AIProvider,
//     ChatMessage,
// };

// AI provider clients
pub use crate::openai::OpenAIClient;
pub use crate::anthropic::AnthropicClient;
pub use crate::gemini::GeminiClient; 