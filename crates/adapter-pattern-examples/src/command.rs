// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Universal pattern mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Command traits and basic implementations.
//!
//! [`Command`] uses `impl Future + Send` (no `async_trait`).
//! [`DynCommand`] provides the object-safe bridge for registries.

use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use crate::CommandResult;

/// Command trait representing a command interface.
///
/// Async work is expressed as `impl Future<Output = _> + Send` (not `async_trait`) so the
/// [`DynCommand`] blanket impl is `Send` and registries can store `Arc<dyn DynCommand>`.
pub trait Command: Send + Sync + Debug {
    /// Get the command name
    fn name(&self) -> &str;

    /// Get the command description
    fn description(&self) -> &str;

    /// Execute the command with the given arguments
    fn execute(
        &self,
        args: Vec<String>,
    ) -> impl std::future::Future<Output = CommandResult<String>> + Send + '_;
}

/// Object-safe command trait for registries (`Arc<dyn DynCommand>`).
pub trait DynCommand: Send + Sync + Debug {
    /// Get the command name
    fn name(&self) -> &str;

    /// Get the command description
    fn description(&self) -> &str;

    /// Execute the command with the given arguments
    fn execute(
        &self,
        args: Vec<String>,
    ) -> Pin<Box<dyn Future<Output = CommandResult<String>> + Send + '_>>;
}

impl<T: Command + Send + Sync> DynCommand for T {
    fn name(&self) -> &str {
        Command::name(self)
    }

    fn description(&self) -> &str {
        Command::description(self)
    }

    fn execute(
        &self,
        args: Vec<String>,
    ) -> Pin<Box<dyn Future<Output = CommandResult<String>> + Send + '_>> {
        Box::pin(Command::execute(self, args))
    }
}

/// Test command for examples and testing.
#[derive(Debug, Clone)]
pub struct TestCommand {
    name: String,
    description: String,
    result: String,
}

impl TestCommand {
    /// Create a new test command.
    #[must_use]
    pub fn new(name: &str, description: &str, result: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            result: result.to_string(),
        }
    }
}

impl Command for TestCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn execute(
        &self,
        args: Vec<String>,
    ) -> impl std::future::Future<Output = CommandResult<String>> + Send + '_ {
        let result = self.result.clone();
        async move {
            if args.is_empty() {
                Ok(result)
            } else {
                Ok(format!("{result} with args: {args:?}"))
            }
        }
    }
}
