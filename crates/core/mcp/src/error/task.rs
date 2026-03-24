// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Task-related error types for the MCP system

/// Errors that can occur during task operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum TaskError {
    /// Task not found
    #[error("Task not found: {0}")]
    NotFound(String),

    /// Task execution failed
    #[error("Task execution failed: {0}")]
    ExecutionFailed(String),

    /// Task timeout
    #[error("Task timeout: {0}")]
    Timeout(String),

    /// Task dependency error
    #[error("Task dependency error: {0}")]
    DependencyError(String),

    /// Task validation error
    #[error("Task validation failed: {0}")]
    ValidationError(String),

    /// Task invalid state
    #[error("Task invalid state: {0}")]
    InvalidState(String),
}

impl TaskError {
    /// Create a new not found error
    pub fn not_found(id: impl Into<String>) -> Self {
        Self::NotFound(id.into())
    }

    /// Create a new execution failed error
    pub fn execution_failed(msg: impl Into<String>) -> Self {
        Self::ExecutionFailed(msg.into())
    }

    /// Create a new timeout error
    #[must_use]
    pub fn timeout(seconds: u64) -> Self {
        Self::Timeout(seconds.to_string())
    }
}

#[cfg(test)]
mod tests {
    // SPDX-License-Identifier: AGPL-3.0-or-later
    // Inline tests follow the pattern used in `context.rs` and `severity.rs`.

    use super::TaskError;
    use std::fmt::Write as _;

    #[test]
    fn task_error_display_not_found() {
        let err = TaskError::NotFound("t1".into());
        assert!(err.to_string().contains("Task not found"));
        assert!(err.to_string().contains("t1"));
    }

    #[test]
    fn task_error_display_execution_failed() {
        let err = TaskError::ExecutionFailed("boom".into());
        assert!(err.to_string().contains("execution"));
        assert!(err.to_string().contains("boom"));
    }

    #[test]
    fn task_error_display_timeout() {
        let err = TaskError::Timeout("9".into());
        assert!(err.to_string().contains("timeout"));
    }

    #[test]
    fn task_error_display_dependency_error() {
        let err = TaskError::DependencyError("dep".into());
        assert!(err.to_string().contains("dependency"));
    }

    #[test]
    fn task_error_display_validation_error() {
        let err = TaskError::ValidationError("inv".into());
        assert!(err.to_string().contains("validation"));
    }

    #[test]
    fn task_error_display_invalid_state() {
        let err = TaskError::InvalidState("bad".into());
        assert!(err.to_string().contains("invalid state"));
    }

    #[test]
    fn task_error_debug_all_variants() {
        let cases = [
            TaskError::NotFound("a".into()),
            TaskError::ExecutionFailed("b".into()),
            TaskError::Timeout("c".into()),
            TaskError::DependencyError("d".into()),
            TaskError::ValidationError("e".into()),
            TaskError::InvalidState("f".into()),
        ];
        for e in cases {
            let mut buf = String::new();
            write!(&mut buf, "{e:?}").expect("format");
            assert!(!buf.is_empty());
        }
    }

    #[test]
    fn task_error_helpers() {
        assert!(matches!(TaskError::not_found("id"), TaskError::NotFound(s) if s == "id"));
        assert!(matches!(
            TaskError::execution_failed("e"),
            TaskError::ExecutionFailed(s) if s == "e"
        ));
        assert!(matches!(TaskError::timeout(7), TaskError::Timeout(s) if s == "7"));
    }
}
