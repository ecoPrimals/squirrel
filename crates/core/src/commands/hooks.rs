use std::error::Error;
use std::sync::Arc;
use std::collections::HashMap;
use std::sync::RwLock;
use crate::core::Command;
use super::lifecycle::LifecycleStage;

#[derive(Debug)]
pub struct HookError {
    pub hook_name: String,
    pub message: String,
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hook error ({}): {}", self.hook_name, self.message)
    }
}

impl Error for HookError {}

pub trait Hook: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, context: &HookContext) -> Result<(), Box<dyn Error>>;
    fn stages(&self) -> Vec<LifecycleStage>;
}

#[derive(Clone)]
pub struct HookContext {
    pub command: Arc<dyn Command + Send + Sync>,
    pub stage: LifecycleStage,
    pub data: HashMap<String, String>,
}

pub struct HookManager {
    hooks: RwLock<Vec<Box<dyn Hook>>>,
    context: RwLock<HashMap<String, String>>,
}

impl HookManager {
    pub fn new() -> Self {
        Self {
            hooks: RwLock::new(Vec::new()),
            context: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_hook(&self, hook: Box<dyn Hook>) -> Result<(), Box<dyn Error>> {
        let mut hooks = self.hooks.write().map_err(|_| {
            Box::new(HookError {
                hook_name: "manager".to_string(),
                message: "Failed to acquire write lock on hooks".to_string(),
            })
        })?;
        hooks.push(hook);
        Ok(())
    }

    pub fn execute_hooks(
        &self,
        command: Arc<dyn Command + Send + Sync>,
        stage: LifecycleStage,
    ) -> Result<(), Box<dyn Error>> {
        let hooks = self.hooks.read().map_err(|_| {
            Box::new(HookError {
                hook_name: "manager".to_string(),
                message: "Failed to acquire read lock on hooks".to_string(),
            })
        })?;

        let context_data = self.context.read().map_err(|_| {
            Box::new(HookError {
                hook_name: "manager".to_string(),
                message: "Failed to acquire read lock on context".to_string(),
            })
        })?;

        let context = HookContext {
            command: command.clone(),
            stage: stage.clone(),
            data: context_data.clone(),
        };

        for hook in hooks.iter() {
            if hook.stages().contains(&stage) {
                hook.execute(&context)?;
            }
        }

        Ok(())
    }

    pub fn set_context_data(&self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        let mut context = self.context.write().map_err(|_| {
            Box::new(HookError {
                hook_name: "manager".to_string(),
                message: "Failed to acquire write lock on context".to_string(),
            })
        })?;
        context.insert(key.to_string(), value.to_string());
        Ok(())
    }

    pub fn get_context_data(&self, key: &str) -> Result<Option<String>, Box<dyn Error>> {
        let context = self.context.read().map_err(|_| {
            Box::new(HookError {
                hook_name: "manager".to_string(),
                message: "Failed to acquire read lock on context".to_string(),
            })
        })?;
        Ok(context.get(key).cloned())
    }
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
    }
}

// Example hooks
pub struct LoggingHook {
    name: String,
    description: String,
}

impl LoggingHook {
    pub fn new() -> Self {
        Self {
            name: "logging".to_string(),
            description: "Logs command execution stages".to_string(),
        }
    }
}

impl Hook for LoggingHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn execute(&self, context: &HookContext) -> Result<(), Box<dyn Error>> {
        println!(
            "Command '{}' at stage {:?}",
            context.command.name(),
            context.stage
        );
        Ok(())
    }

    fn stages(&self) -> Vec<LifecycleStage> {
        vec![
            LifecycleStage::Registration,
            LifecycleStage::Initialization,
            LifecycleStage::Validation,
            LifecycleStage::Execution,
            LifecycleStage::Completion,
            LifecycleStage::Cleanup,
        ]
    }
}

impl Default for LoggingHook {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MetricsHook {
    name: String,
    description: String,
}

impl MetricsHook {
    pub fn new() -> Self {
        Self {
            name: "metrics".to_string(),
            description: "Collects command execution metrics".to_string(),
        }
    }
}

impl Hook for MetricsHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn execute(&self, context: &HookContext) -> Result<(), Box<dyn Error>> {
        // In a real implementation, this would send metrics to a monitoring system
        println!(
            "Metrics - Command: {}, Stage: {:?}",
            context.command.name(),
            context.stage
        );
        Ok(())
    }

    fn stages(&self) -> Vec<LifecycleStage> {
        vec![LifecycleStage::Execution, LifecycleStage::Completion]
    }
}

impl Default for MetricsHook {
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
    fn test_hook_registration() {
        let manager = HookManager::new();
        let hook = LoggingHook::new();
        manager.register_hook(Box::new(hook)).unwrap();
    }

    #[test]
    fn test_hook_execution() {
        let manager = HookManager::new();
        let hook = LoggingHook::new();
        manager.register_hook(Box::new(hook)).unwrap();

        let command = Arc::new(TestCommand {
            name: "test".to_string(),
            description: "Test command".to_string(),
        });

        manager
            .execute_hooks(command, LifecycleStage::Execution)
            .unwrap();
    }

    #[test]
    fn test_hook_context() {
        let manager = HookManager::new();
        manager.set_context_data("test_key", "test_value").unwrap();
        assert_eq!(
            manager.get_context_data("test_key").unwrap().unwrap(),
            "test_value"
        );
    }
} 