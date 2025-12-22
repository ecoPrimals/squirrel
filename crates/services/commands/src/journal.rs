//! Command Journaling System
//!
//! This module provides persistent logging of command execution with support for
//! recovery and audit capabilities.

use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::CommandError;
use crate::{Command, CommandResult};

/// Maximum number of journal entries to keep in memory by default
const DEFAULT_MAX_ENTRIES: usize = 1000;

/// Default journal file path
const DEFAULT_JOURNAL_FILE: &str = "command_journal.json";

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
impl Clone for JournalError {
    fn clone(&self) -> Self {
        match self {
            Self::IoError(e) => Self::IoError(io::Error::new(e.kind(), e.to_string())),
            Self::SerializationError(_) => Self::SerializationError(
                serde_json::from_str::<serde_json::Value>("{}").unwrap_err(),
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

/// Command journal for tracking command execution
pub struct CommandJournal {
    /// Journal entries
    entries: Arc<RwLock<VecDeque<JournalEntry>>>,

    /// Journal persistence
    persistence: Arc<dyn JournalPersistence>,

    /// Maximum number of entries to keep in memory
    max_entries: usize,
}

impl CommandJournal {
    /// Creates a new command journal with persistence
    pub fn new(persistence: Arc<dyn JournalPersistence>, max_entries: usize) -> Self {
        let journal = Self {
            entries: Arc::new(RwLock::new(VecDeque::with_capacity(max_entries))),
            persistence,
            max_entries,
        };

        // Load entries from persistence
        if let Err(e) = journal.load_entries() {
            error!("Failed to load journal entries: {}", e);
        }

        journal
    }

    /// Creates a new command journal with file persistence
    pub fn with_file<P: AsRef<Path>>(file_path: P, max_entries: usize) -> Self {
        let persistence = Arc::new(FileJournalPersistence::new(file_path));
        Self::new(persistence, max_entries)
    }

    /// Handle poisoned lock by recovering safely (write)
    fn handle_poisoned_entries_write(
        &self,
    ) -> JournalResult<std::sync::RwLockWriteGuard<'_, VecDeque<JournalEntry>>> {
        self.entries
            .write()
            .or_else(
                |poison: std::sync::PoisonError<
                    std::sync::RwLockWriteGuard<'_, VecDeque<JournalEntry>>,
                >| {
                    warn!("Journal entries lock was poisoned, recovering");
                    Ok::<
                        std::sync::RwLockWriteGuard<'_, VecDeque<JournalEntry>>,
                        std::sync::PoisonError<
                            std::sync::RwLockWriteGuard<'_, VecDeque<JournalEntry>>,
                        >,
                    >(poison.into_inner())
                },
            )
            .map_err(|e: std::sync::PoisonError<_>| {
                JournalError::CorruptedData(format!("Lock error: {}", e))
            })
    }

    /// Handle poisoned lock by recovering safely (read)
    fn handle_poisoned_entries_read(
        &self,
    ) -> JournalResult<std::sync::RwLockReadGuard<'_, VecDeque<JournalEntry>>> {
        self.entries
            .read()
            .or_else(
                |poison: std::sync::PoisonError<
                    std::sync::RwLockReadGuard<'_, VecDeque<JournalEntry>>,
                >| {
                    warn!("Journal entries lock was poisoned, recovering");
                    Ok::<
                        std::sync::RwLockReadGuard<'_, VecDeque<JournalEntry>>,
                        std::sync::PoisonError<
                            std::sync::RwLockReadGuard<'_, VecDeque<JournalEntry>>,
                        >,
                    >(poison.into_inner())
                },
            )
            .map_err(|e: std::sync::PoisonError<_>| {
                JournalError::CorruptedData(format!("Lock error: {}", e))
            })
    }

    /// Creates a new journal entry for a command execution
    pub fn record_start(&self, command: &dyn Command, args: &[String]) -> JournalResult<String> {
        let entry = JournalEntry::new(command.name(), args.to_vec());
        let id = entry.id.clone();

        debug!("Recording command start: {} (ID: {})", command.name(), id);

        // Add the entry to the journal
        {
            let mut entries = self.handle_poisoned_entries_write()?;

            // If the journal is full, remove the oldest entry
            if entries.len() >= self.max_entries {
                entries.pop_front();
            }

            entries.push_back(entry.clone());
        }

        // Save entry to persistence
        self.persistence.save_entry(&entry)?;

        Ok(id)
    }

    /// Records the completion of a command execution
    pub fn record_completion(
        &self,
        id: String,
        result: CommandResult<String>,
    ) -> JournalResult<()> {
        let mut updated = false;

        // Update the entry in the journal
        {
            let mut entries = self.handle_poisoned_entries_write()?;

            for entry in entries.iter_mut() {
                if entry.id == id {
                    entry.complete(result);
                    updated = true;

                    // Save the updated entry to persistence
                    self.persistence.save_entry(entry)?;

                    break;
                }
            }
        }

        if !updated {
            return Err(JournalError::EntryNotFound(id));
        }

        Ok(())
    }

    /// Finds incomplete journal entries
    pub fn find_incomplete(&self) -> JournalResult<Vec<JournalEntry>> {
        let entries = self.handle_poisoned_entries_read()?;
        let incomplete = entries
            .iter()
            .filter(|e| e.state == JournalEntryState::Started)
            .cloned()
            .collect();

        Ok(incomplete)
    }

    /// Recovers incomplete commands
    pub fn recover_incomplete<F>(&self, recover_fn: F) -> JournalResult<RecoveryReport>
    where
        F: Fn(&JournalEntry) -> CommandResult<String>,
    {
        let incomplete = self.find_incomplete()?;
        let mut report = RecoveryReport {
            processed: incomplete.len(),
            recovered: 0,
            failed: 0,
            recovered_entries: Vec::new(),
            failed_entries: Vec::new(),
        };

        if incomplete.is_empty() {
            return Ok(report);
        }

        info!("Recovering {} incomplete commands", incomplete.len());

        for incomplete_entry in incomplete {
            let id = incomplete_entry.id.clone();
            let command_name = incomplete_entry.command_name.clone();

            debug!("Recovering command: {} (ID: {})", command_name, id);

            // Try to recover the command
            let result = recover_fn(&incomplete_entry);

            // Update the entry with the recovery result
            {
                let mut entries = self.handle_poisoned_entries_write()?;

                for entry in entries.iter_mut() {
                    if entry.id == id {
                        entry.complete(result);
                        entry.recover();

                        // Save the updated entry to persistence
                        self.persistence.save_entry(entry)?;

                        if entry.state == JournalEntryState::Recovered {
                            report.recovered += 1;
                            report.recovered_entries.push(entry.clone());
                            info!(
                                "Successfully recovered command: {} (ID: {})",
                                command_name, id
                            );
                        } else {
                            report.failed += 1;
                            report.failed_entries.push(entry.clone());
                            warn!("Failed to recover command: {} (ID: {})", command_name, id);
                        }

                        break;
                    }
                }
            }
        }

        info!(
            "Recovery completed: {} processed, {} recovered, {} failed",
            report.processed, report.recovered, report.failed
        );

        Ok(report)
    }

    /// Gets all journal entries
    pub fn get_entries(&self) -> JournalResult<Vec<JournalEntry>> {
        let entries = self.handle_poisoned_entries_read()?;
        Ok(entries.iter().cloned().collect())
    }

    /// Gets a journal entry by ID
    pub fn get_entry(&self, id: String) -> JournalResult<JournalEntry> {
        let entries = self.handle_poisoned_entries_read()?;

        for entry in entries.iter() {
            if entry.id == id {
                return Ok(entry.clone());
            }
        }

        Err(JournalError::EntryNotFound(id))
    }

    /// Searches for journal entries by criteria
    pub fn search_entries<F>(&self, predicate: F) -> JournalResult<Vec<JournalEntry>>
    where
        F: Fn(&JournalEntry) -> bool,
    {
        let entries = self.handle_poisoned_entries_read()?;
        let matches = entries.iter().filter(|e| predicate(e)).cloned().collect();
        Ok(matches)
    }

    /// Loads entries from persistence
    fn load_entries(&self) -> JournalResult<()> {
        let loaded_entries = self.persistence.load_entries()?;

        if !loaded_entries.is_empty() {
            let mut entries = self.handle_poisoned_entries_write()?;
            entries.clear();

            // Only keep the most recent entries up to max_entries
            let start_idx = loaded_entries.len().saturating_sub(self.max_entries);
            for entry in loaded_entries.into_iter().skip(start_idx) {
                entries.push_back(entry);
            }

            info!("Loaded {} journal entries", entries.len());
        }

        Ok(())
    }
}

impl Default for CommandJournal {
    fn default() -> Self {
        let persistence = Arc::new(FileJournalPersistence::default());
        Self::new(persistence, DEFAULT_MAX_ENTRIES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, CommandError, CommandResult};
    use clap::Command as ClapCommand;
    use std::sync::Arc;

    // Test command implementation
    struct TestCommand;

    impl Command for TestCommand {
        fn name(&self) -> &str {
            "test-command"
        }

        fn description(&self) -> &str {
            "A test command for unit tests"
        }

        fn execute(&self, args: &[String]) -> CommandResult<String> {
            if args.is_empty() {
                return Ok("Success with no args".to_string());
            }

            // Fail if first argument is "fail"
            if args[0] == "fail" {
                return Err(CommandError::ExecutionError("Command failed".to_string()));
            }

            Ok(format!("Success with args: {:?}", args))
        }

        fn parser(&self) -> ClapCommand {
            ClapCommand::new("test-command").about("A test command for unit tests")
        }

        fn clone_box(&self) -> Box<dyn Command> {
            Box::new(Self {})
        }
    }

    #[test]
    fn test_journal_entry_creation() {
        let entry = JournalEntry::new("test-command", vec!["arg1".to_string(), "arg2".to_string()]);

        assert_eq!(entry.command_name, "test-command");
        assert_eq!(
            entry.arguments,
            vec!["arg1".to_string(), "arg2".to_string()]
        );
        assert_eq!(entry.state, JournalEntryState::Started);
        assert!(entry.result.is_none());
        assert!(entry.error.is_none());
        assert_eq!(entry.retry_count, 0);
        assert!(entry.execution_time.is_none());
        assert!(entry.user.is_none());
    }

    #[test]
    fn test_journal_entry_complete() {
        let mut entry =
            JournalEntry::new("test-command", vec!["arg1".to_string(), "arg2".to_string()]);

        let result = Ok("Command executed successfully".to_string());
        entry.complete(result.clone());

        assert_eq!(entry.state, JournalEntryState::Completed);
        assert_eq!(
            entry.result,
            Some("Command executed successfully".to_string())
        );
        assert!(entry.error.is_none());
        assert!(entry.execution_time.is_some());
    }

    #[test]
    fn test_journal_entry_fail() {
        let mut entry = JournalEntry::new("test-command", vec!["fail".to_string()]);

        let error = Err(CommandError::ExecutionError("Command failed".to_string()));
        entry.complete(error);

        assert_eq!(entry.state, JournalEntryState::Failed);
        assert!(entry.result.is_none());
        assert_eq!(
            entry.error,
            Some("Execution error: Command failed".to_string())
        );
        assert!(entry.execution_time.is_some());
    }

    #[test]
    fn test_in_memory_persistence() {
        let persistence = InMemoryJournalPersistence::new();

        // Create an entry
        let entry = JournalEntry::new("test-command", vec!["arg1".to_string()]);

        // Save the entry
        persistence.save_entry(&entry).unwrap();

        // Load all entries
        let entries = persistence.load_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].command_name, "test-command");

        // Complete the entry and update
        let mut updated_entry = entry.clone();
        updated_entry.complete(Ok("Success".to_string()));
        persistence.save_entry(&updated_entry).unwrap();

        // Verify the update
        let updated_entries = persistence.load_entries().unwrap();
        assert_eq!(updated_entries.len(), 1);
        assert_eq!(updated_entries[0].state, JournalEntryState::Completed);

        // Delete the entry
        persistence.delete_entry(&entry.id).unwrap();

        // Verify deletion
        let empty_entries = persistence.load_entries().unwrap();
        assert_eq!(empty_entries.len(), 0);
    }

    #[test]
    fn test_command_journal_basic_workflow() {
        // Create a journal with in-memory persistence
        let persistence = Arc::new(InMemoryJournalPersistence::new());
        let journal = CommandJournal::new(persistence.clone(), 100);

        let command = TestCommand {};
        let args = vec!["arg1".to_string(), "arg2".to_string()];

        // Record command start
        let id = journal.record_start(&command, &args).unwrap();

        // Execute the command
        let result = command.execute(&args);

        // Record command completion
        journal
            .record_completion(id.clone(), result.clone())
            .unwrap();

        // Get the entry
        let entry = journal.get_entry(id).unwrap();

        // Verify entry state
        assert_eq!(entry.state, JournalEntryState::Completed);
        assert_eq!(
            entry.result,
            Some("Success with args: [\"arg1\", \"arg2\"]".to_string())
        );
    }

    #[test]
    fn test_command_journal_failed_command() {
        // Create a journal with in-memory persistence
        let persistence = Arc::new(InMemoryJournalPersistence::new());
        let journal = CommandJournal::new(persistence.clone(), 100);

        let command = TestCommand {};
        let args = vec!["fail".to_string()];

        // Record command start
        let id = journal.record_start(&command, &args).unwrap();

        // Execute the command (which will fail)
        let result = command.execute(&args);

        // Record command completion
        journal.record_completion(id.clone(), result).unwrap();

        // Get the entry
        let entry = journal.get_entry(id).unwrap();

        // Verify entry state
        assert_eq!(entry.state, JournalEntryState::Failed);
        assert!(entry.result.is_none());
        assert!(entry.error.is_some());
    }

    #[test]
    fn test_find_incomplete_entries() {
        // Create a journal with in-memory persistence
        let persistence = Arc::new(InMemoryJournalPersistence::new());
        let journal = CommandJournal::new(persistence.clone(), 100);

        let command = TestCommand {};

        // Create a completed entry
        let id1 = journal
            .record_start(&command, &["success".to_string()])
            .unwrap();
        journal
            .record_completion(id1, Ok("Success".to_string()))
            .unwrap();

        // Create an incomplete entry
        let id2 = journal
            .record_start(&command, &["incomplete".to_string()])
            .unwrap();

        // Find incomplete entries
        let incomplete = journal.find_incomplete().unwrap();

        // Verify incomplete entries
        assert_eq!(incomplete.len(), 1);
        assert_eq!(incomplete[0].id, id2);
        assert_eq!(incomplete[0].state, JournalEntryState::Started);
    }

    #[test]
    fn test_recover_incomplete_entries() {
        // Create a journal with in-memory persistence
        let persistence = Arc::new(InMemoryJournalPersistence::new());
        let journal = CommandJournal::new(persistence.clone(), 100);

        let command = TestCommand {};

        // Create an incomplete entry
        let id = journal
            .record_start(&command, &["recoverable".to_string()])
            .unwrap();

        // Recover incomplete entries
        let recovery_report = journal
            .recover_incomplete(|entry| {
                assert_eq!(entry.arguments[0], "recoverable");
                Ok("Recovered successfully".to_string())
            })
            .unwrap();

        // Verify recovery report
        assert_eq!(recovery_report.recovered_entries.len(), 1);
        assert_eq!(recovery_report.failed_entries.len(), 0);

        // Verify entry was updated
        let entry = journal.get_entry(id).unwrap();
        assert_eq!(entry.state, JournalEntryState::Recovered);
        assert_eq!(entry.result, Some("Recovered successfully".to_string()));
    }

    #[test]
    fn test_entry_search() {
        // Create a journal with in-memory persistence
        let persistence = Arc::new(InMemoryJournalPersistence::new());
        let journal = CommandJournal::new(persistence.clone(), 100);

        let command = TestCommand {};

        // Create various entries
        let id1 = journal
            .record_start(&command, &["search1".to_string()])
            .unwrap();
        journal
            .record_completion(id1.clone(), Ok("Success 1".to_string()))
            .unwrap();

        let id2 = journal
            .record_start(&command, &["search2".to_string()])
            .unwrap();
        journal
            .record_completion(id2.clone(), Ok("Success 2".to_string()))
            .unwrap();

        let id3 = journal
            .record_start(&command, &["fail".to_string()])
            .unwrap();
        journal
            .record_completion(
                id3.clone(),
                Err(CommandError::ExecutionError("Failed".to_string())),
            )
            .unwrap();

        // Search for completed entries
        let completed = journal
            .search_entries(|entry| entry.state == JournalEntryState::Completed)
            .unwrap();
        assert_eq!(completed.len(), 2);

        // Search for failed entries
        let failed = journal
            .search_entries(|entry| entry.state == JournalEntryState::Failed)
            .unwrap();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].id, id3);

