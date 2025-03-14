use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;

use super::{Command, CommandArgs, CommandResult, CommandExecutor, CommandLifecycle, LifecycleStage};
use crate::core::error::types::{Result, SquirrelError};

/// Basic command executor implementation
pub struct BasicCommandExecutor {
    lifecycle: Arc<CommandLifecycle>,
}

impl BasicCommandExecutor {
    /// Create a new basic command executor
    pub fn new(lifecycle: Arc<CommandLifecycle>) -> Self {
        Self { lifecycle }
    }
}

impl CommandExecutor for BasicCommandExecutor {
    fn execute<'a>(&'a self, command: &'a dyn Command, args: &'a CommandArgs) -> Pin<Box<dyn Future<Output = Result<CommandResult>> + Send + 'a>> {
        Box::pin(async move {
            self.lifecycle.execute_stage(LifecycleStage::Execution, args, None).await?;
            let result = command.execute(args).await?;
            self.lifecycle.execute_stage(LifecycleStage::Completion, args, Some(&result)).await?;
            Ok(result)
        })
    }
    
    fn validate<'a>(&'a self, args: &'a CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            if args.args.is_empty() {
                return Err(SquirrelError::Command("No command arguments provided".to_string()));
            }
            self.lifecycle.execute_stage(LifecycleStage::Validation, args, None).await?;
            Ok(())
        })
    }
    
    fn cleanup<'a>(&'a self, args: &'a CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            self.lifecycle.execute_stage(LifecycleStage::Cleanup, args, None).await?;
            Ok(())
        })
    }
    
    fn pre_execute<'a>(&'a self, args: &'a CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            self.lifecycle.execute_stage(LifecycleStage::Initialization, args, None).await?;
            Ok(())
        })
    }
    
    fn post_execute<'a>(&'a self, args: &'a CommandArgs, result: &'a CommandResult) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            self.lifecycle.execute_stage(LifecycleStage::Completion, args, Some(result)).await?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    
    #[test]
    async fn test_basic_executor() {
        let lifecycle = Arc::new(CommandLifecycle::new());
        let executor = BasicCommandExecutor::new(lifecycle.clone());
        
        let args = CommandArgs {
            args: vec!["test".to_string()],
            env: Vec::new(),
        };
        
        // Test validation
        executor.validate(&args).await.unwrap();
        
        // Test execution
        let command = super::super::BasicCommand::new("test", "Test command");
        let result = executor.execute(&command, &args).await.unwrap();
        assert!(result.success);
    }
} 