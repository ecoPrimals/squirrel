//! Command history system for Squirrel
//!
//! This module provides functionality for tracking, persisting, and searching
//! command execution history, as well as replaying previous commands.

use std::collections::VecDeque;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use log::{debug, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::CommandError;

/// Result type for history operations
pub type HistoryResult<T> = Result<T, CommandError>;

/// Maximum number of entries to keep in memory
const DEFAULT_MAX_HISTORY_SIZE: usize = 1000;

/// Default history file path
const DEFAULT_HISTORY_FILE: &str = "command_history.json";

/// A single entry in the command history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Unique identifier for this history entry
    pub id: String,
    
    /// The command name that was executed
    pub command: String,
    
    /// The arguments passed to the command
    pub args: Vec<String>,
    
    /// Timestamp when the command was executed (as seconds since UNIX epoch)
    pub timestamp: u64,
    
    /// Whether the command executed successfully
    pub success: bool,
    
    /// Optional error message if the command failed
    pub error_message: Option<String>,
    
    /// Optional metadata about the command execution
    pub metadata: Option<serde_json::Value>,
}

impl HistoryEntry {
    /// Creates a new history entry for a command execution
    pub fn new(
        command: String, 
        args: Vec<String>, 
        success: bool, 
        error_message: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
            
        Self {
            id: Uuid::new_v4().to_string(),
            command,
            args,
            timestamp,
            success,
            error_message,
            metadata,
        }
    }
    
    /// Returns a formatted string representation of this history entry
    pub fn formatted(&self) -> String {
        let status = if self.success { "✓" } else { "✗" };
        let args_str = self.args.join(" ");
        
        format!(
            "[{}] {} {} {}",
            chrono::DateTime::from_timestamp(self.timestamp as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| self.timestamp.to_string()),
            status,
            self.command,
            args_str
        )
    }
}

/// Command history manager that handles storage, retrieval, and search of command history
#[derive(Debug)]
pub struct CommandHistory {
    /// In-memory history entries
    entries: Arc<RwLock<VecDeque<HistoryEntry>>>,
    
    /// Maximum number of entries to keep in memory
    max_size: usize,
    
    /// Path to the history file for persistence
    history_file: PathBuf,
    
    /// Lock for file operations to prevent concurrent writes
    file_lock: Arc<Mutex<()>>,
}

impl CommandHistory {
    /// Creates a new command history manager with default settings
    pub fn new() -> HistoryResult<Self> {
        Self::with_options(DEFAULT_MAX_HISTORY_SIZE, DEFAULT_HISTORY_FILE)
    }
    
