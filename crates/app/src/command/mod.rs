use crate::error::{Result, CoreError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::fmt::Debug;
use async_trait::async_trait;
use squirrel_commands::Command;

/// Command handling functionality for the application
///
/// This module provides the core command processing components that 
/// enable the application to handle incoming commands, execute them,
/// and process them through pre and post hooks.
/// Adapter module for command handling
pub mod adapter;
/// Command history management
pub mod history;
/// Command suggestions
pub mod suggestions;

pub use adapter::{CommandHandlerAdapter, create_handler_adapter, create_handler_adapter_with_handler};
pub use history::{CommandHistory, CommandHistoryEntry};
pub use suggestions::{CommandSuggestions, CommandSuggestion};

/// A command handler that processes commands
#[derive(Debug)]
pub struct CommandHandler {
    /// Map of command types to their processors
    handlers: Arc<RwLock<HashMap<String, Box<dyn CommandProcessor>>>>,
    /// Command history manager
    history: Arc<CommandHistory>,
    /// Command suggestions manager
    suggestions: Arc<CommandSuggestions>,
}

impl CommandHandler {
    /// Creates a new `CommandHandler` with default settings
    #[must_use]
    pub fn new() -> Self {
        let history = Arc::new(CommandHistory::new());
        let suggestions = Arc::new(CommandSuggestions::new(Arc::clone(&history)));
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            history,
            suggestions,
        }
    }
    
    /// Creates a new `CommandHandler` with dependencies
    #[must_use]
    pub fn with_dependencies(
        handlers: Arc<RwLock<HashMap<String, Box<dyn CommandProcessor>>>>,
        history: Arc<CommandHistory>,
        suggestions: Arc<CommandSuggestions>,
    ) -> Self {
        Self { handlers, history, suggestions }
    }
    
    /// Registers a command processor for a specific command type
    /// 
    /// # Errors
    /// 
    /// Returns an error if the registration fails
    pub async fn register(&self, command_type: String, processor: Box<dyn CommandProcessor>) -> Result<()> {
        let mut handlers = self.handlers.write().await;
        handlers.insert(command_type, processor);
        Ok(())
    }
    
    /// Handles a command by routing it to the appropriate processor
    /// 
    /// # Errors
    /// 
    /// Returns an error if the command handling fails
    pub async fn handle(&self, command: &dyn Command) -> Result<()> {
        let handlers = self.handlers.read().await;
        let command_name = command.name();
        let result = if let Some(processor) = handlers.get(command_name) {
            processor.process(command).await
        } else {
            Err(CoreError::Command(format!("Command not found: {command_name}")))
        };

        // Record command execution in history
        self.history.record(command, result.is_ok(), None).await?;
        result
    }

    /// Gets the command history manager
    #[must_use]
    pub fn history(&self) -> Arc<CommandHistory> {
        Arc::clone(&self.history)
    }

    /// Gets the command suggestions manager
    #[must_use]
    pub fn suggestions(&self) -> Arc<CommandSuggestions> {
        Arc::clone(&self.suggestions)
    }

    /// Searches command history for entries matching the given criteria
    /// 
    /// # Arguments
    /// * `query` - The search query to match against command names
    /// * `limit` - Maximum number of entries to return
    /// 
    /// # Returns
    /// A vector of matching history entries
    pub async fn search_history(&self, query: &str, limit: usize) -> Vec<CommandHistoryEntry> {
        self.history.search(query, limit).await
    }

    /// Gets the last N command history entries
    /// 
    /// # Arguments
    /// * `count` - Number of entries to retrieve
    /// 
    /// # Returns
    /// A vector of the most recent history entries
    pub async fn get_recent_history(&self, count: usize) -> Vec<CommandHistoryEntry> {
        self.history.get_recent(count).await
    }

    /// Gets command suggestions based on partial input
    /// 
    /// # Arguments
    /// * `partial_input` - Partial command input
    /// * `context` - Optional context string to improve suggestions
    /// 
    /// # Returns
    /// A vector of command suggestions sorted by relevance
    pub async fn get_suggestions(&self, partial_input: &str, context: Option<&str>) -> Vec<CommandSuggestion> {
        self.suggestions.get_suggestions(partial_input, context).await
    }

    /// Gets command suggestions based on command usage patterns
    /// 
    /// # Arguments
    /// * `last_command` - Last executed command
    /// * `limit` - Maximum number of suggestions
    /// 
    /// # Returns
    /// A vector of command suggestions based on common patterns
    pub async fn get_pattern_suggestions(&self, last_command: &str, limit: usize) -> Vec<CommandSuggestion> {
        self.suggestions.get_pattern_suggestions(last_command, limit).await
    }

    /// Adds metadata for command suggestions
    /// 
    /// # Arguments
    /// * `command_name` - Name of the command
    /// * `description` - Command description
    /// * `example` - Usage example
    /// 
    /// # Errors
    /// Returns an error if the metadata update fails
    pub async fn add_suggestion_metadata(&self, command_name: String, description: String, example: String) -> Result<()> {
        self.suggestions.add_metadata(command_name, description, example).await
    }
}

