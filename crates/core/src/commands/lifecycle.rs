use std::error::Error;
use std::sync::RwLock;
use crate::commands::validation::ValidationError;
use super::Command;

/// Represents different stages in a command's lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LifecycleStage {
    /// Before command registration
    Registration,
    /// During command initialization
    Initialization,
    /// During command validation
    Validation,
    /// Before command execution
    PreExecution,
    /// During command execution
    Execution,
    /// After command execution
    PostExecution,
    /// During command completion
    Completion,
    /// During cleanup
    Cleanup,
}

/// A hook that can be executed at different stages of a command's lifecycle
pub trait LifecycleHook: Send + Sync {
    /// Called before a lifecycle stage is executed
    ///
    /// # Arguments
    /// * `stage` - The lifecycle stage being executed
    /// * `command` - The command being executed
    ///
    /// # Errors
    /// Returns an error if the hook fails to execute
    fn pre_stage(&self, stage: &LifecycleStage, command: &dyn Command) -> Result<(), Box<dyn Error>>;

    /// Called after a lifecycle stage is executed
    ///
    /// # Arguments
    /// * `stage` - The lifecycle stage being executed
    /// * `command` - The command being executed
    ///
    /// # Errors
    /// Returns an error if the hook fails to execute
    fn post_stage(&self, stage: &LifecycleStage, command: &dyn Command) -> Result<(), Box<dyn Error>>;
}

/// Manages the lifecycle of a command
pub struct CommandLifecycle {
    /// List of hooks that will be executed during each lifecycle stage
    hooks: RwLock<Vec<Box<dyn LifecycleHook>>>,
}

impl Default for CommandLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandLifecycle {
    /// Creates a new command lifecycle manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            hooks: RwLock::new(Vec::new()),
        }
    }

    /// Executes all hooks for the given lifecycle stage.
    ///
    /// # Arguments
    /// * `stage` - The lifecycle stage to execute hooks for
    /// * `command` - The command being executed
    ///
    /// # Errors
    /// Returns an error if any hook fails to execute or if the hooks lock is poisoned
    ///
    /// # Panics
    /// Panics if the hooks lock is poisoned
    pub fn execute_stage(&self, stage: LifecycleStage, command: &dyn Command) -> Result<(), Box<dyn Error>> {
        let hooks = self.hooks.read().map_err(|e| Box::new(ValidationError {
            rule_name: "LifecycleHook".to_string(),
            message: format!("Failed to acquire read lock: {e}"),
        }))?;
        
        // Execute pre-stage hooks
        for hook in hooks.iter() {
            if let Err(e) = hook.pre_stage(&stage, command) {
                return Err(Box::new(ValidationError {
                    rule_name: "LifecycleHook".to_string(),
                    message: format!("Pre-stage hook failed: {e}"),
                }));
            }
        }

        // Execute post-stage hooks
        for hook in hooks.iter() {
            if let Err(e) = hook.post_stage(&stage, command) {
                return Err(Box::new(ValidationError {
                    rule_name: "LifecycleHook".to_string(),
                    message: format!("Post-stage hook failed: {e}"),
                }));
            }
        }

        Ok(())
    }

    /// Adds a hook to be executed during lifecycle stages
    /// 
    /// # Arguments
    /// * `hook` - The hook to add
    ///
    /// # Errors
    /// Returns an error if the hook cannot be added or if the hooks lock is poisoned
    ///
    /// # Panics
    /// Panics if the hooks lock is poisoned
    #[allow(dead_code)]
    pub fn add_hook(&self, hook: Box<dyn LifecycleHook>) -> Result<(), Box<dyn Error>> {
        let mut hooks = self.hooks.write().map_err(|e| Box::new(ValidationError {
            rule_name: "LifecycleHook".to_string(),
            message: format!("Failed to acquire write lock: {e}"),
        }))?;
        hooks.push(hook);
        Ok(())
    }

    /// Returns the number of hooks registered with the lifecycle manager
    #[allow(dead_code)]
    pub fn hooks(&self) -> usize {
        self.hooks.read().map(|hooks| hooks.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_stages() {
        let stage = LifecycleStage::PreExecution;
        assert_eq!(stage, LifecycleStage::PreExecution);
    }
}