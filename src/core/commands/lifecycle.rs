use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::SystemTime;
use std::future::Future;
use std::pin::Pin;

use crate::core::commands::{Command, CommandResult, CommandArgs};
use crate::core::error::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleStage {
    Registration,
    Initialization,
    Validation,
    Execution,
    Completion,
    Cleanup,
}

#[derive(Debug, thiserror::Error)]
#[error("Lifecycle error at {stage:?}: {message}")]
pub struct LifecycleError {
    pub stage: LifecycleStage,
    pub message: String,
}

/// Hook for command lifecycle events
pub trait CommandHook: Send + Sync {
    fn before_stage<'a>(&'a self, stage: &'a LifecycleStage, command: &'a CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
    fn after_stage<'a>(&'a self, stage: &'a LifecycleStage, command: &'a CommandArgs, result: Option<&'a CommandResult>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
}

pub struct CommandLifecycle {
    hooks: Arc<RwLock<Vec<Arc<dyn CommandHook>>>>,
    state: Arc<RwLock<HashMap<String, LifecycleStage>>>,
}

impl Default for CommandLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandLifecycle {
    pub fn new() -> Self {
        Self {
            hooks: Arc::new(RwLock::new(Vec::new())),
            state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_hook(&self, hook: Arc<dyn CommandHook>) -> Result<()> {
        let mut hooks = self.hooks.write().await;
        hooks.push(hook);
        Ok(())
    }

    pub async fn execute_stage(
        &self,
        stage: LifecycleStage,
        command: &CommandArgs,
        result: Option<&CommandResult>,
    ) -> Result<()> {
        // Execute before hooks
        let hooks = self.hooks.read().await;
        for hook in hooks.iter() {
            hook.before_stage(&stage, command).await?;
        }

        // Update state
        let mut state = self.state.write().await;
        if let Some(cmd) = command.args.first() {
            state.insert(cmd.clone(), stage.clone());
        }

        // Execute after hooks
        for hook in hooks.iter() {
            hook.after_stage(&stage, command, result).await?;
        }

        Ok(())
    }

    pub async fn get_stage(&self, command_name: &str) -> Result<Option<LifecycleStage>> {
        let state = self.state.read().await;
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
    fn before_stage<'a>(&'a self, stage: &'a LifecycleStage, command: &'a CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            println!("Starting {:?} stage for command {:?}", stage, command);
            Ok(())
        })
    }

    fn after_stage<'a>(&'a self, stage: &'a LifecycleStage, command: &'a CommandArgs, result: Option<&'a CommandResult>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let duration = SystemTime::now()
                .duration_since(self.start_time)
                .unwrap_or_default();
            let status = result.map(|r| if r.success { "success" } else { "failure" }).unwrap_or("unknown");
            println!(
                "Completed {:?} stage for command {:?} with status {} after {:?}",
                stage,
                command,
                status,
                duration
            );
            Ok(())
        })
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
    use tokio::test;

    #[test]
    async fn test_lifecycle_stages() {
        let lifecycle = CommandLifecycle::new();
        let command = CommandArgs {
            args: vec!["test".to_string()],
            env: Vec::new(),
        };

        // Test each stage
        lifecycle.execute_stage(LifecycleStage::Registration, &command, None).await.unwrap();
        assert_eq!(
            lifecycle.get_stage("test").await.unwrap().unwrap(),
            LifecycleStage::Registration
        );

        lifecycle.execute_stage(LifecycleStage::Initialization, &command, None).await.unwrap();
        assert_eq!(
            lifecycle.get_stage("test").await.unwrap().unwrap(),
            LifecycleStage::Initialization
        );
    }

    #[test]
    async fn test_hooks() {
        let lifecycle = CommandLifecycle::new();
        let hook = Arc::new(LoggingHook::new());
        lifecycle.add_hook(hook).await.unwrap();

        let command = CommandArgs {
            args: vec!["test".to_string()],
            env: Vec::new(),
        };

        let result = Some(CommandResult {
            success: true,
            output: "Test output".to_string(),
            error: None,
        });

        lifecycle.execute_stage(LifecycleStage::Registration, &command, None).await.unwrap();
        lifecycle.execute_stage(LifecycleStage::Execution, &command, result.as_ref()).await.unwrap();
    }
} 