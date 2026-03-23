// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use thiserror::Error;
use uuid;

/// Errors related to MCP context operations
///
/// This enum represents errors that can occur when working with MCP contexts,
/// including context lookup failures, validation errors, and synchronization issues.
#[derive(Debug, Clone, Error)]
pub enum ContextError {
    /// Error that occurs when a context with the specified UUID cannot be found
    ///
    /// This typically happens when trying to access a context that doesn't exist
    /// or has been removed.
    #[error("Context not found: {0}")]
    NotFound(uuid::Uuid),

    /// Error that occurs when context validation fails
    ///
    /// This can happen when a context contains invalid data or doesn't meet
    /// the required constraints.
    #[error("Context validation error: {0}")]
    ValidationError(String),

    /// Error that occurs during context synchronization
    ///
    /// This can happen when there are issues synchronizing context data
    /// between components or systems.
    #[error("Context sync error: {0}")]
    SyncError(String),

    /// General context error with a message
    ///
    /// This is used for errors that don't fit into the other categories.
    #[error("Context error: {0}")]
    General(String),
}

impl From<String> for ContextError {
    fn from(message: String) -> Self {
        Self::General(message)
    }
}

impl From<&str> for ContextError {
    fn from(message: &str) -> Self {
        Self::General(message.to_string())
    }
}

#[cfg(test)]
mod tests {
    // SPDX-License-Identifier: AGPL-3.0-only
    // Inline tests follow the pattern used in `context.rs` and `severity.rs`.

    use super::ContextError;
    use std::fmt::Write as _;
    use uuid::Uuid;

    #[test]
    fn context_error_display_not_found() {
        let id = Uuid::nil();
        let err = ContextError::NotFound(id);
        let s = err.to_string();
        assert!(s.contains("Context not found"));
        assert!(s.contains(&id.to_string()));
    }

    #[test]
    fn context_error_display_validation_error() {
        let err = ContextError::ValidationError("bad".into());
        assert!(err.to_string().contains("validation"));
        assert!(err.to_string().contains("bad"));
    }

    #[test]
    fn context_error_display_sync_error() {
        let err = ContextError::SyncError("sync".into());
        assert!(err.to_string().contains("sync"));
    }

    #[test]
    fn context_error_display_general() {
        let err = ContextError::General("msg".into());
        assert!(err.to_string().contains("Context error"));
        assert!(err.to_string().contains("msg"));
    }

    #[test]
    fn context_error_debug_all_variants() {
        let id = Uuid::nil();
        let cases = [
            ContextError::NotFound(id),
            ContextError::ValidationError("v".into()),
            ContextError::SyncError("s".into()),
            ContextError::General("g".into()),
        ];
        for e in cases {
            let mut buf = String::new();
            write!(&mut buf, "{e:?}").expect("format");
            assert!(!buf.is_empty());
        }
    }

    #[test]
    fn context_error_from_string() {
        let err: ContextError = "hello".to_string().into();
        assert!(matches!(err, ContextError::General(ref s) if s == "hello"));
        assert!(err.to_string().contains("hello"));
    }

    #[test]
    fn context_error_from_str_slice() {
        let err: ContextError = "slice".into();
        assert!(matches!(err, ContextError::General(ref s) if s == "slice"));
    }
}
