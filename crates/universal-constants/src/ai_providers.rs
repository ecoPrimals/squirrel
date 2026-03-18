// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! AI Provider URL Configuration - Infant Primal Pattern
//!
//! **Philosophy**: Zero hardcoded knowledge. All AI provider URLs discovered at runtime.
//!
//! Following the infant primal pattern (see `network` module):
//! 1. Try environment variable first (e.g., `OPENAI_API_BASE_URL`)
//! 2. Fall back to default constant (with warning)
//!
//! # Example
//!
//! ```rust,ignore
//! // ✅ GOOD: Discovery-based
//! let base = universal_constants::ai_providers::openai_base_url();
//!
//! // Override via env:
//! // export OPENAI_API_BASE_URL="https://custom.openai.proxy/v1"
//! ```

/// Default Anthropic API base URL (fallback when env not set)
pub const DEFAULT_ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com/v1";

/// Default `OpenAI` API base URL (fallback when env not set)
pub const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";

/// Anthropic Messages API path (appended to base URL)
pub const ANTHROPIC_MESSAGES_PATH: &str = "/messages";

/// `OpenAI` Chat Completions API path (appended to base URL)
pub const OPENAI_CHAT_COMPLETIONS_PATH: &str = "/chat/completions";

/// Get Anthropic API base URL (infant primal pattern)
///
/// Discovery order:
/// 1. `ANTHROPIC_API_BASE_URL` environment variable
/// 2. Fallback to default (with warning)
#[must_use]
pub fn anthropic_base_url() -> String {
    std::env::var("ANTHROPIC_API_BASE_URL").unwrap_or_else(|_| {
        tracing::warn!(
            "Using fallback Anthropic API URL: {} - set ANTHROPIC_API_BASE_URL for production",
            DEFAULT_ANTHROPIC_BASE_URL
        );
        DEFAULT_ANTHROPIC_BASE_URL.to_string()
    })
}

/// Get `OpenAI` API base URL (infant primal pattern)
///
/// Discovery order:
/// 1. `OPENAI_API_BASE_URL` environment variable
/// 2. Fallback to default (with warning)
#[must_use]
pub fn openai_base_url() -> String {
    std::env::var("OPENAI_API_BASE_URL").unwrap_or_else(|_| {
        tracing::warn!(
            "Using fallback OpenAI API URL: {} - set OPENAI_API_BASE_URL for production",
            DEFAULT_OPENAI_BASE_URL
        );
        DEFAULT_OPENAI_BASE_URL.to_string()
    })
}

/// Get Anthropic Messages API URL (base + /messages)
#[must_use]
pub fn anthropic_messages_url() -> String {
    let base = anthropic_base_url();
    let base = base.trim_end_matches('/');
    format!("{base}{ANTHROPIC_MESSAGES_PATH}")
}

/// Get `OpenAI` Chat Completions API URL (base + `/chat/completions`)
#[must_use]
pub fn openai_chat_completions_url() -> String {
    let base = openai_base_url();
    let base = base.trim_end_matches('/');
    format!("{base}{OPENAI_CHAT_COMPLETIONS_PATH}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_constants() {
        assert_eq!(DEFAULT_ANTHROPIC_BASE_URL, "https://api.anthropic.com/v1");
        assert_eq!(DEFAULT_OPENAI_BASE_URL, "https://api.openai.com/v1");
    }

    #[test]
    fn test_anthropic_base_url_env_override() {
        temp_env::with_var(
            "ANTHROPIC_API_BASE_URL",
            Some("https://custom.anthropic/v1"),
            || {
                assert_eq!(anthropic_base_url(), "https://custom.anthropic/v1");
            },
        );
    }

    #[test]
    fn test_openai_base_url_env_override() {
        temp_env::with_var(
            "OPENAI_API_BASE_URL",
            Some("https://custom.openai/v1"),
            || {
                assert_eq!(openai_base_url(), "https://custom.openai/v1");
            },
        );
    }

    #[test]
    fn test_anthropic_messages_url() {
        temp_env::with_var_unset("ANTHROPIC_API_BASE_URL", || {
            assert_eq!(
                anthropic_messages_url(),
                "https://api.anthropic.com/v1/messages"
            );
        });
    }

    #[test]
    fn test_openai_chat_completions_url() {
        temp_env::with_var_unset("OPENAI_API_BASE_URL", || {
            assert_eq!(
                openai_chat_completions_url(),
                "https://api.openai.com/v1/chat/completions"
            );
        });
    }
}
