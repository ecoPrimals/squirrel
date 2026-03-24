// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Journal error types and result aliases.

use std::io;
use thiserror::Error;

/// Error types specific to journal operations
#[derive(Debug, Error)]
pub enum JournalError {
    /// Error during journal I/O operations
    #[error("Journal I/O error: {0}")]
    IoError(#[from] io::Error),

    /// Error during serialization or deserialization
    #[error("Journal serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Error when the journal entry is not found
    #[error("Journal entry not found: {0}")]
    EntryNotFound(String),

    /// Error when journal data is corrupted
    #[error("Journal data corrupted: {0}")]
    CorruptedData(String),

    /// Error when the journal is in an invalid state
    #[error("Invalid journal state: {0}")]
    InvalidState(String),

    /// Error during persistence operations
    #[error("Persistence error: {0}")]
    PersistenceError(String),
}

// Manual implementation of Clone for JournalError
#[allow(
    clippy::unwrap_used,
    reason = "Invalid JSON always fails; unwrap_err is safe"
)]
impl Clone for JournalError {
    fn clone(&self) -> Self {
        match self {
            Self::IoError(e) => Self::IoError(io::Error::new(e.kind(), e.to_string())),
            Self::SerializationError(_) => Self::SerializationError(
                serde_json::from_str::<serde_json::Value>("{").unwrap_err(),
            ),
            Self::EntryNotFound(s) => Self::EntryNotFound(s.clone()),
            Self::CorruptedData(s) => Self::CorruptedData(s.clone()),
            Self::InvalidState(s) => Self::InvalidState(s.clone()),
            Self::PersistenceError(s) => Self::PersistenceError(s.clone()),
        }
    }
}

/// Result type for journal operations
pub type JournalResult<T> = Result<T, JournalError>;
