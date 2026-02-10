// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

use std::sync::Arc;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

// Import the commands crate directly from the workspace
use squirrel_commands::{Command, CommandError, CommandResult};
use squirrel_commands::journal::{CommandJournal, FileJournalPersistence, JournalEntry, JournalEntryState};
use clap::{Arg, Command as ClapCommand};

// A simple command that either succeeds or fails based on input
#[derive(Clone)]
struct ExampleCommand;

impl Command for ExampleCommand {
    fn name(&self) -> &str {
        "example"
    }

    fn description(&self) -> &str {
        "Example command for journaling demonstration"
    }

    fn execute(&self, args: &[String]) -> CommandResult<String> {
        // Wait a bit to simulate work
        thread::sleep(Duration::from_millis(500));
        
        // Succeed or fail based on the first argument
        if args.is_empty() || args[0] == "succeed" {
            Ok(format!("Command executed successfully with args: {:?}", args))
        } else if args[0] == "fail" {
            Err(CommandError::ExecutionError(format!(
                "Command execution failed with args: {:?}", 
                args
            )))
        } else if args[0] == "panic" {
            // This will be an incomplete entry that needs recovery
            panic!("Command panicked!");
        } else {
            Ok(format!("Command executed with args: {:?}", args))
        }
    }

    fn parser(&self) -> ClapCommand {
        // Fix lifetime issues by using static strings
        ClapCommand::new("example")
            .about("Example command for journaling demonstration")
            .arg(Arg::new("action")
                .help("Action to perform (succeed, fail, panic, or anything else)")
                .index(1)
                .required(false))
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(Self {})
    }
}

// Simple registry implementation since we can't use the private one
struct SimpleRegistry {
    commands: Vec<Box<dyn Command>>,
}

impl SimpleRegistry {
    fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    fn register_command(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
    }

    fn get_command(&self, name: &str) -> Option<&Box<dyn Command>> {
        self.commands.iter().find(|cmd| cmd.name() == name)
    }
}

fn main() {
    println!("Command Journaling System Example");
    println!("=================================\n");
    
    // Create a journal file in a temp location
    let journal_path = Path::new("example_journal.json");
    let persistence = Arc::new(FileJournalPersistence::new(&journal_path));
    let journal = CommandJournal::new(persistence, 100);
    
    // Create a command registry and add our example command
    let mut registry = SimpleRegistry::new();
    registry.register_command(Box::new(ExampleCommand {}));
    
    println!("1. Basic command execution with journaling\n");
    execute_with_journal(&registry, &journal, "example", &["succeed"]);
    
    println!("\n2. Failed command execution with journaling\n");
    execute_with_journal(&registry, &journal, "example", &["fail"]);
    
    println!("\n3. Simulating incomplete commands\n");
    // We'll record the start of a command but not its completion
    let cmd = registry.get_command("example").unwrap();
    let incomplete_args = vec!["incomplete".to_string()];
    let incomplete_id = journal.record_start(cmd.as_ref(), &incomplete_args).unwrap();
    println!("Created incomplete journal entry: {}", incomplete_id);
    
    // Try to simulate a crash by not calling record_completion
    
    println!("\n4. Finding incomplete entries\n");
    let incomplete_entries = journal.find_incomplete().unwrap();
    println!("Found {} incomplete entries:", incomplete_entries.len());
    for entry in &incomplete_entries {
        println!("  - ID: {}, Command: {}, Args: {:?}, State: {}",
            entry.id, entry.command_name, entry.arguments, entry.state);
    }
    
    println!("\n5. Recovering incomplete entries\n");
    let recovery_report = journal.recover_incomplete(|entry| {
        println!("Recovering command: {} with args: {:?}", entry.command_name, entry.arguments);
        
        // For this example, we'll just succeed all recovery attempts
        Ok(format!("Recovery succeeded for command: {}", entry.command_name))
    }).unwrap();
    
    println!("Recovery report:");
    println!("  - Processed: {}", recovery_report.processed);
    println!("  - Recovered: {}", recovery_report.recovered);
    println!("  - Failed: {}", recovery_report.failed);
    
    println!("\n6. Searching journal entries\n");
    
    // Search for successful entries
    let successful = journal.search_entries(|e| e.state == JournalEntryState::Completed).unwrap();
    println!("Found {} successful entries", successful.len());
    
    // Search for failed entries
    let failed = journal.search_entries(|e| e.state == JournalEntryState::Failed).unwrap();
    println!("Found {} failed entries", failed.len());
    
    // Search for recovered entries
    let recovered = journal.search_entries(|e| e.state == JournalEntryState::Recovered).unwrap();
    println!("Found {} recovered entries", recovered.len());
    
    // Search by command argument content
    let with_fail_arg = journal.search_entries(|e| 
        e.arguments.iter().any(|arg| arg == "fail")
    ).unwrap();
    println!("Found {} entries with 'fail' argument", with_fail_arg.len());
    
    println!("\n7. Journal entries summary\n");
    let all_entries = journal.get_entries().unwrap();
    for entry in all_entries {
        print_entry_summary(&entry);
    }
    
    // Clean up the journal file
    fs::remove_file(journal_path).ok();
    println!("\nJournal file removed. Example completed.");
}

// Helper function to execute a command with journaling
fn execute_with_journal(registry: &SimpleRegistry, journal: &CommandJournal, cmd_name: &str, args: &[&str]) {
    let cmd = registry.get_command(cmd_name).unwrap();
    let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    
    // Record command start
    let journal_id = journal.record_start(cmd.as_ref(), &args_vec).unwrap();
    println!("Starting command '{}' with args: {:?}", cmd_name, args);
    println!("Journal entry created with ID: {}", journal_id);
    
    // Execute the command
    let result = match cmd.execute(&args_vec) {
        Ok(output) => {
            println!("Command succeeded: {}", output);
            Ok(output)
        }
        Err(err) => {
            println!("Command failed: {}", err);
            Err(err)
        }
    };
    
    // Record command completion
    journal.record_completion(journal_id.clone(), result).unwrap();
    println!("Command execution recorded in journal");
    
    // Get the entry to verify
    let entry = journal.get_entry(journal_id).unwrap();
    println!("Entry state: {}", entry.state);
}

// Helper function to print a summary of a journal entry
fn print_entry_summary(entry: &JournalEntry) {
    println!("Entry ID: {}", entry.id);
    println!("  Command: {}", entry.command_name);
    println!("  Arguments: {:?}", entry.arguments);
    println!("  State: {}", entry.state);
    println!("  Timestamp: {}", entry.timestamp);
    
    if let Some(result) = &entry.result {
        println!("  Result: {}", result);
    }
    
    if let Some(error) = &entry.error {
        println!("  Error: {}", error);
    }
    
    if let Some(time) = entry.execution_time {
        println!("  Execution time: {}ms", time);
    }
    
    println!("  Retry count: {}", entry.retry_count);
    println!();
} 