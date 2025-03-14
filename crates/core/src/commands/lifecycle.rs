use std::collections::HashMap;
use std::error::Error;
use std::sync::RwLock;
use std::time::SystemTime;
use crate::core::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleStage {
    Registration,
    Initialization,
    Validation,
    Execution,
    Completion,
    Cleanup,
}

#[derive(Debug)]
pub struct LifecycleError {
    pub stage: LifecycleStage,
    pub message: String,
}

impl std::fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lifecycle error at {:?}: {}", self.stage, self.message)
    }
}

impl Error for LifecycleError {}

pub trait CommandHook: Send + Sync {
    fn before_stage(&self, stage: &LifecycleStage, command: &dyn Command) -> Result<(), Box<dyn Error>>;
    fn after_stage(&self, stage: &LifecycleStage, command: &dyn Command) -> Result<(), Box<dyn Error>>;
}

#[derive(Default)]
pub struct CommandLifecycle {
    hooks: RwLock<Vec<Box<dyn CommandHook>>>,
    state: RwLock<HashMap<String, LifecycleStage>>,
}

impl CommandLifecycle {
    pub fn new() -> Self {
        Self {
            hooks: RwLock::new(Vec::new()),
            state: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_hook(&self, hook: Box<dyn CommandHook>) -> Result<(), Box<dyn Error>> {
        let mut hooks = self.hooks.write().map_err(|_| {
            Box::new(LifecycleError {
                stage: LifecycleStage::Registration,
                message: "Failed to acquire write lock on hooks".to_string(),
            }) as Box<dyn Error>
        })?;
        hooks.push(hook);
        Ok(())
    }

    pub fn execute_stage(&self, stage: LifecycleStage, command: &dyn Command) -> Result<(), Box<dyn Error>> {
        // Execute before hooks
        let hooks = self.hooks.read().map_err(|_| {
            Box::new(LifecycleError {
                stage: stage.clone(),
                message: "Failed to acquire read lock on hooks".to_string(),
            }) as Box<dyn Error>
        })?;

        for hook in hooks.iter() {
            hook.before_stage(&stage, command)?;
        }

        // Update state
        let mut state = self.state.write().map_err(|_| {
            Box::new(LifecycleError {
                stage: stage.clone(),
                message: "Failed to acquire write lock on state".to_string(),
            }) as Box<dyn Error>
        })?;
        state.insert(command.name().to_string(), stage.clone());

        // Execute after hooks
        for hook in hooks.iter() {
            hook.after_stage(&stage, command)?;
        }

        Ok(())
    }

    pub fn get_stage(&self, command_name: &str) -> Result<Option<LifecycleStage>, Box<dyn Error>> {
        let state = self.state.read().map_err(|_| {
            Box::new(LifecycleError {
                stage: LifecycleStage::Registration,
                message: "Failed to acquire read lock on state".to_string(),
            }) as Box<dyn Error>
        })?;
        Ok(state.get(command_name).cloned())
    }
}

// Example hook implementation
pub struct LoggingHook {
    start_time: SystemTime,
}

impl LoggingHook {
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now(),
        }
    }
}

impl CommandHook for LoggingHook {
    fn before_stage(&self, stage: &LifecycleStage, command: &dyn Command) -> Result<(), Box<dyn Error>> {
        println!("Starting {:?} stage for command {}", stage, command.name());
        Ok(())
    }

    fn after_stage(&self, stage: &LifecycleStage, command: &dyn Command) -> Result<(), Box<dyn Error>> {
        let duration = SystemTime::now().duration_since(self.start_time).unwrap_or_default();
        println!(
            "Completed {:?} stage for command {} after {:?}",
            stage,
            command.name(),
            duration
        );
        Ok(())
    }
}

impl Default for LoggingHook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    struct TestCommand {
        name: String,
        description: String,
    }

    impl crate::core::CommandOutput for TestCommand {
        fn execute_with_output(&self, output: &mut dyn Write) -> Result<(), Box<dyn Error>> {
            writeln!(output, "Test command executed")?;
            Ok(())
        }
    }

    impl Command for TestCommand {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn execute(&self) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
    }

    #[test]
    fn test_lifecycle_stages() {
        let lifecycle = CommandLifecycle::new();
        let command = TestCommand {
            name: "test".to_string(),
            description: "Test command".to_string(),
        };

        // Test each stage
        lifecycle.execute_stage(LifecycleStage::Registration, &command).unwrap();
        assert_eq!(
            lifecycle.get_stage("test").unwrap().unwrap(),
            LifecycleStage::Registration
        );

        lifecycle.execute_stage(LifecycleStage::Initialization, &command).unwrap();
        assert_eq!(
            lifecycle.get_stage("test").unwrap().unwrap(),
            LifecycleStage::Initialization
        );
    }

    #[test]
    fn test_hooks() {
        let lifecycle = CommandLifecycle::new();
        let hook = LoggingHook::new();
        lifecycle.add_hook(Box::new(hook)).unwrap();

        let command = TestCommand {
            name: "test".to_string(),
            description: "Test command".to_string(),
        };

        lifecycle.execute_stage(LifecycleStage::Registration, &command).unwrap();
    }
} 