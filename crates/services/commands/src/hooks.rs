// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;

use tracing::{debug, error};

use crate::CommandResult;
use crate::error::CommandError;
use crate::history::CommandHistory;
use crate::validation::CommandValidator;
use crate::{Command, lifecycle::LifecycleHook, lifecycle::LifecycleStage};

/// A trait for command hooks that can be executed during command lifecycle stages
pub trait Hook: Send + Sync {
    /// Returns the name of the hook.
    fn name(&self) -> &'static str;

    /// Returns the description of the hook.
    fn description(&self) -> &'static str;

    /// Executes the hook
    ///
    /// # Arguments
    ///
    /// * `command` - The command being executed
    ///
    /// # Errors
    ///
    /// Returns an error if the hook fails to execute
    fn execute(&self, command: &dyn Command) -> Result<(), CommandError>;
}

/// Context for hook execution.
///
/// This struct contains metadata about the command and stage
/// being processed by a hook.
pub struct HookContext {
    /// Name of the command being processed
    command_name: String,
    /// Additional hook data
    data: RwLock<HashMap<String, String>>,
}

/// Type alias for a hook function that returns a Result
type HookFunction = Box<dyn Fn() -> Result<(), CommandError>>;

/// Type alias for a map of hook names to their implementations
type HookMap = HashMap<String, HookFunction>;

/// A registry for managing command hooks
pub struct HookRegistry {
    /// Map of hook names to their implementations
    hooks: HookMap,
    /// Shared context for hooks to store and retrieve data
    context: RwLock<HashMap<String, String>>,
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HookRegistry {
    /// Creates a new hook registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
            context: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a new hook with the given name
    ///
    /// # Arguments
    /// * `name` - The name of the hook
    /// * `hook` - The hook function to register
    ///
    /// # Errors
    /// Returns an error if a hook with the given name already exists
    pub fn register<F>(&mut self, name: String, hook: F) -> Result<(), CommandError>
    where
        F: Fn() -> Result<(), CommandError> + 'static,
    {
        if self.hooks.contains_key(&name) {
            return Err(CommandError::Hook(format!("Hook '{name}' already exists")));
        }

        self.hooks.insert(name, Box::new(hook));
        Ok(())
    }

    /// Executes all registered hooks
    ///
    /// # Errors
    /// Returns an error if any hook fails to execute
    pub fn execute_hooks(&self) -> Result<(), CommandError> {
        for (name, hook) in &self.hooks {
            hook().map_err(|e| CommandError::Hook(format!("Hook '{name}' failed: {e}")))?;
        }
        Ok(())
    }

    /// Sets a value in the shared context
    ///
    /// # Arguments
    /// * `key` - The key to set
    /// * `value` - The value to set
    ///
    /// # Errors
    /// Returns an error if unable to acquire write lock on context
    pub fn set_context_data(&self, key: &str, value: &str) -> Result<(), CommandError> {
        let mut context = self.context.write().map_err(|_| {
            CommandError::Lock("Failed to acquire write lock on context".to_string())
        })?;
        context.insert(key.to_string(), value.to_string());
        Ok(())
    }

    /// Gets a value from the shared context
    ///
    /// # Arguments
    /// * `key` - The key to get
    ///
    /// # Returns
    /// * `Ok(Some(String))` if the value exists
    /// * `Ok(None)` if the value does not exist
    ///
    /// # Errors
    /// Returns an error if unable to acquire read lock on context
    pub fn get_context_data(&self, key: &str) -> Result<Option<String>, CommandError> {
        let context = self.context.read().map_err(|_| {
            CommandError::Lock("Failed to acquire read lock on context".to_string())
        })?;
        Ok(context.get(key).cloned())
    }
}

/// Hook that logs command execution events with descriptive messages
pub struct LoggingHook {
    /// Name of the hook for identification
    name: String,
    /// Description of the hook's purpose
    description: String,
}

impl LoggingHook {
    /// Creates a new logging hook
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: "logging".to_string(),
            description: "Logs command execution stages".to_string(),
        }
    }
}

impl Hook for LoggingHook {
    fn name(&self) -> &'static str {
        "logging"
    }

    fn description(&self) -> &'static str {
        "Logs command execution stages"
    }

    fn execute(&self, command: &dyn Command) -> Result<(), CommandError> {
        println!("Executing command '{}'", command.name());
        Ok(())
    }
}

impl Default for LoggingHook {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook that collects and records command execution metrics
pub struct MetricsHook {
    /// Name of the hook for identification
    name: String,
    /// Description of the hook's purpose
    description: String,
}

impl MetricsHook {
    /// Creates a new metrics hook
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: "metrics".to_string(),
            description: "Collects command execution metrics".to_string(),
        }
    }
}