        // Search by argument content
        let search2 = journal
            .search_entries(|entry| entry.arguments.iter().any(|arg| arg.contains("search2")))
            .unwrap();
        assert_eq!(search2.len(), 1);
        assert_eq!(search2[0].id, id2);
    }

    #[test]
    fn test_journal_capacity() {
        // Create a journal with small capacity
        let persistence = Arc::new(InMemoryJournalPersistence::new());
        let journal = CommandJournal::new(persistence.clone(), 2);

        let command = TestCommand {};

        // Add entries to reach capacity
        let id1 = journal
            .record_start(&command, &["first".to_string()])
            .unwrap();
        journal
            .record_completion(id1.clone(), Ok("First success".to_string()))
            .unwrap();

        let id2 = journal
            .record_start(&command, &["second".to_string()])
            .unwrap();
        journal
            .record_completion(id2.clone(), Ok("Second success".to_string()))
            .unwrap();

        // Verify entries
        let entries = journal.get_entries().unwrap();
        assert_eq!(entries.len(), 2);

        // Add one more entry (should evict oldest)
        let id3 = journal
            .record_start(&command, &["third".to_string()])
            .unwrap();
        journal
            .record_completion(id3.clone(), Ok("Third success".to_string()))
            .unwrap();

        // Verify entries (should now have id2 and id3, but not id1)
        let updated_entries = journal.get_entries().unwrap();
        assert_eq!(updated_entries.len(), 2);

        // The first entry should be evicted
        let entry_ids: Vec<String> = updated_entries.iter().map(|e| e.id.clone()).collect();
        assert!(!entry_ids.contains(&id1));
        assert!(entry_ids.contains(&id2));
        assert!(entry_ids.contains(&id3));
    }
}
