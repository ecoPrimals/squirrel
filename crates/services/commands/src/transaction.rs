// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Command Transaction System
//!
//! This module provides transaction-like execution with rollback capabilities for commands.
//! It allows multiple commands to be executed as a single unit of work, with rollback
//! functionality if any command fails.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::registry::{Command, CommandResult};

/// Error types specific to transaction operations
#[derive(Debug, Error, Clone)]
pub enum TransactionError {
    /// Error during transaction execution
    #[error("Transaction execution error: {0}")]
    ExecutionError(String),

    /// Error during transaction rollback
    #[error("Transaction rollback error: {0}")]
    RollbackError(String),

    /// Error when the transaction is empty
    #[error("Empty transaction")]
    EmptyTransaction,

    /// Error when the rollback handler is missing
    #[error("Missing rollback handler for command: {0}")]
    MissingRollbackHandler(String),

    /// Error when the transaction is in an invalid state
    #[error("Invalid transaction state: {0}")]
    InvalidState(String),
}

/// Execution result for a command in a transaction
#[derive(Debug)]
pub struct ExecutionResult {
    /// Command name
    pub command_name: String,

    /// Command arguments
    pub args: Vec<String>,

    /// Execution result
    pub result: CommandResult<String>,

    /// Execution timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Transaction state for tracking execution progress
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    /// Transaction is initialized but not started
    Initialized,

    /// Transaction is in progress
    InProgress,

    /// Transaction has completed successfully
    Completed,

    /// Transaction has failed and rollback is needed
    Failed,

    /// Transaction is rolled back
    RolledBack,
}

impl fmt::Display for TransactionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionState::Initialized => write!(f, "Initialized"),
            TransactionState::InProgress => write!(f, "InProgress"),
            TransactionState::Completed => write!(f, "Completed"),
            TransactionState::Failed => write!(f, "Failed"),
            TransactionState::RolledBack => write!(f, "RolledBack"),
        }
    }
}

/// Trait for implementing rollback handlers
pub trait RollbackHandler: Send + Sync {
    /// Get the name of the rollback handler
    fn name(&self) -> &str;

    /// Execute the rollback handler for a command
    fn rollback(&self, result: &ExecutionResult) -> Result<(), TransactionError>;
}

/// Command transaction for executing multiple commands as a single unit of work
pub struct CommandTransaction {
    /// Transaction ID
    id: Uuid,

    /// Transaction state
    state: TransactionState,

    /// Commands to execute
    commands: Vec<(Box<dyn Command>, Vec<String>)>,

    /// Executed commands with results
    executed: Vec<ExecutionResult>,

    /// Rollback handlers for commands
    rollback_handlers: HashMap<String, Arc<dyn RollbackHandler>>,
}