impl Hook for MetricsHook {
    fn name(&self) -> &'static str {
        "metrics"
    }

    fn description(&self) -> &'static str {
        "Collects command execution metrics"
    }

    fn execute(&self, command: &dyn Command) -> Result<(), CommandError> {
        println!("Metrics - Command: {}", command.name());
        Ok(())
    }
}

impl Default for MetricsHook {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook that measures and records command execution timing
pub struct TimingHook {
    /// Start time of the hook execution
    start_time: RwLock<Option<Instant>>,
}

impl TimingHook {
    /// Creates a new timing hook
    #[must_use]
    pub fn new() -> Self {
        Self {
            start_time: RwLock::new(None),
        }
    }
}

impl Hook for TimingHook {
    fn name(&self) -> &'static str {
        "timing"
    }

    fn description(&self) -> &'static str {
        "Measures command execution time"
    }

    fn execute(&self, command: &dyn Command) -> Result<(), CommandError> {
        let mut start_time = self.start_time.write().map_err(|_| {
            CommandError::Lock("Failed to acquire write lock on start time".to_string())
        })?;

        match *start_time {
            None => {
                *start_time = Some(Instant::now());
            }
            Some(start) => {
                let duration = start.elapsed();
                println!("Command '{}' took {:?}", command.name(), duration);
                *start_time = None;
            }
        }

        Ok(())
    }
}

impl Default for TimingHook {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook for validating command arguments
#[derive(Debug, Clone)]
pub struct ArgumentValidationHook {
    /// The validator component that performs argument validation
    validator: Arc<RwLock<CommandValidator>>,
}

impl ArgumentValidationHook {
    /// Creates a new `ArgumentValidationHook`
    #[must_use]
    pub fn new() -> Self {
        Self {
            validator: Arc::new(RwLock::new(CommandValidator::new())),
        }
    }
}

impl LifecycleHook for ArgumentValidationHook {
    fn name(&self) -> &'static str {
        "argument_validation"
    }

    fn stages(&self) -> Vec<LifecycleStage> {
        vec![LifecycleStage::Validation]
    }

    fn on_stage(&self, stage: &LifecycleStage, command: &dyn Command) -> Result<(), CommandError> {
        if *stage == LifecycleStage::Validation {
            let validator = self
                .validator
                .read()
                .map_err(|e| CommandError::Lock(format!("Failed to acquire read lock: {e}")))?;

            validator.validate(command)?;
        }

        Ok(())
    }

    fn clone_box(&self) -> Box<dyn LifecycleHook> {
        Box::new(self.clone())
    }
}

impl Default for ArgumentValidationHook {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook for validating environment requirements
#[derive(Debug, Clone)]
pub struct EnvironmentValidationHook {
    /// The validator component that performs environment validation
    validator: Arc<RwLock<CommandValidator>>,
}

impl EnvironmentValidationHook {
    /// Creates a new `EnvironmentValidationHook`
    #[must_use]
    pub fn new() -> Self {
        Self {
            validator: Arc::new(RwLock::new(CommandValidator::new())),
        }
    }
}

impl LifecycleHook for EnvironmentValidationHook {
    fn name(&self) -> &'static str {
        "environment_validation"
    }

    fn stages(&self) -> Vec<LifecycleStage> {
        vec![LifecycleStage::Validation]
    }

    fn on_stage(&self, stage: &LifecycleStage, command: &dyn Command) -> Result<(), CommandError> {
        if *stage == LifecycleStage::Validation {
            let validator = self
                .validator
                .read()
                .map_err(|e| CommandError::Lock(format!("Failed to acquire read lock: {e}")))?;

            validator.validate(command)?;
        }

        Ok(())
    }

    fn clone_box(&self) -> Box<dyn LifecycleHook> {
        Box::new(self.clone())
    }
}

impl Default for EnvironmentValidationHook {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook for validating resource requirements
#[derive(Debug, Clone)]
pub struct ResourceValidationHook {
    /// The validator component that performs resource validation
    validator: Arc<RwLock<CommandValidator>>,
}

impl ResourceValidationHook {
    /// Creates a new `ResourceValidationHook`
    #[must_use]
    pub fn new() -> Self {
        Self {
            validator: Arc::new(RwLock::new(CommandValidator::new())),
        }
    }
}

impl LifecycleHook for ResourceValidationHook {
    fn name(&self) -> &'static str {
        "resource_validation"
    }

    fn stages(&self) -> Vec<LifecycleStage> {
        vec![LifecycleStage::Validation]
    }

    fn on_stage(&self, stage: &LifecycleStage, command: &dyn Command) -> Result<(), CommandError> {
        if *stage == LifecycleStage::Validation {
            let validator = self
                .validator
                .read()
                .map_err(|e| CommandError::Lock(format!("Failed to acquire read lock: {e}")))?;

            validator.validate(command)?;
        }

        Ok(())
    }

