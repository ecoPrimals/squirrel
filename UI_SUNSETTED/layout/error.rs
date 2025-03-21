use thiserror::Error;

#[derive(Debug, Error)]
pub enum LayoutError {
    #[error("Invalid indentation size")]
    InvalidIndentSize,

    #[error("Cannot dedent below zero")]
    NegativeIndentation,

    #[error("Invalid constraint: {message}")]
    InvalidConstraint {
        message: String,
    },

    #[error("Cache error: {message}")]
    CacheError {
        message: String,
    },

    #[error("Maximum recursion depth exceeded")]
    MaxRecursionDepthExceeded,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
} 