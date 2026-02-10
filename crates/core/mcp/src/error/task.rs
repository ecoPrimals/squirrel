// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

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
    pub fn timeout(seconds: u64) -> Self {
        Self::Timeout(seconds.to_string())
    }
}