    fn clone_box(&self) -> Box<dyn LifecycleHook> {
        Box::new(self.clone())
    }
}

impl Default for ResourceValidationHook {
    fn default() -> Self {
        Self::new()
    }
}

/// A manager for command hooks
pub struct HookManager {
    /// Map of hook names to their implementations
    hooks: HookMap,
}

impl HookManager {
    /// Creates a new hook manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }

    /// Adds a hook to the manager
    ///
    /// # Arguments
    /// * `name` - Name of the hook
    /// * `hook` - The hook implementation
    ///
    /// # Errors
    /// Returns an error if a hook with the same name already exists
    pub fn add_hook(
        &mut self,
        name: &str,
        hook: Box<dyn Fn() -> Result<(), CommandError>>,
    ) -> Result<(), CommandError> {
        if self.hooks.contains_key(name) {
            return Err(CommandError::Hook(format!("Hook '{name}' already exists")));
        }
        self.hooks.insert(name.to_string(), hook);
        Ok(())
    }

    /// Executes all registered hooks
    ///
    /// # Errors
    /// Returns an error if any hook fails to execute
    pub fn execute_hooks(&self) -> Result<(), CommandError> {
        for (name, hook) in &self.hooks {
            hook().map_err(|e| CommandError::Hook(format!("Hook '{name}' failed: {e}")))?;
        }
        Ok(())
    }
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation hook for commands
#[derive(Debug)]
pub struct ValidationHook {
    /// The validator component that performs the actual validation
    validator: Arc<RwLock<CommandValidator>>,
}

/// Pre-execution lifecycle hook
#[derive(Debug)]
pub struct PreExecutionHook {
    /// The validator component that performs pre-execution validation
    validator: Arc<RwLock<CommandValidator>>,
}

/// Post-execution lifecycle hook
#[derive(Debug)]
pub struct PostExecutionHook {
    /// The validator component that performs post-execution validation
    validator: Arc<RwLock<CommandValidator>>,
}

/// Result type for command processors
pub type CommandProcessorResult = Result<(), CommandError>;

/// Command processor trait
///
/// This trait is implemented by types that process commands before or after execution
pub trait CommandProcessor: Send + Sync + Debug {
    /// Returns the name of the processor
    fn name(&self) -> &'static str;

    /// Pre-process a command before execution
    ///
    /// # Arguments
    ///
    /// * `command` - The command to process
    /// * `args` - The arguments to the command
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if processing succeeded, or an error if it failed
    fn pre_process(&self, _command: &dyn Command, _args: &[String]) -> CommandProcessorResult {
        // Default implementation does nothing
        Ok(())
    }

    /// Post-process a command after execution
    ///
    /// # Arguments
    ///
    /// * `command` - The command that was executed
    /// * `args` - The arguments to the command
    /// * `result` - The result of the command execution
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if processing succeeded, or an error if it failed
    fn post_process(
        &self,
        _command: &dyn Command,
        _args: &[String],
        _result: &CommandResult<String>,
    ) -> CommandProcessorResult {
        // Default implementation does nothing
        Ok(())
    }
}

/// Hook for recording command executions in history
#[derive(Debug, Clone)]
pub struct HistoryHook {
    /// Command history manager
    history: Arc<CommandHistory>,
}

impl HistoryHook {
    /// Creates a new history hook
    #[must_use]
    pub fn new(history: Arc<CommandHistory>) -> Self {
        debug!("HistoryHook: Creating new instance");
        Self { history }
    }
}

impl CommandProcessor for HistoryHook {
    fn name(&self) -> &'static str {
        "history_hook"
    }

    fn post_process(
        &self,
        command: &dyn Command,
        args: &[String],
        result: &CommandResult<String>,
    ) -> CommandProcessorResult {
        debug!("HistoryHook: Recording command execution");

        // Determine if command succeeded and get error message if it failed
        let (success, error_message) = match result {
            Ok(_) => (true, None),
            Err(e) => (false, Some(e.to_string())),
        };

        // Record execution
        match self.history.add(
            command.name().to_string(),
            args.to_vec(),
            success,
            error_message,
            None, // No metadata for now
        ) {
            Ok(_) => {
                debug!("HistoryHook: Command execution recorded");
                Ok(())
            }
            Err(e) => {
                error!("HistoryHook: Failed to record command execution: {}", e);
                // We don't want to fail the command because of history recording
                Ok(())
            }
        }
    }
}

