// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

use std::sync::Arc;
use std::error::Error;
use clap::{Command as ClapCommand, Arg};
use squirrel_commands::{CommandError, CommandResult, Command};
use squirrel_commands::transaction::{CommandTransaction, RollbackHandler, ExecutionResult, TransactionError};

// Example file system command
struct FileWriteCommand;

impl Command for FileWriteCommand {
    fn name(&self) -> &str {
        "file-write"
    }

    fn description(&self) -> &str {
        "Write content to a file"
    }

    fn execute(&self, args: &[String]) -> CommandResult<String> {
        if args.len() < 2 {
            return Err(CommandError::ExecutionError(
                "Usage: file-write <filename> <content>".to_string(),
            ));
        }

        let filename = &args[0];
        let content = &args[1];

        std::fs::write(filename, content).map_err(|e| {
            CommandError::ExecutionError(format!("Failed to write to file: {}", e))
        })?;

        Ok(format!("Successfully wrote to file: {}", filename))
    }

    fn parser(&self) -> ClapCommand {
        ClapCommand::new("file-write")
            .about("Write content to a file")
            .arg(Arg::new("filename").required(true))
            .arg(Arg::new("content").required(true))
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(Self {})
    }
}

// Example database command
struct DbWriteCommand;

impl Command for DbWriteCommand {
    fn name(&self) -> &str {
        "db-write"
    }

    fn description(&self) -> &str {
        "Write data to a database"
    }

    fn execute(&self, args: &[String]) -> CommandResult<String> {
        if args.len() < 2 {
            return Err(CommandError::ExecutionError(
                "Usage: db-write <key> <value>".to_string(),
            ));
        }

        let key = &args[0];
        let value = &args[1];

        // Simulate database write
        // In a real implementation, this would write to a database
        println!("Writing to database: {} = {}", key, value);

        // For demo purposes, we'll simulate a failure if the key is "fail"
        if key == "fail" {
            return Err(CommandError::ExecutionError(
                "Simulated database failure".to_string(),
            ));
        }

        Ok(format!("Successfully wrote to database: {} = {}", key, value))
    }

    fn parser(&self) -> ClapCommand {
        ClapCommand::new("db-write")
            .about("Write data to a database")
            .arg(Arg::new("key").required(true))
            .arg(Arg::new("value").required(true))
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(Self {})
    }
}

// Rollback handler for file write command
struct FileWriteRollbackHandler;

impl RollbackHandler for FileWriteRollbackHandler {
    fn name(&self) -> &str {
        "file-write-rollback"
    }

    fn rollback(&self, result: &ExecutionResult) -> Result<(), TransactionError> {
        if result.args.is_empty() {
            return Ok(());
        }

        let filename = &result.args[0];
        println!("Rolling back file write: {}", filename);

        // Delete the file to rollback the write operation
        if std::path::Path::new(filename).exists() {
            std::fs::remove_file(filename).map_err(|e| {
                TransactionError::RollbackError(format!("Failed to delete file: {}", e))
            })?;
        }

        Ok(())
    }
}

// Rollback handler for database write command
struct DbWriteRollbackHandler;

impl RollbackHandler for DbWriteRollbackHandler {
    fn name(&self) -> &str {
        "db-write-rollback"
    }

    fn rollback(&self, result: &ExecutionResult) -> Result<(), TransactionError> {
        if result.args.len() < 2 {
            return Ok(());
        }

        let key = &result.args[0];
        println!("Rolling back database write: {}", key);

        // In a real implementation, this would delete the entry from the database
        // or restore the previous value

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize a new transaction
    let mut transaction = CommandTransaction::new();
    println!("Transaction ID: {}", transaction.id());

    // Add commands to the transaction
    let temp_file = format!("temp_{}.txt", transaction.id());
    
    // Add file write command
    transaction.add_command(
        Box::new(FileWriteCommand {}),
        vec![temp_file.clone(), "Hello, Transaction!".to_string()],
    )?;

    // Add database write command - this will succeed
    transaction.add_command(
        Box::new(DbWriteCommand {}),
        vec!["user_123".to_string(), "John Doe".to_string()],
    )?;

    // Register rollback handlers
    transaction.add_rollback_handler("file-write", Arc::new(FileWriteRollbackHandler {}))?;
    transaction.add_rollback_handler("db-write", Arc::new(DbWriteRollbackHandler {}))?;

    // Execute the transaction (should succeed)
    println!("\n=== Executing successful transaction ===");
    match transaction.execute().await {
        Ok(outputs) => {
            println!("Transaction completed successfully");
            for (i, output) in outputs.iter().enumerate() {
                println!("Command {}: {}", i+1, output);
            }
        }
        Err(e) => {
            println!("Transaction failed: {}", e);
        }
    }

    // Clean up test file
    if std::path::Path::new(&temp_file).exists() {
        std::fs::remove_file(&temp_file)?;
    }

    // Create a new transaction that will fail
    let mut failing_transaction = CommandTransaction::new();
    println!("\n=== Executing failing transaction ===");
    println!("Transaction ID: {}", failing_transaction.id());

    // Add file write command
    let temp_file2 = format!("temp_{}.txt", failing_transaction.id());
    failing_transaction.add_command(
        Box::new(FileWriteCommand {}),
        vec![temp_file2.clone(), "This will be rolled back".to_string()],
    )?;

    // Add database write command that will fail
    failing_transaction.add_command(
        Box::new(DbWriteCommand {}),
        vec!["fail".to_string(), "This will fail".to_string()],
    )?;

    // Register rollback handlers
    failing_transaction.add_rollback_handler("file-write", Arc::new(FileWriteRollbackHandler {}))?;
    failing_transaction.add_rollback_handler("db-write", Arc::new(DbWriteRollbackHandler {}))?;

    // Execute the transaction (should fail and rollback)
    match failing_transaction.execute().await {
        Ok(_) => {
            println!("Transaction completed successfully (unexpected)");
        }
        Err(e) => {
            println!("Transaction failed as expected: {}", e);
            println!("Transaction state: {}", failing_transaction.state());
            
            // Check if the file was rolled back (should not exist)
            if std::path::Path::new(&temp_file2).exists() {
                println!("Error: File was not rolled back");
                std::fs::remove_file(&temp_file2)?;
            } else {
                println!("Success: File was rolled back");
            }
        }
    }

    Ok(())
} 