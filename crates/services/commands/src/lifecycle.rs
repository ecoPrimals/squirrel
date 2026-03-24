// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::sync::RwLock;

use crate::Command;
use crate::error::CommandError;

/// Represents the different stages in a command's lifecycle
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum LifecycleStage {
    /// Before command registration.
    Registration,

    /// The initialization stage, executed before any validation
    Initialization,

    /// The validation stage, where command parameters are validated
    Validation,

    /// The pre-execution stage, executed after validation but before execution
    PreExecution,

    /// Pre-validation stage, before command validation is performed
    PreValidation,

    /// The execution stage, where the command is actually executed
    Execution,

    /// The post-execution stage, executed after a successful execution
    PostExecution,

    /// Post-validation stage, after command validation is complete
    PostValidation,

    /// During command completion.
    Completion,

    /// During cleanup.
    Cleanup,

    /// The error handling stage, executed if an error occurs during any stage
    ErrorHandling,
}

/// Trait that defines a command lifecycle hook.
///
/// Lifecycle hooks allow hooking into different stages of command processing.
pub trait LifecycleHook: Send + Sync + std::fmt::Debug {
    /// Returns the name of the lifecycle hook.
    fn name(&self) -> &'static str;

    /// Returns the lifecycle stages this hook is interested in.
    fn stages(&self) -> Vec<LifecycleStage>;

    /// Called when a command is processed at different lifecycle stages.
    ///
    /// # Arguments
    /// * `stage` - The current lifecycle stage
    /// * `command` - The command being processed
    /// * `context` - The context for the command
    ///
    /// # Errors
    /// Returns an error if the hook fails
    fn on_stage(&self, stage: &LifecycleStage, command: &dyn Command) -> Result<(), CommandError>;

    /// Clone the hook into a new Box.
    fn clone_box(&self) -> Box<dyn LifecycleHook>;
}

/// Manager for command lifecycle stages.
#[derive(Debug)]
pub struct CommandLifecycle {
    /// List of lifecycle hooks to execute.
    hooks: RwLock<Vec<Box<dyn LifecycleHook>>>,
}

impl CommandLifecycle {
    /// Creates a new command lifecycle manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            hooks: RwLock::new(Vec::new()),
        }
    }

    /// Executes a lifecycle stage for a command
    ///
    /// # Arguments
    ///
    /// * `stage` - The lifecycle stage to execute
    /// * `command` - The command to execute the stage for
    ///
    /// # Errors
    ///
    /// Returns an error if any lifecycle hook fails during execution
    pub fn execute_stage(
        &self,
        stage: LifecycleStage,
        command: &dyn Command,
    ) -> Result<(), CommandError> {
        // Get a copy of the hooks while holding the lock
        let hooks_copy = {
            let hooks = self.hooks.read().map_err(|e| {
                CommandError::Lock(format!(
                    "Failed to acquire read lock on lifecycle hooks: {e}"
                ))
            })?;

            // Clone the hooks into a temporary vector
            hooks
                .iter()
                .map(|hook| hook.clone_box())
                .collect::<Vec<_>>()
        }; // Lock is released here

        // Execute hooks after the lock is released
        for hook in &hooks_copy {
            hook.on_stage(&stage, command)?;
        }

        Ok(())
    }

    /// Adds a lifecycle hook to the manager
    ///
    /// # Arguments
    ///
    /// * `hook` - The hook to add
    ///
    /// # Errors
    ///
    /// Returns an error if the hook cannot be added
    pub fn add_hook(&self, hook: Box<dyn LifecycleHook>) -> Result<(), CommandError> {
        let mut hooks = self.hooks.write().map_err(|e| {
            CommandError::Lock(format!(
                "Failed to acquire write lock on lifecycle hooks: {e}"
            ))
        })?;
        hooks.push(hook);
        Ok(())
    }

    /// Returns the number of hooks registered
    #[must_use]
    pub fn hooks(&self) -> usize {
        self.hooks.read().map(|h| h.len()).unwrap_or(0)
    }
}

impl Default for CommandLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CommandResult;
    use std::sync::Arc;

    #[derive(Debug)]
    struct TestCommand;

    impl crate::Command for TestCommand {
        fn name(&self) -> &str {
            "test_cmd"
        }

        fn description(&self) -> &str {
            "test"
        }

        fn execute(&self, _args: &[String]) -> CommandResult<String> {
            Ok("ok".into())
        }

        fn parser(&self) -> clap::Command {
            clap::Command::new("test_cmd")
        }

        fn clone_box(&self) -> Box<dyn crate::Command> {
            Box::new(TestCommand)
        }
    }

    #[test]
    fn test_lifecycle_stages() {
        let stage = LifecycleStage::PreExecution;
        assert_eq!(stage, LifecycleStage::PreExecution);
    }

    #[derive(Debug)]
    struct StageRecordingHook {
        name: &'static str,
        seen: std::sync::Arc<std::sync::Mutex<Vec<LifecycleStage>>>,
    }

    impl LifecycleHook for StageRecordingHook {
        fn name(&self) -> &'static str {
            self.name
        }

        fn stages(&self) -> Vec<LifecycleStage> {
            vec![
                LifecycleStage::Registration,
                LifecycleStage::Initialization,
                LifecycleStage::Validation,
            ]
        }

        fn on_stage(
            &self,
            stage: &LifecycleStage,
            _command: &dyn Command,
        ) -> Result<(), CommandError> {
            self.seen.lock().expect("should succeed").push(*stage);
            Ok(())
        }

        fn clone_box(&self) -> Box<dyn LifecycleHook> {
            Box::new(StageRecordingHook {
                name: self.name,
                seen: Arc::clone(&self.seen),
            })
        }
    }

    #[test]
    fn command_lifecycle_executes_all_hook_stages() {
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let hook = StageRecordingHook {
            name: "rec",
            seen: Arc::clone(&seen),
        };
        let lifecycle = CommandLifecycle::new();
        lifecycle.add_hook(Box::new(hook)).expect("should succeed");
        assert_eq!(lifecycle.hooks(), 1);

        let cmd = TestCommand;
        for s in [
            LifecycleStage::Registration,
            LifecycleStage::Initialization,
            LifecycleStage::Validation,
        ] {
            lifecycle.execute_stage(s, &cmd).expect("should succeed");
        }

        let got = seen.lock().expect("should succeed").clone();
        assert_eq!(got.len(), 3);
        assert_eq!(got[0], LifecycleStage::Registration);
    }

    #[test]
    fn command_lifecycle_propagates_hook_error() {
        #[derive(Debug)]
        struct FailOnValidation;

        impl LifecycleHook for FailOnValidation {
            fn name(&self) -> &'static str {
                "fail"
            }

            fn stages(&self) -> Vec<LifecycleStage> {
                vec![LifecycleStage::Validation]
            }

            fn on_stage(
                &self,
                stage: &LifecycleStage,
                _command: &dyn Command,
            ) -> Result<(), CommandError> {
                if *stage == LifecycleStage::Validation {
                    return Err(CommandError::Lifecycle("no".into()));
                }
                Ok(())
            }

            fn clone_box(&self) -> Box<dyn LifecycleHook> {
                Box::new(FailOnValidation)
            }
        }

        let lifecycle = CommandLifecycle::new();
        lifecycle
            .add_hook(Box::new(FailOnValidation))
            .expect("should succeed");
        let cmd = TestCommand;
        assert!(
            lifecycle
                .execute_stage(LifecycleStage::Validation, &cmd)
                .is_err()
        );
    }
}
