// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Journal persistence traits and implementations.

use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use super::entry::JournalEntry;
use super::error::{JournalError, JournalResult};

/// Default journal file path
pub const DEFAULT_JOURNAL_FILE: &str = "command_journal.json";

/// Persistence interface for journal storage
pub trait JournalPersistence: Send + Sync {
    /// Save entries to persistent storage
    fn save_entry(&self, entry: &JournalEntry) -> JournalResult<()>;

    /// Load entries from persistent storage
    fn load_entries(&self) -> JournalResult<Vec<JournalEntry>>;

    /// Delete a journal entry
    fn delete_entry(&self, id: &str) -> JournalResult<()>;
}

/// File-based journal persistence
pub struct FileJournalPersistence {
    /// Path to the journal file
    file_path: PathBuf,
}

impl FileJournalPersistence {
    /// Creates a new file-based journal persistence
    pub fn new<P: AsRef<Path>>(file_path: P) -> Self {
        Self {
            file_path: file_path.as_ref().to_path_buf(),
        }
    }

    // Helper to load all entries
    fn load_all_entries(&self) -> JournalResult<Vec<JournalEntry>> {
        // If the file doesn't exist, return an empty vector
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let entries = serde_json::from_reader(reader)?;

        Ok(entries)
    }

    // Helper to save all entries
    fn save_all_entries(&self, entries: &[JournalEntry]) -> JournalResult<()> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.file_path)?;

        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &entries)?;

        Ok(())
    }
}

impl Default for FileJournalPersistence {
    fn default() -> Self {
        Self::new(DEFAULT_JOURNAL_FILE)
    }
}

impl JournalPersistence for FileJournalPersistence {
    fn save_entry(&self, entry: &JournalEntry) -> JournalResult<()> {
        let mut entries = self.load_all_entries()?;

        // Find and update the entry if it exists, or add a new one
        let entry_index = entries.iter().position(|e| e.id == entry.id);
        if let Some(index) = entry_index {
            entries[index] = entry.clone();
        } else {
            entries.push(entry.clone());
        }

        self.save_all_entries(&entries)
    }

    fn load_entries(&self) -> JournalResult<Vec<JournalEntry>> {
        self.load_all_entries()
    }

    fn delete_entry(&self, id: &str) -> JournalResult<()> {
        let mut entries = self.load_all_entries()?;
        entries.retain(|e| e.id != id);
        self.save_all_entries(&entries)
    }
}

/// In-memory journal persistence (for testing)
pub struct InMemoryJournalPersistence {
    /// Entries stored in memory
    entries: RwLock<Vec<JournalEntry>>,
}

impl Default for InMemoryJournalPersistence {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryJournalPersistence {
    /// Creates a new in-memory journal persistence
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
        }
    }
}

impl JournalPersistence for InMemoryJournalPersistence {
    fn save_entry(&self, entry: &JournalEntry) -> JournalResult<()> {
        let mut entries = self.entries.write().map_err(|_| {
            JournalError::PersistenceError("Failed to acquire write lock".to_string())
        })?;

        // Find and update the entry if it exists, or add a new one
        let entry_index = entries.iter().position(|e| e.id == entry.id);
        if let Some(index) = entry_index {
            entries[index] = entry.clone();
        } else {
            entries.push(entry.clone());
        }

        Ok(())
    }

    fn load_entries(&self) -> JournalResult<Vec<JournalEntry>> {
        let entries = self.entries.read().map_err(|_| {
            JournalError::PersistenceError("Failed to acquire read lock".to_string())
        })?;
        Ok(entries.clone())
    }

    fn delete_entry(&self, id: &str) -> JournalResult<()> {
        let mut entries = self.entries.write().map_err(|_| {
            JournalError::PersistenceError("Failed to acquire write lock".to_string())
        })?;
        entries.retain(|e| e.id != id);
        Ok(())
    }
}