    /// Creates a new command history manager with custom options
    pub fn with_options(max_size: usize, history_file: impl AsRef<Path>) -> HistoryResult<Self> {
        let history_file = history_file.as_ref().to_path_buf();
        
        // Create the history file if it doesn't exist
        if !history_file.exists() {
            debug!("History file does not exist, creating: {:?}", history_file);
            
            // Create parent directories if they don't exist
            if let Some(parent) = history_file.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).map_err(|err| {
                        CommandError::ResourceError(format!(
                            "Failed to create history directory: {}", err
                        ))
                    })?;
                }
            }
            
            // Create an empty file
            File::create(&history_file).map_err(|err| {
                CommandError::ResourceError(format!(
                    "Failed to create history file: {}", err
                ))
            })?;
        }
        
        let mut history = Self {
            entries: Arc::new(RwLock::new(VecDeque::new())),
            max_size,
            history_file,
            file_lock: Arc::new(Mutex::new(())),
        };
        
        // Load existing history
        history.load()?;
        
        Ok(history)
    }
    
    /// Adds a new entry to the command history
    pub fn add_entry(&self, entry: HistoryEntry) -> HistoryResult<()> {
        // Add to memory
        {
            let mut entries = self.entries.write().map_err(|err| {
                CommandError::ResourceError(format!(
                    "Failed to acquire write lock on history entries: {}", err
                ))
            })?;
            
            entries.push_front(entry.clone());
            
            // Trim if needed
            while entries.len() > self.max_size {
                entries.pop_back();
            }
        }
        
        // Persist to file
        self.save_entry(&entry)
    }
    
    /// Adds a command execution to history
    pub fn add(
        &self,
        command: String,
        args: Vec<String>,
        success: bool,
        error_message: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> HistoryResult<()> {
        let entry = HistoryEntry::new(command, args, success, error_message, metadata);
        self.add_entry(entry)
    }
    
    /// Searches the command history using the provided query
    pub fn search(&self, query: &str) -> HistoryResult<Vec<HistoryEntry>> {
        let entries = self.entries.read().map_err(|err| {
            CommandError::ResourceError(format!(
                "Failed to acquire read lock on history entries: {}", err
            ))
        })?;
        
        let results: Vec<HistoryEntry> = entries
            .iter()
            .filter(|entry| {
                entry.command.contains(query) || 
                entry.args.iter().any(|arg| arg.contains(query))
            })
            .cloned()
            .collect();
            
        Ok(results)
    }
    
    /// Gets the most recent entry for a specific command
    pub fn get_last_for_command(&self, command: &str) -> HistoryResult<Option<HistoryEntry>> {
        let entries = self.entries.read().map_err(|err| {
            CommandError::ResourceError(format!(
                "Failed to acquire read lock on history entries: {}", err
            ))
        })?;
        
        let entry = entries
            .iter()
            .find(|entry| entry.command == command)
            .cloned();
            
        Ok(entry)
    }
    
    /// Gets the last N entries from the history
    pub fn get_last(&self, count: usize) -> HistoryResult<Vec<HistoryEntry>> {
        let entries = self.entries.read().map_err(|err| {
            CommandError::ResourceError(format!(
                "Failed to acquire read lock on history entries: {}", err
            ))
        })?;
        
        let result: Vec<HistoryEntry> = entries
            .iter()
            .take(count)
            .cloned()
            .collect();
            
        Ok(result)
    }
    
    /// Clears the command history
    pub fn clear(&self) -> HistoryResult<()> {
        // Clear memory
        {
            let mut entries = self.entries.write().map_err(|err| {
                CommandError::ResourceError(format!(
                    "Failed to acquire write lock on history entries: {}", err
                ))
            })?;
            
            entries.clear();
        }
        
        // Clear file
        let _lock = self.file_lock.lock().map_err(|err| {
            CommandError::ResourceError(format!(
                "Failed to acquire file lock: {}", err
            ))
        })?;
        
        // Truncate the file
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.history_file)
            .map_err(|err| {
                CommandError::ResourceError(format!(
                    "Failed to open history file for truncation: {}", err
                ))
            })?;
            
        // Write an empty JSON array to maintain valid JSON structure
        file.set_len(0).map_err(|err| {
            CommandError::ResourceError(format!(
                "Failed to truncate history file: {}", err
            ))
        })?;
        
        // Write empty array
        let mut writer = io::BufWriter::new(file);
        writer.write_all(b"[]").map_err(|err| {
            CommandError::ResourceError(format!(
                "Failed to write empty array to history file: {}", err
            ))
        })?;
        
        Ok(())
    }
    
    /// Removes entries older than the specified timestamp
    pub fn cleanup_older_than(&self, timestamp: u64) -> HistoryResult<usize> {
        // Initialize removal count
        let mut removed_count = 0;
        
        // Remove from memory
        {
            let mut entries = self.entries.write().map_err(|err| {
                CommandError::ResourceError(format!(
                    "Failed to acquire write lock on history entries: {}", err
                ))
            })?;
            
            // Filter out old entries
            let original_len = entries.len();
            entries.retain(|entry| entry.timestamp > timestamp);
            removed_count = original_len - entries.len();
        }
        
        // Persist changes
        self.save_all()?;
        
        Ok(removed_count)
    }
    
    /// Loads history from the file
    fn load(&mut self) -> HistoryResult<()> {
        let _lock = self.file_lock.lock().map_err(|err| {
            CommandError::ResourceError(format!(
                "Failed to acquire file lock: {}", err
            ))
        })?;
        
        // Read the file
        let file = match File::open(&self.history_file) {
            Ok(file) => file,
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                // Create empty file
                return Ok(());
            }
            Err(err) => {
                return Err(CommandError::ResourceError(format!(
                    "Failed to open history file: {}", err
                )));
            }
        };
        
        let reader = BufReader::new(file);
        
        // Parse entries
        let mut entries = VecDeque::new();
        
        // Read entire file as JSON array
        match serde_json::from_reader::<_, Vec<HistoryEntry>>(reader) {
            Ok(loaded_entries) => {
                for entry in loaded_entries {
                    entries.push_back(entry);
                }
            }
            Err(err) => {
                warn!("Error parsing history file as JSON array: {}", err);
                
                // Try line-by-line parsing as fallback
                warn!("Attempting line-by-line parsing as fallback");
                let file = File::open(&self.history_file).map_err(|err| {
                    CommandError::ResourceError(format!(
                        "Failed to reopen history file: {}", err
                    ))
                })?;
                
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    match line {
                        Ok(line) if !line.trim().is_empty() => {
                            match serde_json::from_str::<HistoryEntry>(&line) {
                                Ok(entry) => {
                                    entries.push_back(entry);
                                }
                                Err(e) => {
                                    warn!("Failed to parse history entry: {}", e);
                                }
                            }
                        }
                        Ok(_) => {
                            // Skip empty lines
                        }
                        Err(e) => {
                            warn!("Error reading line from history file: {}", e);
                        }
                    }
                }
            }
        }
        
        // Sort by timestamp (newest first) and limit to max size
        let mut entries_vec: Vec<HistoryEntry> = entries.into_iter().collect();
        entries_vec.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries_vec.truncate(self.max_size);
        
        // Convert back to VecDeque
        let entries = VecDeque::from(entries_vec);
        
        // Update entries
        let mut entries_lock = self.entries.write().map_err(|err| {
            CommandError::ResourceError(format!(
                "Failed to acquire write lock on history entries: {}", err
            ))
        })?;
        
        *entries_lock = entries;
        
        Ok(())
    }
    
    /// Saves a single entry to the history file
    fn save_entry(&self, entry: &HistoryEntry) -> HistoryResult<()> {
        let _lock = self.file_lock.lock().map_err(|err| {
            CommandError::ResourceError(format!(
                "Failed to acquire file lock: {}", err
            ))
        })?;
        
        // Append to file
        // We'll use two approaches depending on file size:
        // 1. If file is empty, create a new JSON array
        // 2. If file already has content, update the array
        
        let file_metadata = std::fs::metadata(&self.history_file).map_err(|err| {
            CommandError::ResourceError(format!(
                "Failed to get history file metadata: {}", err
            ))
        })?;
        
        if file_metadata.len() == 0 {
            // Empty file, create new array
            let file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&self.history_file)
                .map_err(|err| {
                    CommandError::ResourceError(format!(
                        "Failed to open history file for writing: {}", err
                    ))
                })?;
                
            let entries = vec![entry.clone()];
            serde_json::to_writer_pretty(file, &entries).map_err(|err| {
                CommandError::ResourceError(format!(
                    "Failed to write history entry to file: {}", err
                ))
            })?;
        } else {
            // Read existing entries
            let existing_entries = {
                let file = File::open(&self.history_file).map_err(|err| {
                    CommandError::ResourceError(format!(
                        "Failed to open history file for reading: {}", err
                    ))
                })?;
                
                match serde_json::from_reader::<_, Vec<HistoryEntry>>(file) {
                    Ok(entries) => entries,
                    Err(err) => {
                        // Handle corrupted file - create new array
                        warn!("History file corrupted, creating new: {}", err);
                        Vec::new()
                    }
                }
            };
            
            // Create updated entries list
            let mut updated_entries = Vec::with_capacity(existing_entries.len() + 1);
            updated_entries.push(entry.clone());
            updated_entries.extend(existing_entries);
            
            // Limit to max size
            if updated_entries.len() > self.max_size {
                updated_entries.truncate(self.max_size);
            }
            
            // Write back to file
            let file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&self.history_file)
                .map_err(|err| {
                    CommandError::ResourceError(format!(
                        "Failed to open history file for writing: {}", err
                    ))
                })?;
                
            serde_json::to_writer_pretty(file, &updated_entries).map_err(|err| {
                CommandError::ResourceError(format!(
                    "Failed to write history entries to file: {}", err
                ))
            })?;
        }
        
        Ok(())
    }
    
    /// Saves all entries to the history file
    fn save_all(&self) -> HistoryResult<()> {
        let _lock = self.file_lock.lock().map_err(|err| {
            CommandError::ResourceError(format!(
                "Failed to acquire file lock: {}", err
            ))
        })?;
        
        let entries = self.entries.read().map_err(|err| {
            CommandError::ResourceError(format!(
                "Failed to acquire read lock on history entries: {}", err
            ))
        })?;
        
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.history_file)
            .map_err(|err| {
                CommandError::ResourceError(format!(
                    "Failed to open history file for writing: {}", err
                ))
            })?;
            
        let entries_vec: Vec<HistoryEntry> = entries.iter().cloned().collect();
        serde_json::to_writer_pretty(file, &entries_vec).map_err(|err| {
            CommandError::ResourceError(format!(
                "Failed to write history entries to file: {}", err
            ))
        })?;
        
        Ok(())
    }

    /// Cleans up history entries that exceed the maximum size limit
    pub fn cleanup(&mut self) -> Result<usize, Box<dyn Error>> {
        let mut entries = self.entries.write().map_err(|e| Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to acquire write lock: {}", e)
        )))?;
        
        let current_len = entries.len();
        
        if current_len <= self.max_size {
            return Ok(0);
        }
        
        let entries_to_remove = current_len - self.max_size;
        
        for _ in 0..entries_to_remove {
            entries.pop_front();
        }
        
        Ok(entries_to_remove)
    }

    /// Cleans up old entries if the history size exceeds the maximum limit
    /// This is used internally when adding new entries to maintain size limits
    #[allow(dead_code)]
    fn cleanup_if_needed(&mut self) -> Result<(), Box<dyn Error>> {
        self.cleanup()?;
        Ok(())
    }
}