impl Default for CommandTransaction {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandTransaction {
    /// Creates a new command transaction
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            state: TransactionState::Initialized,
            commands: Vec::new(),
            executed: Vec::new(),
            rollback_handlers: HashMap::new(),
        }
    }

    /// Get the transaction ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get the transaction state
    pub fn state(&self) -> TransactionState {
        self.state
    }

    /// Add a command to the transaction
    pub fn add_command(
        &mut self,
        command: Box<dyn Command>,
        args: Vec<String>,
    ) -> Result<(), TransactionError> {
        if self.state != TransactionState::Initialized {
            return Err(TransactionError::InvalidState(format!(
                "Cannot add command in state {}",
                self.state
            )));
        }

        self.commands.push((command, args));
        Ok(())
    }

    /// Add a rollback handler for a command
    pub fn add_rollback_handler(
        &mut self,
        command_name: &str,
        handler: Arc<dyn RollbackHandler>,
    ) -> Result<(), TransactionError> {
        if self.state != TransactionState::Initialized {
            return Err(TransactionError::InvalidState(format!(
                "Cannot add rollback handler in state {}",
                self.state
            )));
        }

        self.rollback_handlers
            .insert(command_name.to_string(), handler);
        Ok(())
    }

    /// Execute all commands in the transaction
    pub async fn execute(&mut self) -> Result<Vec<String>, TransactionError> {
        if self.commands.is_empty() {
            return Err(TransactionError::EmptyTransaction);
        }

        if self.state != TransactionState::Initialized {
            return Err(TransactionError::InvalidState(format!(
                "Cannot execute transaction in state {}",
                self.state
            )));
        }

        self.state = TransactionState::InProgress;
        let mut outputs = Vec::new();

        info!("Starting transaction {}", self.id);

        for (idx, (command, args)) in self.commands.iter().enumerate() {
            let command_name = command.name().to_string();
            debug!(
                "Executing command {}/{}: {} with args {:?}",
                idx + 1,
                self.commands.len(),
                command_name,
                args
            );

            let result = command.execute(args);
            let timestamp = chrono::Utc::now();

            // Handle command execution result safely
            if let Err(command_error) = &result {
                error!("Command {} failed: {:?}", command_name, command_error);

                self.executed.push(ExecutionResult {
                    command_name: command_name.clone(),
                    args: args.clone(),
                    result: result.clone(),
                    timestamp,
                });

                self.state = TransactionState::Failed;

                // Rollback previously executed commands
                if let Err(err) = self.rollback().await {
                    error!("Transaction rollback failed: {}", err);
                    return Err(TransactionError::ExecutionError(format!(
                        "Command {command_name} failed and rollback failed: {err}"
                    )));
                }

                return Err(TransactionError::ExecutionError(format!(
                    "Command {command_name} failed: {command_error}"
                )));
            }

            let output = match &result {
                Ok(success_output) => success_output.to_string(),
                Err(_) => {
                    // This shouldn't happen due to the check above, but let's be safe
                    return Err(TransactionError::ExecutionError(format!(
                        "Unexpected error state for command {command_name}"
                    )));
                }
            };
            outputs.push(output);

            debug!("Command {} succeeded", command_name);

            // Store the execution result
            self.executed.push(ExecutionResult {
                command_name,
                args: args.clone(),
                result,
                timestamp,
            });
        }

        self.state = TransactionState::Completed;
        info!("Transaction {} completed successfully", self.id);

        Ok(outputs)
    }

    /// Manually rollback the transaction
    pub async fn rollback(&mut self) -> Result<(), TransactionError> {
        if self.executed.is_empty() {
            warn!("Nothing to rollback in transaction {}", self.id);
            return Ok(());
        }

        if self.state == TransactionState::RolledBack {
            warn!("Transaction {} already rolled back", self.id);
            return Ok(());
        }

        info!("Rolling back transaction {}", self.id);

        // Rollback in reverse order
        for result in self.executed.iter().rev() {
            let command_name = &result.command_name;

            debug!("Rolling back command: {}", command_name);

            if let Some(handler) = self.rollback_handlers.get(command_name) {
                if let Err(err) = handler.rollback(result) {
                    error!("Failed to rollback command {}: {}", command_name, err);
                    return Err(TransactionError::RollbackError(format!(
                        "Failed to rollback command {command_name}: {err}"
                    )));
                }
            } else {
                warn!(
                    "No rollback handler for command: {}. Skipping rollback.",
                    command_name
                );
            }
        }

        self.state = TransactionState::RolledBack;
        info!("Transaction {} rolled back successfully", self.id);

        Ok(())
    }

    /// Get the executed commands with results
    pub fn executed_commands(&self) -> &[ExecutionResult] {
        &self.executed
    }
}