/// Default implementation
impl Default for CommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Processes a command and returns a result
#[async_trait]
pub trait CommandProcessor: Send + Sync + Debug {
    /// Process a command asynchronously
    /// 
    /// # Errors
    /// 
    /// Returns an error if the command processing fails
    async fn process(&self, command: &dyn Command) -> Result<()>;
}

/// A default command processor that logs commands but doesn't do anything else
#[derive(Debug)]
pub struct DefaultCommandProcessor;

impl DefaultCommandProcessor {
    /// Create a new default command processor
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultCommandProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommandProcessor for DefaultCommandProcessor {
    async fn process(&self, _command: &dyn Command) -> Result<()> {
        // Implementation left empty for documentation purposes
        Ok(())
    }
}

/// A trait for command hooks that can be executed before or after command processing
#[async_trait]
pub trait CommandHook: Send + Sync + Debug {
    /// Execute the hook
    /// 
    /// # Errors
    /// 
    /// Returns an error if the hook execution fails
    async fn execute(&self, command: &dyn Command) -> Result<()>;
}

/// A command processor with pre and post hooks
#[derive(Debug)]
pub struct HookedCommandProcessor {
    /// The underlying processor
    processor: Box<dyn CommandProcessor>,
    /// Hooks to execute before processing
    pre_hooks: Vec<Box<dyn CommandHook>>,
    /// Hooks to execute after processing
    post_hooks: Vec<Box<dyn CommandHook>>,
}

impl HookedCommandProcessor {
    /// Creates a new `HookedCommandProcessor`
    #[must_use]
    pub fn new(processor: Box<dyn CommandProcessor>) -> Self {
        Self {
            processor,
            pre_hooks: Vec::new(),
            post_hooks: Vec::new(),
        }
    }
    
    /// Adds a pre-hook to the processor
    pub fn add_pre_hook(&mut self, hook: Box<dyn CommandHook>) {
        self.pre_hooks.push(hook);
    }
    
    /// Adds a post-hook to the processor
    pub fn add_post_hook(&mut self, hook: Box<dyn CommandHook>) {
        self.post_hooks.push(hook);
    }
    
    /// Execute all pre-hooks
    /// 
    /// # Errors
    /// 
    /// Returns an error if any pre-hook execution fails
    pub async fn execute_pre_hooks(&self, command: &dyn Command) -> Result<()> {
        for hook in &self.pre_hooks {
            hook.execute(command).await?;
        }
        Ok(())
    }
    