// Implement Default for CommandHistory
impl Default for CommandHistory {
    fn default() -> Self {
        Self::new().expect("Failed to create default command history")
    }
}

/// Trait for replaying command history
pub trait HistoryReplay {
    /// Replays a command from history
    /// 
    /// Returns the result of the command execution
    fn replay(&self, entry: &HistoryEntry) -> HistoryResult<String>;
    
    /// Replays the last command executed
    /// 
    /// Returns the result of the command execution
    fn replay_last(&self) -> HistoryResult<String>;
    
    /// Replays the last command matching the given name
    /// 
    /// Returns the result of the command execution or an error if no matching command was found
    fn replay_last_command(&self, command: &str) -> HistoryResult<String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_history_entry_creation() {
        let entry = HistoryEntry::new(
            "test-command".to_string(),
            vec!["--arg1".to_string(), "value1".to_string()],
            true,
            None,
            None,
        );
        
        assert_eq!(entry.command, "test-command");
        assert_eq!(entry.args, vec!["--arg1".to_string(), "value1".to_string()]);
        assert!(entry.success);
        assert!(entry.error_message.is_none());
        assert!(entry.metadata.is_none());
    }
    
    #[test]
    fn test_history_add_and_get() {
        let dir = tempdir().unwrap();
        let history_file = dir.path().join("test-history.json");
        
        let history = CommandHistory::with_options(10, &history_file).unwrap();
        
        // Add some entries
        history.add(
            "cmd1".to_string(),
            vec!["arg1".to_string()],
            true,
            None,
            None,
        ).unwrap();
        
        history.add(
            "cmd2".to_string(),
            vec!["arg2".to_string()],
            false,
            Some("error".to_string()),
            None,
        ).unwrap();
        
        // Get last entries
        let last_entries = history.get_last(10).unwrap();
        assert_eq!(last_entries.len(), 2);
        assert_eq!(last_entries[0].command, "cmd2");
        assert_eq!(last_entries[1].command, "cmd1");
        
        // Get last for command
        let last_cmd1 = history.get_last_for_command("cmd1").unwrap();
        assert!(last_cmd1.is_some());
        assert_eq!(last_cmd1.unwrap().command, "cmd1");
    }
    
