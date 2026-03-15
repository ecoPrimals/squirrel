// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use crate::journal::{
    CommandJournal, FileJournalPersistence, InMemoryJournalPersistence, 
    JournalEntryState, JournalError
};
use crate::{Command, CommandError, CommandResult};
use clap::Command as ClapCommand;
use std::sync::Arc;
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[derive(Clone)]
struct TestCommand;

impl Command for TestCommand {
    fn name(&self) -> &str {
        "test-command"
    }

    fn description(&self) -> &str {
        "A test command for journal testing"
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
        ClapCommand::new("test-command")
            .about("A test command for journal testing")
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(Self {})
    }
}

// Integration test for CommandError and JournalError interactions
#[tokio::test]
async fn test_journal_error_conversion() {
    // Create a specific journal error
    let journal_error = JournalError::EntryNotFound("test-id".to_string());
    
    // Convert to CommandError
    let command_error: CommandError = journal_error.clone().into();
    
    // Validate the error conversion
    match command_error {
        CommandError::JournalError(e) => {
            match e {
                JournalError::EntryNotFound(id) => assert_eq!(id, "test-id"),
                _ => panic!("Unexpected journal error type"),
            }
        },
        _ => panic!("Error was not converted to CommandError::JournalError"),
    }
}

#[tokio::test]
async fn test_journal_integration() {
    // Create temporary file for journal persistence
    let journal_file = format!("test_journal_{}.json", Uuid::new_v4());
    
    // Create a new journal with file persistence
    let persistence = Arc::new(FileJournalPersistence::new(&journal_file));
    let journal = CommandJournal::new(persistence, 100);
    
    // Create a test command
    let command = TestCommand {};
    
    // Record a command execution
    let id = journal.record_start(&command, &["test-arg".to_string()]).unwrap();
    
    // Execute the command
    let result = command.execute(&["test-arg".to_string()]);
    
    // Record the completion
    journal.record_completion(id.clone(), result).unwrap();
    
    // Retrieve and verify the entry
    let entry = journal.get_entry(id).unwrap();
    assert_eq!(entry.command_name, "test-command");
    assert_eq!(entry.arguments, vec!["test-arg".to_string()]);
    assert_eq!(entry.state, JournalEntryState::Completed);
    assert_eq!(entry.result, Some("Success with args: [\"test-arg\"]".to_string()));
    
    // Clean up the file
    if Path::new(&journal_file).exists() {
        fs::remove_file(&journal_file).unwrap();
    }
}

#[tokio::test]
async fn test_journal_incomplete_recovery() {
    // Create in-memory persistence for testing
    let persistence = Arc::new(InMemoryJournalPersistence::new());
    let journal = CommandJournal::new(persistence, 100);
    
    // Create a test command
    let command = TestCommand {};
    
    // Record a command start but don't complete it
    let id = journal.record_start(&command, &["recoverable".to_string()]).unwrap();
    
    // Find incomplete entries
    let incomplete = journal.find_incomplete().unwrap();
    assert_eq!(incomplete.len(), 1);
    assert_eq!(incomplete[0].id, id);
    
    // Recover incomplete entries
    let report = journal.recover_incomplete(|entry| {
        assert_eq!(entry.command_name, "test-command");
        assert_eq!(entry.arguments[0], "recoverable");
        
        // Simulate recovery by re-executing the command
        let cmd = TestCommand {};
        cmd.execute(&entry.arguments)
    }).unwrap();
    
    // Verify recovery report
    assert_eq!(report.recovered_entries.len(), 1);
    assert_eq!(report.failed_entries.len(), 0);
    
    // Verify the entry is now marked as recovered
    let entry = journal.get_entry(id).unwrap();
    assert_eq!(entry.state, JournalEntryState::Recovered);
    assert_eq!(entry.result, Some("Success with args: [\"recoverable\"]".to_string()));
} 