    /// Execute all post-hooks
    /// 
    /// # Errors
    /// 
    /// Returns an error if any post-hook execution fails
    pub async fn execute_post_hooks(&self, command: &dyn Command) -> Result<()> {
        for hook in &self.post_hooks {
            hook.execute(command).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl CommandProcessor for HookedCommandProcessor {
    async fn process(&self, command: &dyn Command) -> Result<()> {
        self.execute_pre_hooks(command).await?;
        let result = self.processor.process(command).await;
        if result.is_ok() {
            self.execute_post_hooks(command).await?;
        }
        result
    }
}

/// Provides pre and post processing hooks for commands
///
/// Allows for registering processors that will run before
/// and after command execution, enabling cross-cutting concerns like
/// logging, validation, and side effects.
#[derive(Debug)]
pub struct CommandHookImpl {
    /// Processors that run before command execution
    pre_hooks: Arc<RwLock<Vec<Box<dyn CommandProcessor>>>>,
    /// Processors that run after command execution
    post_hooks: Arc<RwLock<Vec<Box<dyn CommandProcessor>>>>,
}

impl CommandHookImpl {
    /// Creates a new command hook with empty pre and post hook lists
    ///
    /// # Returns
    /// A new instance
    #[must_use] pub fn new() -> Self {
        Self {
            pre_hooks: Arc::new(RwLock::new(Vec::new())),
            post_hooks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Adds a pre-execution hook for commands.
    ///
    /// # Arguments
    /// * `hook` - The command processor to execute before command processing
    ///
    /// # Errors
    /// This function will not produce errors, but returns a Result type for consistency
    /// with the async interface.
    pub async fn add_pre_hook(&self, hook: Box<dyn CommandProcessor>) -> Result<()> {
        let mut pre_hooks = self.pre_hooks.write().await;
        pre_hooks.push(hook);
        Ok(())
    }

    /// Adds a post-execution hook for commands.
    ///
    /// # Arguments
    /// * `hook` - The command processor to execute after command processing
    ///
    /// # Errors
    /// This function will not produce errors, but returns a Result type for consistency
    /// with the async interface.
    pub async fn add_post_hook(&self, hook: Box<dyn CommandProcessor>) -> Result<()> {
        let mut post_hooks = self.post_hooks.write().await;
        post_hooks.push(hook);
        Ok(())
    }

    /// Executes all pre-hooks for the given command.
    ///
    /// # Arguments
    /// * `command` - The command about to be processed
    ///
    /// # Errors
    /// Returns an error if any of the pre-hooks fails during execution.
    pub async fn execute_pre_hooks(&self, command: &dyn Command) -> Result<()> {
        let pre_hooks = self.pre_hooks.read().await;
        for hook in pre_hooks.iter() {
            hook.process(command).await?;
        }
        Ok(())
    }

    /// Executes all post-hooks for the given command.
    ///
    /// # Arguments
    /// * `command` - The command that was processed
    ///
    /// # Errors
    /// Returns an error if any of the post-hooks fails during execution.
    pub async fn execute_post_hooks(&self, command: &dyn Command) -> Result<()> {
        let post_hooks = self.post_hooks.read().await;
        for hook in post_hooks.iter() {
            hook.process(command).await?;
        }
        Ok(())
    }
}

impl Default for CommandHookImpl {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating command handlers
#[derive(Debug, Default)]
pub struct CommandHandlerFactory;

impl CommandHandlerFactory {
    /// Creates a new command handler factory
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Creates a new command handler
    #[must_use]
    pub fn create(&self) -> Arc<CommandHandler> {
        Arc::new(CommandHandler::new())
    }

    /// Creates a new command handler with dependencies
    #[must_use]
    pub fn create_with_dependencies(
        &self,
        handlers: Arc<RwLock<HashMap<String, Box<dyn CommandProcessor>>>>,
        history: Arc<CommandHistory>,
        suggestions: Arc<CommandSuggestions>,
    ) -> Arc<CommandHandler> {
        Arc::new(CommandHandler::with_dependencies(handlers, history, suggestions))
    }

    /// Creates a new command handler adapter
    #[must_use]
    pub fn create_adapter(&self) -> Arc<CommandHandlerAdapter> {
        create_handler_adapter()
    }

    /// Creates a new command handler adapter with an existing handler
    #[must_use]
    pub fn create_adapter_with_handler(_handler: &Arc<CommandHandler>) -> Arc<CommandHandlerAdapter> {
        Arc::new(CommandHandlerAdapter::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;
    use squirrel_commands::CommandError;
    use clap::Command as ClapCommand;

    #[derive(Debug)]
    struct TestCommand {
        name: &'static str,
    }

    impl Command for TestCommand {
        fn name(&self) -> &str {
            self.name
        }
        
        fn description(&self) -> &str {
            "Test command for unit tests"
        }
        
        fn execute(&self, _args: &[String]) -> std::result::Result<String, CommandError> {
            Ok("Test command executed".to_string())
        }
        
        fn parser(&self) -> ClapCommand {
            ClapCommand::new(self.name)
                .about("Test command for unit tests")
        }
        
        fn clone_box(&self) -> Box<dyn Command + 'static> {
            Box::new(TestCommand {
                name: self.name,
            })
        }
    }

    impl TestCommand {
        fn new(name: &'static str) -> Self {
            Self { name }
        }
    }

    #[derive(Debug)]
    struct TestProcessor;

    #[async_trait]
    impl CommandProcessor for TestProcessor {
        async fn process(&self, _command: &dyn Command) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_command_handler_with_history() {
        let handler = CommandHandler::new();
        let cmd = TestCommand::new("test_command");

        // Register and execute command
        handler.register(cmd.name().to_string(), Box::new(TestProcessor)).await.unwrap();
        handler.handle(&cmd).await.unwrap();

        // Check history
        let history = handler.get_recent_history(1).await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].command_name, "test_command");
        assert!(history[0].success);
    }

    #[tokio::test]
    async fn test_command_history_search() {
        let handler = CommandHandler::new();
        let cmd1 = TestCommand::new("test_command1");
        let cmd2 = TestCommand::new("test_command2");

        // Register and execute commands
        handler.register(cmd1.name().to_string(), Box::new(TestProcessor)).await.unwrap();
        handler.register(cmd2.name().to_string(), Box::new(TestProcessor)).await.unwrap();
        handler.handle(&cmd1).await.unwrap();
        handler.handle(&cmd2).await.unwrap();

        // Search history
        let results = handler.search_history("command1", 10).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].command_name, "test_command1");
    }

    #[tokio::test]
    async fn test_failed_command_history() {
        let handler = CommandHandler::new();
        let cmd = TestCommand::new("nonexistent_command");

        // Try to execute non-existent command
        let result = handler.handle(&cmd).await;
        assert!(result.is_err());

        // Check history
        let history = handler.get_recent_history(1).await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].command_name, "nonexistent_command");
        assert!(!history[0].success);
    }

    #[tokio::test]
    async fn test_command_suggestions() {
        let handler = CommandHandler::new();
        let cmd1 = TestCommand::new("test_command1");
        let cmd2 = TestCommand::new("test_command2");

        // Register and execute commands
        handler.register(cmd1.name().to_string(), Box::new(TestProcessor)).await.unwrap();
        handler.register(cmd2.name().to_string(), Box::new(TestProcessor)).await.unwrap();
        handler.handle(&cmd1).await.unwrap();
        handler.handle(&cmd2).await.unwrap();

        // Add metadata for suggestions
        handler.add_suggestion_metadata(
            "test_command1".to_string(),
            "Test command 1".to_string(),
            "test_command1 arg1".to_string(),
        ).await.unwrap();

        // Get suggestions
        let suggestions = handler.get_suggestions("test", None).await;
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].command_name, "test_command1");
        assert!(suggestions[0].description.is_some());
        assert!(suggestions[0].example.is_some());
    }

    #[tokio::test]
    async fn test_pattern_suggestions() {
        let handler = CommandHandler::new();
        let cmd1 = TestCommand::new("command1");
        let cmd2 = TestCommand::new("command2");

        // Register commands
        handler.register(cmd1.name().to_string(), Box::new(TestProcessor)).await.unwrap();
        handler.register(cmd2.name().to_string(), Box::new(TestProcessor)).await.unwrap();

        // Create a pattern of command1 followed by command2
        for _ in 0..3 {
            handler.handle(&cmd1).await.unwrap();
            handler.handle(&cmd2).await.unwrap();
        }

        // Get pattern suggestions
        let suggestions = handler.get_pattern_suggestions("command1", 5).await;
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].command_name, "command2");
    }
} 