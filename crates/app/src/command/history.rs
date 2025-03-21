use std::sync::Arc;
use std::collections::VecDeque;
use tokio::sync::RwLock;
use crate::error::Result;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use squirrel_commands::Command;

/// Maximum number of commands to keep in history
const MAX_HISTORY_SIZE: usize = 1000;

/// Represents a command execution entry in history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistoryEntry {
    /// Command name
    pub command_name: String,
    /// Timestamp when the command was executed
    pub timestamp: DateTime<Utc>,
    /// Whether the command execution was successful
    pub success: bool,
    /// Additional metadata about the command execution
    pub metadata: Option<serde_json::Value>,
}

/// Manages command execution history
#[derive(Debug)]
pub struct CommandHistory {
    /// Command history entries
    entries: Arc<RwLock<VecDeque<CommandHistoryEntry>>>,
}

impl CommandHistory {
    /// Creates a new command history manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_HISTORY_SIZE))),
        }
    }

    /// Records a command execution in history
    /// 
    /// # Arguments
    /// * `command` - The command that was executed
    /// * `success` - Whether the execution was successful
    /// * `metadata` - Optional metadata about the execution
    /// 
    /// # Errors
    /// Returns an error if the history update fails
    pub async fn record(&self, command: &dyn Command, success: bool, metadata: Option<serde_json::Value>) -> Result<()> {
        let entry = CommandHistoryEntry {
            command_name: command.name().to_string(),
            timestamp: Utc::now(),
            success,
            metadata,
        };

        let mut entries = self.entries.write().await;
        if entries.len() >= MAX_HISTORY_SIZE {
            entries.pop_front();
        }
        entries.push_back(entry);
        Ok(())
    }

    /// Searches command history for entries matching the given criteria
    /// 
    /// # Arguments
    /// * `query` - The search query to match against command names
    /// * `limit` - Maximum number of entries to return
    /// 
    /// # Returns
    /// A vector of matching history entries
    pub async fn search(&self, query: &str, limit: usize) -> Vec<CommandHistoryEntry> {
        let entries = self.entries.read().await;
        entries
            .iter()
            .filter(|entry| entry.command_name.contains(query))
            .take(limit)
            .cloned()
            .collect()
    }

    /// Gets the last N command history entries
    /// 
    /// # Arguments
    /// * `count` - Number of entries to retrieve
    /// 
    /// # Returns
    /// A vector of the most recent history entries
    pub async fn get_recent(&self, count: usize) -> Vec<CommandHistoryEntry> {
        let entries = self.entries.read().await;
        entries
            .iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }

    /// Gets successful command executions matching the given name pattern
    /// 
    /// # Arguments
    /// * `pattern` - Pattern to match against command names
    /// * `limit` - Maximum number of entries to return
    /// 
    /// # Returns
    /// A vector of matching successful command executions
    pub async fn get_successful(&self, pattern: &str, limit: usize) -> Vec<CommandHistoryEntry> {
        let entries = self.entries.read().await;
        entries
            .iter()
            .filter(|entry| entry.success && entry.command_name.contains(pattern))
            .take(limit)
            .cloned()
            .collect()
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;
    use squirrel_commands::{Command, CommandError};
    use clap::Command as ClapCommand;

    #[derive(Debug)]
    struct TestCommand {
        name: &'static str,
    }

    impl Command for TestCommand {
        fn name(&self) -> &str {
            self.name
        }
        
        fn description(&self) -> &str {
            "Test command for unit tests"
        }
        
        fn execute(&self, _args: &[String]) -> std::result::Result<String, CommandError> {
            Ok("Test command executed".to_string())
        }
        
        fn parser(&self) -> ClapCommand {
            ClapCommand::new(self.name)
                .about("Test command for unit tests")
        }
        
        fn clone_box(&self) -> Box<dyn Command + 'static> {
            Box::new(TestCommand {
                name: self.name,
            })
        }
    }

    impl TestCommand {
        fn new(name: &'static str) -> Self {
            Self { name }
        }
    }

    #[tokio::test]
    async fn test_record_and_search() {
        let history = CommandHistory::new();
        let cmd = TestCommand::new("test_command");

        // Record some commands
        history.record(&cmd, true, None).await.unwrap();
        history.record(&cmd, false, None).await.unwrap();

        // Search for commands
        let results = history.search("test", 10).await;
        assert_eq!(results.len(), 2);
        assert!(results[0].success);
        assert!(!results[1].success);
    }

    #[tokio::test]
    async fn test_max_history_size() {
        let history = CommandHistory::new();
        let cmd = TestCommand::new("test_command");

        // Record more than MAX_HISTORY_SIZE commands
        for _ in 0..=MAX_HISTORY_SIZE {
            history.record(&cmd, true, None).await.unwrap();
        }

        let entries = history.entries.read().await;
        assert_eq!(entries.len(), MAX_HISTORY_SIZE);
    }

    #[tokio::test]
    async fn test_get_recent() {
        let history = CommandHistory::new();
        let cmd1 = TestCommand::new("cmd1");
        let cmd2 = TestCommand::new("cmd2");

        history.record(&cmd1, true, None).await.unwrap();
        history.record(&cmd2, true, None).await.unwrap();

        let recent = history.get_recent(1).await;
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].command_name, "cmd2");
    }

    #[tokio::test]
    async fn test_get_successful() {
        let history = CommandHistory::new();
        let cmd = TestCommand::new("test_command");

        history.record(&cmd, true, None).await.unwrap();
        history.record(&cmd, false, None).await.unwrap();

        let successful = history.get_successful("test", 10).await;
        assert_eq!(successful.len(), 1);
        assert!(successful[0].success);
    }
} 