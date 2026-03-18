// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Journal entry types and state.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::CommandError;

/// State of a journal entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JournalEntryState {
    /// Command execution has started
    Started,

    /// Command execution completed successfully
    Completed,

    /// Command execution failed
    Failed,

    /// Command execution was recovered
    Recovered,
}

impl std::fmt::Display for JournalEntryState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JournalEntryState::Started => write!(f, "Started"),
            JournalEntryState::Completed => write!(f, "Completed"),
            JournalEntryState::Failed => write!(f, "Failed"),
            JournalEntryState::Recovered => write!(f, "Recovered"),
        }
    }
}

/// Journal entry for command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Unique identifier for the journal entry
    pub id: String,

    /// Name of the command
    pub command_name: String,

    /// Command arguments
    pub arguments: Vec<String>,

    /// Timestamp when the command started executing
    pub timestamp: DateTime<Utc>,

    /// Current state of the journal entry
    pub state: JournalEntryState,

    /// Result of the command execution (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,

    /// Error message if the command failed (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Retry count for the command
    pub retry_count: u32,

    /// Execution time in milliseconds (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_time: Option<u64>,

    /// User who executed the command (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl JournalEntry {
    /// Creates a new journal entry
    pub fn new(command_name: &str, arguments: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            command_name: command_name.to_string(),
            arguments,
            timestamp: Utc::now(),
            state: JournalEntryState::Started,
            result: None,
            error: None,
            retry_count: 0,
            execution_time: None,
            user: None,
        }
    }

    /// Sets the entry state to completed with a result
    pub fn complete(&mut self, result: Result<String, CommandError>) {
        let execution_time = (Utc::now() - self.timestamp).num_milliseconds() as u64;
        self.execution_time = Some(execution_time);

        match result {
            Ok(output) => {
                self.state = JournalEntryState::Completed;
                self.result = Some(output);
            }
            Err(e) => {
                self.state = JournalEntryState::Failed;
                self.error = Some(e.to_string());
            }
        }
    }

    /// Sets the entry state to recovered
    pub fn recover(&mut self) {
        self.state = JournalEntryState::Recovered;
        self.retry_count += 1;
    }

    /// Sets the user who executed the command
    pub fn set_user(&mut self, user: String) {
        self.user = Some(user);
    }
}

/// Report of recovered journal entries
#[derive(Debug)]
pub struct RecoveryReport {
    /// Number of entries processed
    pub processed: usize,

    /// Number of entries recovered
    pub recovered: usize,

    /// Number of entries that failed recovery
    pub failed: usize,

    /// Details of recovered entries
    pub recovered_entries: Vec<JournalEntry>,

    /// Details of failed entries
    pub failed_entries: Vec<JournalEntry>,
}
