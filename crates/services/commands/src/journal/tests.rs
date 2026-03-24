// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Journal module tests.

use std::sync::Arc;

use clap::Command as ClapCommand;

use crate::{Command, CommandError, CommandResult};

use super::command_journal::CommandJournal;
use super::entry::{JournalEntry, JournalEntryState};
use super::persistence::{InMemoryJournalPersistence, JournalPersistence};

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
    let mut entry = JournalEntry::new("test-command", vec!["arg1".to_string(), "arg2".to_string()]);

    let result = Ok("Command executed successfully".to_string());
    entry.complete(result);

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
    persistence.save_entry(&entry).expect("should succeed");

    // Load all entries
    let entries = persistence.load_entries().expect("should succeed");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].command_name, "test-command");

    // Complete the entry and update
    let mut updated_entry = entry.clone();
    updated_entry.complete(Ok("Success".to_string()));
    persistence
        .save_entry(&updated_entry)
        .expect("should succeed");

    // Verify the update
    let updated_entries = persistence.load_entries().expect("should succeed");
    assert_eq!(updated_entries.len(), 1);
    assert_eq!(updated_entries[0].state, JournalEntryState::Completed);

    // Delete the entry
    persistence.delete_entry(&entry.id).expect("should succeed");

    // Verify deletion
    let empty_entries = persistence.load_entries().expect("should succeed");
    assert_eq!(empty_entries.len(), 0);
}

#[test]
fn test_command_journal_basic_workflow() {
    // Create a journal with in-memory persistence
    let persistence = Arc::new(InMemoryJournalPersistence::new());
    let journal = CommandJournal::new(persistence, 100);

    let command = TestCommand {};
    let args = vec!["arg1".to_string(), "arg2".to_string()];

    // Record command start
    let id = journal
        .record_start(&command, &args)
        .expect("should succeed");

    // Execute the command
    let result = command.execute(&args);

    // Record command completion
    journal
        .record_completion(id.clone(), result)
        .expect("should succeed");

    // Get the entry
    let entry = journal.get_entry(id).expect("should succeed");

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
    let journal = CommandJournal::new(persistence, 100);

    let command = TestCommand {};
    let args = vec!["fail".to_string()];

    // Record command start
    let id = journal
        .record_start(&command, &args)
        .expect("should succeed");

    // Execute the command (which will fail)
    let result = command.execute(&args);

    // Record command completion
    journal
        .record_completion(id.clone(), result)
        .expect("should succeed");

    // Get the entry
    let entry = journal.get_entry(id).expect("should succeed");

    // Verify entry state
    assert_eq!(entry.state, JournalEntryState::Failed);
    assert!(entry.result.is_none());
    assert!(entry.error.is_some());
}

#[test]
fn test_find_incomplete_entries() {
    // Create a journal with in-memory persistence
    let persistence = Arc::new(InMemoryJournalPersistence::new());
    let journal = CommandJournal::new(persistence, 100);

    let command = TestCommand {};

    // Create a completed entry
    let id1 = journal
        .record_start(&command, &["success".to_string()])
        .expect("should succeed");
    journal
        .record_completion(id1, Ok("Success".to_string()))
        .expect("should succeed");

    // Create an incomplete entry
    let id2 = journal
        .record_start(&command, &["incomplete".to_string()])
        .expect("should succeed");

    // Find incomplete entries
    let incomplete = journal.find_incomplete().expect("should succeed");

    // Verify incomplete entries
    assert_eq!(incomplete.len(), 1);
    assert_eq!(incomplete[0].id, id2);
    assert_eq!(incomplete[0].state, JournalEntryState::Started);
}

#[test]
fn test_recover_incomplete_entries() {
    // Create a journal with in-memory persistence
    let persistence = Arc::new(InMemoryJournalPersistence::new());
    let journal = CommandJournal::new(persistence, 100);

    let command = TestCommand {};

    // Create an incomplete entry
    let id = journal
        .record_start(&command, &["recoverable".to_string()])
        .expect("should succeed");

    // Recover incomplete entries
    let recovery_report = journal
        .recover_incomplete(|entry| {
            assert_eq!(entry.arguments[0], "recoverable");
            Ok("Recovered successfully".to_string())
        })
        .expect("should succeed");

    // Verify recovery report
    assert_eq!(recovery_report.recovered_entries.len(), 1);
    assert_eq!(recovery_report.failed_entries.len(), 0);

    // Verify entry was updated
    let entry = journal.get_entry(id).expect("should succeed");
    assert_eq!(entry.state, JournalEntryState::Recovered);
    assert_eq!(entry.result, Some("Recovered successfully".to_string()));
}

#[test]
fn test_entry_search() {
    // Create a journal with in-memory persistence
    let persistence = Arc::new(InMemoryJournalPersistence::new());
    let journal = CommandJournal::new(persistence, 100);

    let command = TestCommand {};

    // Create various entries
    let id1 = journal
        .record_start(&command, &["search1".to_string()])
        .expect("should succeed");
    journal
        .record_completion(id1, Ok("Success 1".to_string()))
        .expect("should succeed");

    let id2 = journal
        .record_start(&command, &["search2".to_string()])
        .expect("should succeed");
    journal
        .record_completion(id2.clone(), Ok("Success 2".to_string()))
        .expect("should succeed");

    let id3 = journal
        .record_start(&command, &["fail".to_string()])
        .expect("should succeed");
    journal
        .record_completion(
            id3.clone(),
            Err(CommandError::ExecutionError("Failed".to_string())),
        )
        .expect("should succeed");

    // Search for completed entries
    let completed = journal
        .search_entries(|entry| entry.state == JournalEntryState::Completed)
        .expect("should succeed");
    assert_eq!(completed.len(), 2);

    // Search for failed entries
    let failed = journal
        .search_entries(|entry| entry.state == JournalEntryState::Failed)
        .expect("should succeed");
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].id, id3);

    // Search by argument content
    let search2 = journal
        .search_entries(|entry| entry.arguments.iter().any(|arg| arg.contains("search2")))
        .expect("should succeed");
    assert_eq!(search2.len(), 1);
    assert_eq!(search2[0].id, id2);
}

#[test]
fn test_journal_capacity() {
    // Create a journal with small capacity
    let persistence = Arc::new(InMemoryJournalPersistence::new());
    let journal = CommandJournal::new(persistence, 2);

    let command = TestCommand {};

    // Add entries to reach capacity
    let id1 = journal
        .record_start(&command, &["first".to_string()])
        .expect("should succeed");
    journal
        .record_completion(id1.clone(), Ok("First success".to_string()))
        .expect("should succeed");

    let id2 = journal
        .record_start(&command, &["second".to_string()])
        .expect("should succeed");
    journal
        .record_completion(id2.clone(), Ok("Second success".to_string()))
        .expect("should succeed");

    // Verify entries
    let entries = journal.get_entries().expect("should succeed");
    assert_eq!(entries.len(), 2);

    // Add one more entry (should evict oldest)
    let id3 = journal
        .record_start(&command, &["third".to_string()])
        .expect("should succeed");
    journal
        .record_completion(id3.clone(), Ok("Third success".to_string()))
        .expect("should succeed");

    // Verify entries (should now have id2 and id3, but not id1)
    let updated_entries = journal.get_entries().expect("should succeed");
    assert_eq!(updated_entries.len(), 2);

    // The first entry should be evicted
    let entry_ids: Vec<String> = updated_entries.iter().map(|e| e.id.clone()).collect();
    assert!(!entry_ids.contains(&id1));
    assert!(entry_ids.contains(&id2));
    assert!(entry_ids.contains(&id3));
}