/// Factory function to create a history hook
pub fn create_history_hook(history: Arc<CommandHistory>) -> Box<dyn CommandProcessor> {
    Box::new(HistoryHook::new(history))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    #[command(name = "test")]
    struct TestArgs {
        #[arg(short, long)]
        value: String,
    }

    #[derive(Debug)]
    struct TestCommand;

    impl Command for TestCommand {
        fn name(&self) -> &str {
            "test"
        }

        fn description(&self) -> &str {
            "A test command"
        }

        fn execute(&self, _args: &[String]) -> CommandResult<String> {
            Ok("Test command executed".to_string())
        }

        fn parser(&self) -> clap::Command {
            clap::Command::new("test").about("A test command")
        }

        fn clone_box(&self) -> Box<dyn Command> {
            Box::new(TestCommand)
        }
    }

    #[test]
    fn test_hook_execution() {
        let mut manager = HookManager::new();
        let hook = || -> Result<(), CommandError> { Ok(()) };
        manager
            .add_hook("test_hook", Box::new(hook))
            .expect("should succeed");
        manager.execute_hooks().expect("should succeed");
    }

    #[test]
    fn test_hook_error_handling() {
        let mut manager = HookManager::new();
        let hook =
            || -> Result<(), CommandError> { Err(CommandError::Hook("Hook error".to_string())) };
        manager
            .add_hook("error_hook", Box::new(hook))
            .expect("should succeed");
        assert!(manager.execute_hooks().is_err());
    }

    #[test]
    fn hook_registry_register_execute_and_context() {
        let mut reg = HookRegistry::new();
        reg.register("a".into(), || Ok(())).expect("should succeed");
        assert!(reg.register("a".into(), || Ok(())).is_err());
        reg.set_context_data("k", "v").expect("should succeed");
        assert_eq!(
            reg.get_context_data("k").expect("should succeed"),
            Some("v".into())
        );
        assert!(
            reg.get_context_data("missing")
                .expect("should succeed")
                .is_none()
        );
        reg.execute_hooks().expect("should succeed");
    }

    #[test]
    fn hook_registry_execute_propagates_error() {
        let mut reg = HookRegistry::new();
        reg.register("x".into(), || {
            Err(CommandError::ValidationError("bad".into()))
        })
        .expect("should succeed");
        assert!(reg.execute_hooks().is_err());
    }

    #[test]
    fn logging_and_metrics_hooks_run() {
        let cmd = TestCommand;
        LoggingHook::default()
            .execute(&cmd)
            .expect("should succeed");
        MetricsHook::default()
            .execute(&cmd)
            .expect("should succeed");
    }

    #[test]
    fn timing_hook_two_phases() {
        let cmd = TestCommand;
        let hook = TimingHook::new();
        hook.execute(&cmd).expect("should succeed");
        hook.execute(&cmd).expect("should succeed");
    }

    #[test]
    fn hook_manager_duplicate_name_errors() {
        let mut m = HookManager::new();
        m.add_hook("x", Box::new(|| Ok(())))
            .expect("should succeed");
        assert!(
            m.add_hook("x", Box::new(|| Ok(())))
                .unwrap_err()
                .to_string()
                .contains("already exists")
        );
    }

    #[test]
    fn lifecycle_validation_hooks_run_on_validation_stage() {
        let cmd = TestCommand;
        let stages = [
            LifecycleStage::Registration,
            LifecycleStage::Initialization,
            LifecycleStage::Validation,
            LifecycleStage::PreExecution,
            LifecycleStage::PreValidation,
            LifecycleStage::Execution,
            LifecycleStage::PostExecution,
            LifecycleStage::PostValidation,
            LifecycleStage::Completion,
            LifecycleStage::Cleanup,
            LifecycleStage::ErrorHandling,
        ];
        let arg_hook = ArgumentValidationHook::default();
        let env_hook = EnvironmentValidationHook::default();
        let res_hook = ResourceValidationHook::default();
        for s in stages {
            arg_hook.on_stage(&s, &cmd).expect("should succeed");
            env_hook.on_stage(&s, &cmd).expect("should succeed");
            res_hook.on_stage(&s, &cmd).expect("should succeed");
        }
        let _b: Box<dyn LifecycleHook> = arg_hook.clone_box();
    }

    #[test]
    fn history_hook_records_success_and_failure() {
        use tempfile::tempdir;

        let dir = tempdir().expect("should succeed");
        let path = dir.path().join("hist.json");
        let history = Arc::new(CommandHistory::with_options(50, &path).expect("should succeed"));
        let hook = HistoryHook::new(Arc::clone(&history));

        let cmd = TestCommand;
        assert_eq!(hook.name(), "history_hook");
        hook.post_process(&cmd, &[], &Ok("ok".into()))
            .expect("should succeed");
        hook.post_process(&cmd, &[], &Err(CommandError::ValidationError("e".into())))
            .expect("should succeed");

        let boxed = create_history_hook(history);
        assert_eq!(boxed.name(), "history_hook");
    }
}