/// Transaction manager for tracking and managing transactions
pub struct TransactionManager {
    /// Active transactions
    transactions: Mutex<HashMap<Uuid, Arc<Mutex<CommandTransaction>>>>,
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionManager {
    /// Creates a new transaction manager
    pub fn new() -> Self {
        Self {
            transactions: Mutex::new(HashMap::new()),
        }
    }

    /// Create a new transaction and register it with the manager
    pub fn create_transaction(&self) -> Result<Arc<Mutex<CommandTransaction>>, TransactionError> {
        let transaction = CommandTransaction::new();
        let id = transaction.id();
        let transaction = Arc::new(Mutex::new(transaction));

        let mut transactions = self.transactions.lock().map_err(|e| {
            error!(
                "Failed to acquire transaction manager lock for creation: {}",
                e
            );
            TransactionError::InvalidState(format!(
                "Failed to acquire transaction manager lock: {e}"
            ))
        })?;

        transactions.insert(id, transaction.clone());

        Ok(transaction)
    }

    /// Get a transaction by ID
    pub fn get_transaction(&self, id: Uuid) -> Option<Arc<Mutex<CommandTransaction>>> {
        match self.transactions.lock() {
            Ok(transactions) => transactions.get(&id).cloned(),
            Err(e) => {
                error!(
                    "Failed to acquire transaction manager lock for retrieval: {}",
                    e
                );
                None
            }
        }
    }

    /// Remove a transaction by ID
    pub fn remove_transaction(&self, id: Uuid) -> Result<(), TransactionError> {
        let mut transactions = self.transactions.lock().map_err(|e| {
            error!(
                "Failed to acquire transaction manager lock for removal: {}",
                e
            );
            TransactionError::InvalidState(format!(
                "Failed to acquire transaction manager lock: {e}"
            ))
        })?;

        if transactions.remove(&id).is_none() {
            return Err(TransactionError::InvalidState(format!(
                "Transaction {id} not found"
            )));
        }

        Ok(())
    }

    /// Get all active transactions
    pub fn list_transactions(&self) -> Vec<Uuid> {
        match self.transactions.lock() {
            Ok(transactions) => transactions.keys().cloned().collect(),
            Err(e) => {
                error!(
                    "Failed to acquire transaction manager lock for listing: {}",
                    e
                );
                Vec::new() // Return empty list on lock failure
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CommandError;
    use std::sync::Arc;

    // Mock command for testing
    struct MockCommand {
        name: String,
        success: bool,
    }

    impl Command for MockCommand {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "Mock command for testing"
        }

        fn execute(&self, _args: &[String]) -> CommandResult<String> {
            if self.success {
                Ok(format!("{} executed successfully", self.name))
            } else {
                Err(CommandError::ExecutionError(format!(
                    "{} failed",
                    self.name
                )))
            }
        }

        fn parser(&self) -> clap::Command {
            clap::Command::new("mock-command").about("Mock command for testing")
        }

        fn clone_box(&self) -> Box<dyn Command> {
            Box::new(Self {
                name: self.name.clone(),
                success: self.success,
            })
        }
    }

    // Mock rollback handler for testing
    struct MockRollbackHandler {
        name: String,
        success: bool,
    }

    impl RollbackHandler for MockRollbackHandler {
        fn name(&self) -> &str {
            &self.name
        }

        fn rollback(&self, _result: &ExecutionResult) -> Result<(), TransactionError> {
            if self.success {
                Ok(())
            } else {
                Err(TransactionError::RollbackError(format!(
                    "{} rollback failed",
                    self.name
                )))
            }
        }
    }

    #[tokio::test]
    async fn test_successful_transaction() {
        let mut transaction = CommandTransaction::new();

        // Add commands
        transaction
            .add_command(
                Box::new(MockCommand {
                    name: "command1".to_string(),
                    success: true,
                }),
                vec!["arg1".to_string(), "arg2".to_string()],
            )
            .expect("Failed to add command1 to transaction in test");

        transaction
            .add_command(
                Box::new(MockCommand {
                    name: "command2".to_string(),
                    success: true,
                }),
                vec!["arg3".to_string(), "arg4".to_string()],
            )
            .expect("Failed to add command2 to transaction in test");

        // Add rollback handlers
        transaction
            .add_rollback_handler(
                "command1",
                Arc::new(MockRollbackHandler {
                    name: "command1_rollback".to_string(),
                    success: true,
                }),
            )
            .expect("Failed to add rollback handler for command1 in test");

        transaction
            .add_rollback_handler(
                "command2",
                Arc::new(MockRollbackHandler {
                    name: "command2_rollback".to_string(),
                    success: true,
                }),
            )
            .expect("Failed to add rollback handler for command2 in test");

        // Execute transaction
        let result = transaction.execute().await;

        // Check results
        assert!(result.is_ok());
        let outputs = result.expect("Transaction execution should succeed in test");
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0], "command1 executed successfully");
        assert_eq!(outputs[1], "command2 executed successfully");
        assert_eq!(transaction.state(), TransactionState::Completed);
    }

    #[tokio::test]
    async fn test_failed_transaction() {
        let mut transaction = CommandTransaction::new();

        // Add commands
        transaction
            .add_command(
                Box::new(MockCommand {
                    name: "command1".to_string(),
                    success: true,
                }),
                vec!["arg1".to_string(), "arg2".to_string()],
            )
            .expect("Failed to add command1 to transaction in test");

        transaction
            .add_command(
                Box::new(MockCommand {
                    name: "command2".to_string(),
                    success: false,
                }),
                vec!["arg3".to_string(), "arg4".to_string()],
            )
            .expect("Failed to add command2 to transaction in test");

        // Add rollback handlers
        transaction
            .add_rollback_handler(
                "command1",
                Arc::new(MockRollbackHandler {
                    name: "command1_rollback".to_string(),
                    success: true,
                }),
            )
            .expect("Failed to add rollback handler for command1 in test");

        // Execute transaction
        let result = transaction.execute().await;

        // Check results
        assert!(result.is_err());
        assert_eq!(transaction.state(), TransactionState::RolledBack);
        assert_eq!(transaction.executed_commands().len(), 2);
    }

    #[tokio::test]
    async fn test_transaction_manager() {
        let manager = TransactionManager::new();

        // Create transactions
        let transaction1 = manager
            .create_transaction()
            .expect("Failed to create transaction1 in test");
        let transaction2 = manager
            .create_transaction()
            .expect("Failed to create transaction2 in test");

        // Get transaction IDs
        let id1 = transaction1
            .lock()
            .expect("Failed to lock transaction1 in test")
            .id();
        let id2 = transaction2
            .lock()
            .expect("Failed to lock transaction2 in test")
            .id();

        // Get transactions
        let retrieved1 = manager.get_transaction(id1);
        let retrieved2 = manager.get_transaction(id2);

        assert!(retrieved1.is_some());
        assert!(retrieved2.is_some());

        // List transactions
        let tx_list = manager.list_transactions();
        assert_eq!(tx_list.len(), 2);
        assert!(tx_list.contains(&id1));
        assert!(tx_list.contains(&id2));

        // Remove a transaction
        manager
            .remove_transaction(id1)
            .expect("Failed to remove transaction1 in test");

        // List transactions again
        let tx_list = manager.list_transactions();
        assert_eq!(tx_list.len(), 1);
        assert!(tx_list.contains(&id2));

        // Try to get the removed transaction
        let retrieved1 = manager.get_transaction(id1);
        assert!(retrieved1.is_none());
    }
}
