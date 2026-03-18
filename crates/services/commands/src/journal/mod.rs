// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Command Journaling System
//!
//! This module provides persistent logging of command execution with support for
//! recovery and audit capabilities.

mod command_journal;
mod entry;
mod error;
mod persistence;

#[cfg(test)]
mod tests;

// Re-export public API for backward compatibility
pub use command_journal::CommandJournal;
pub use entry::{JournalEntry, JournalEntryState, RecoveryReport};
pub use error::{JournalError, JournalResult};
pub use persistence::{FileJournalPersistence, InMemoryJournalPersistence, JournalPersistence};
