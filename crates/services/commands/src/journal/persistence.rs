// SPDX-License-Identifier: AGPL-3.0-or-later
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

/// Journal storage backend (enum dispatch instead of `Box<dyn JournalPersistence>`).
pub enum JournalBackend {
    /// On-disk JSON journal file.
    File(FileJournalPersistence),
    /// In-memory journal (tests and ephemeral use).
    InMemory(InMemoryJournalPersistence),
}

impl JournalPersistence for JournalBackend {
    fn save_entry(&self, entry: &JournalEntry) -> JournalResult<()> {
        match self {
            Self::File(p) => p.save_entry(entry),
            Self::InMemory(p) => p.save_entry(entry),
        }
    }

    fn load_entries(&self) -> JournalResult<Vec<JournalEntry>> {
        match self {
            Self::File(p) => p.load_entries(),
            Self::InMemory(p) => p.load_entries(),
        }
    }

    fn delete_entry(&self, id: &str) -> JournalResult<()> {
        match self {
            Self::File(p) => p.delete_entry(id),
            Self::InMemory(p) => p.delete_entry(id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::journal::entry::{JournalEntry, JournalEntryState};
    use tempfile::tempdir;

    #[test]
    fn file_journal_missing_file_loads_empty() {
        let dir = tempdir().expect("should succeed");
        let path = dir.path().join("nj.json");
        let p = FileJournalPersistence::new(&path);
        assert!(p.load_entries().expect("should succeed").is_empty());
    }

    #[test]
    fn file_journal_save_load_update_delete_roundtrip() {
        let dir = tempdir().expect("should succeed");
        let path = dir.path().join("j.json");
        let p = FileJournalPersistence::new(&path);

        let mut e1 = JournalEntry::new("echo", vec!["a".into()]);
        p.save_entry(&e1).expect("should succeed");
        let loaded = p.load_entries().expect("should succeed");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].command_name, "echo");

        e1.state = JournalEntryState::Completed;
        p.save_entry(&e1).expect("should succeed");
        assert_eq!(
            p.load_entries().expect("should succeed")[0].state,
            JournalEntryState::Completed
        );

        p.delete_entry(&e1.id).expect("should succeed");
        assert!(p.load_entries().expect("should succeed").is_empty());
    }

    #[test]
    fn in_memory_journal_roundtrip() {
        let p = InMemoryJournalPersistence::new();
        let e = JournalEntry::new("ls", vec![]);
        let id = e.id.clone();
        p.save_entry(&e).expect("should succeed");
        assert_eq!(p.load_entries().expect("should succeed").len(), 1);
        p.delete_entry(&id).expect("should succeed");
        assert!(p.load_entries().expect("should succeed").is_empty());
    }

    #[test]
    fn in_memory_default_matches_new() {
        let _ = InMemoryJournalPersistence::default();
    }
}