    #[test]
    fn test_history_search() {
        let dir = tempdir().unwrap();
        let history_file = dir.path().join("test-history.json");
        
        let history = CommandHistory::with_options(10, &history_file).unwrap();
        
        // Add some entries
        history.add(
            "find".to_string(),
            vec!["file.txt".to_string()],
            true,
            None,
            None,
        ).unwrap();
        
        history.add(
            "grep".to_string(),
            vec!["pattern".to_string(), "file.txt".to_string()],
            true,
            None,
            None,
        ).unwrap();
        
        // Search for "file.txt"
        let results = history.search("file.txt").unwrap();
        assert_eq!(results.len(), 2);
        
        // Search for "pattern"
        let results = history.search("pattern").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].command, "grep");
    }
    
    #[test]
    fn test_history_clear() {
        let dir = tempdir().unwrap();
        let history_file = dir.path().join("test-history.json");
        
        let history = CommandHistory::with_options(10, &history_file).unwrap();
        
        // Add some entries
        history.add(
            "cmd1".to_string(),
            vec!["arg1".to_string()],
            true,
            None,
            None,
        ).unwrap();
        
        history.add(
            "cmd2".to_string(),
            vec!["arg2".to_string()],
            true,
            None,
            None,
        ).unwrap();
        
        // Verify entries exist
        let entries = history.get_last(10).unwrap();
        assert_eq!(entries.len(), 2);
        
        // Clear history
        history.clear().unwrap();
        
        // Verify entries are gone
        let entries = history.get_last(10).unwrap();
        assert_eq!(entries.len(), 0);
        
        // Verify file is empty JSON array
        let contents = fs::read_to_string(&history_file).unwrap();
        assert_eq!(contents, "[]");
    }
    
    #[test]
    fn test_history_cleanup() {
        let dir = tempdir().unwrap();
        let history_file = dir.path().join("test-history.json");
        
        let history = CommandHistory::with_options(10, &history_file).unwrap();
        
        // Add entry with custom timestamp
        let mut entry1 = HistoryEntry::new(
            "old-cmd".to_string(),
            vec!["arg1".to_string()],
            true,
            None,
            None,
        );
        entry1.timestamp = 1000; // Old timestamp
        history.add_entry(entry1).unwrap();
        
        let mut entry2 = HistoryEntry::new(
            "new-cmd".to_string(),
            vec!["arg2".to_string()],
            true,
            None,
            None,
        );
        entry2.timestamp = 2000; // Newer timestamp
        history.add_entry(entry2).unwrap();
        
        // Cleanup entries older than 1500
        let removed = history.cleanup_older_than(1500).unwrap();
        assert_eq!(removed, 1);
        
        // Verify only new entry remains
        let entries = history.get_last(10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].command, "new-cmd");
    }
    
    #[test]
    fn test_history_persistence() {
        let dir = tempdir().unwrap();
        let history_file = dir.path().join("test-history.json");
        
        // Create history and add entries
        {
            let history = CommandHistory::with_options(10, &history_file).unwrap();
            
            history.add(
                "cmd1".to_string(),
                vec!["arg1".to_string()],
                true,
                None,
                None,
            ).unwrap();
            
            history.add(
                "cmd2".to_string(),
                vec!["arg2".to_string()],
                true,
                None,
                None,
            ).unwrap();
        }
        
        // Create new history instance and verify entries are loaded
        {
            let history = CommandHistory::with_options(10, &history_file).unwrap();
            
            let entries = history.get_last(10).unwrap();
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].command, "cmd2");
            assert_eq!(entries[1].command, "cmd1");
        }
    }
} 