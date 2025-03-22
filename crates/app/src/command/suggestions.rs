use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::Result;
use crate::command::history::CommandHistory;

/// Maximum number of suggestions to return
const MAX_SUGGESTIONS: usize = 5;

/// Represents a command suggestion with relevance score
#[derive(Debug, Clone)]
pub struct CommandSuggestion {
    /// Command name
    pub command_name: String,
    /// Relevance score (0.0 to 1.0)
    pub relevance: f64,
    /// Optional description
    pub description: Option<String>,
    /// Usage example
    pub example: Option<String>,
}

/// Manages command suggestions based on history and patterns
#[derive(Debug)]
pub struct CommandSuggestions {
    /// Command history for suggestions
    history: Arc<CommandHistory>,
    /// Command descriptions and examples
    metadata: Arc<RwLock<HashMap<String, (String, String)>>>,
}

impl CommandSuggestions {
    /// Creates a new command suggestions manager
    #[must_use]
    pub fn new(history: Arc<CommandHistory>) -> Self {
        Self {
            history,
            metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Adds metadata for a command
    /// 
    /// # Arguments
    /// * `command_name` - Name of the command
    /// * `description` - Command description
    /// * `example` - Usage example
    /// 
    /// # Errors
    /// Returns an error if the metadata update fails
    pub async fn add_metadata(&self, command_name: String, description: String, example: String) -> Result<()> {
        let mut metadata = self.metadata.write().await;
        metadata.insert(command_name, (description, example));
        Ok(())
    }

    /// Gets suggestions based on partial input
    /// 
    /// # Arguments
    /// * `partial_input` - Partial command input
    /// * `context` - Optional context string to improve suggestions
    /// 
    /// # Returns
    /// A vector of command suggestions sorted by relevance
    /// 
    /// # Panics
    /// This function will panic if the comparison between relevance scores fails,
    /// which should never happen as we're comparing f32 values
    pub async fn get_suggestions(&self, partial_input: &str, context: Option<&str>) -> Vec<CommandSuggestion> {
        let mut suggestions = Vec::new();
        let metadata = self.metadata.read().await;

        // Get recent successful commands matching the partial input
        let recent = self.history.get_successful(partial_input, MAX_SUGGESTIONS).await;
        for entry in recent {
            let relevance = calculate_relevance(&entry.command_name, partial_input, context);
            if let Some((desc, example)) = metadata.get(&entry.command_name) {
                suggestions.push(CommandSuggestion {
                    command_name: entry.command_name,
                    relevance,
                    description: Some(desc.clone()),
                    example: Some(example.clone()),
                });
            } else {
                suggestions.push(CommandSuggestion {
                    command_name: entry.command_name,
                    relevance,
                    description: None,
                    example: None,
                });
            }
        }

        // Sort by relevance - we know this won't fail as we're comparing f32 values
        suggestions.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal));
        suggestions.truncate(MAX_SUGGESTIONS);
        suggestions
    }

    /// Gets suggestions based on command usage patterns
    /// 
    /// # Arguments
    /// * `last_command` - Last executed command
    /// * `limit` - Maximum number of suggestions
    /// 
    /// # Returns
    /// A vector of command suggestions based on common patterns
    pub async fn get_pattern_suggestions(&self, last_command: &str, limit: usize) -> Vec<CommandSuggestion> {
        let mut suggestions = Vec::new();
        let metadata = self.metadata.read().await;

        // Get commands that commonly follow the last command
        let history = self.history.get_recent(100).await;
        let mut patterns = HashMap::new();

        for window in history.windows(2) {
            if window[0].command_name == last_command {
                *patterns.entry(window[1].command_name.clone()).or_insert(0) += 1;
            }
        }

        // Convert patterns to suggestions
        let mut pattern_vec: Vec<_> = patterns.into_iter().collect();
        pattern_vec.sort_by(|a, b| b.1.cmp(&a.1));

        for (command_name, count) in pattern_vec.into_iter().take(limit) {
            let relevance = f64::from(count) / 100.0;
            if let Some((desc, example)) = metadata.get(&command_name) {
                suggestions.push(CommandSuggestion {
                    command_name,
                    relevance,
                    description: Some(desc.clone()),
                    example: Some(example.clone()),
                });
            } else {
                suggestions.push(CommandSuggestion {
                    command_name,
                    relevance,
                    description: None,
                    example: None,
                });
            }
        }

        suggestions
    }
}

/// Calculates relevance score for a command suggestion
fn calculate_relevance(command: &str, input: &str, context: Option<&str>) -> f64 {
    let mut score: f64 = 0.0;

    // Exact prefix match
    if command.starts_with(input) {
        score += 0.5;
    }

    // Contains input
    if command.contains(input) {
        score += 0.3;
    }

    // Context match
    if let Some(ctx) = context {
        if command.contains(ctx) {
            score += 0.2;
        }
    }

    score.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use squirrel_commands::{Command, CommandError};
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

    #[tokio::test]
    async fn test_suggestions() {
        let history = Arc::new(CommandHistory::new());
        let suggestions = CommandSuggestions::new(Arc::clone(&history));

        // Add some commands to history
        let cmd1 = TestCommand::new("test_command1");
        let cmd2 = TestCommand::new("test_command2");

        history.record(&cmd1, true, None).await.unwrap();
        history.record(&cmd2, true, None).await.unwrap();

        // Add metadata
        suggestions.add_metadata(
            "test_command1".to_string(),
            "Test command 1".to_string(),
            "test_command1 arg1".to_string(),
        ).await.unwrap();

        // Get suggestions
        let results = suggestions.get_suggestions("test", None).await;
        assert!(!results.is_empty());
        assert!(results[0].relevance > 0.0);
    }

    #[tokio::test]
    async fn test_pattern_suggestions() {
        let history = Arc::new(CommandHistory::new());
        let suggestions = CommandSuggestions::new(Arc::clone(&history));

        // Add command sequence to history
        let cmd1 = TestCommand::new("command1");
        let cmd2 = TestCommand::new("command2");

        // Record a pattern of command1 followed by command2
        for _ in 0..3 {
            history.record(&cmd1, true, None).await.unwrap();
            history.record(&cmd2, true, None).await.unwrap();
        }

        // Get pattern suggestions
        let results = suggestions.get_pattern_suggestions("command1", 5).await;
        assert!(!results.is_empty());
        assert_eq!(results[0].command_name, "command2");
    }

    #[tokio::test]
    async fn test_relevance_calculation() {
        assert!(calculate_relevance("test_command", "test", None) > 0.0);
        assert!(calculate_relevance("command", "test", None) == 0.0);
        assert!(calculate_relevance("test_command", "test", Some("command")) > 0.0);
    }
} 