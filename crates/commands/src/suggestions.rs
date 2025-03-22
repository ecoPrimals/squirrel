//! Command suggestions system for Squirrel
//!
//! This module provides functionality for generating context-aware command suggestions,
//! intelligent command completion, usage hints, and learning from user patterns.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Timelike;
use tracing::{debug, info};
use serde::{Deserialize, Serialize};

use crate::CommandError;
use crate::history::CommandHistory;

/// Result type for suggestion operations
pub type SuggestionResult<T> = Result<T, CommandError>;

/// Weights for different suggestion factors
const WEIGHT_RECENCY: f64 = 0.5;
const WEIGHT_FREQUENCY: f64 = 0.3;
const WEIGHT_CONTEXT: f64 = 0.2;

/// Maximum age in seconds to consider for recency score (7 days)
const MAX_AGE_SECONDS: u64 = 60 * 60 * 24 * 7;

/// Default confidence threshold for suggestions
const DEFAULT_CONFIDENCE_THRESHOLD: f64 = 0.2;

/// Default number of suggestions to return
const DEFAULT_MAX_SUGGESTIONS: usize = 5;

/// Confidence score for a suggestion
/// 
/// This represents how confident the system is that the suggestion is relevant
/// to the current context. Higher values indicate higher confidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuggestionScore {
    /// Overall confidence score (0.0 to 1.0)
    pub confidence: f64,
    
    /// Recency component of the score
    pub recency_score: f64,
    
    /// Frequency component of the score
    pub frequency_score: f64,
    
    /// Context-relevance component of the score
    pub context_score: f64,
}

impl SuggestionScore {
    /// Creates a new suggestion score with the given components
    pub fn new(recency_score: f64, frequency_score: f64, context_score: f64) -> Self {
        // Calculate weighted confidence
        let confidence = (recency_score * WEIGHT_RECENCY) +
                         (frequency_score * WEIGHT_FREQUENCY) +
                         (context_score * WEIGHT_CONTEXT);
                         
        Self {
            confidence,
            recency_score,
            frequency_score,
            context_score,
        }
    }
    
    /// Checks if this score meets the given confidence threshold
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.confidence >= threshold
    }
}

/// A command suggestion with context-aware metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSuggestion {
    /// The command name
    pub command: String,
    
    /// Common arguments for this command based on history
    pub common_args: Vec<String>,
    
    /// Usage hint or example
    pub usage_hint: Option<String>,
    
    /// Description of the command
    pub description: Option<String>,
    
    /// Confidence score for this suggestion
    pub score: SuggestionScore,
}

impl CommandSuggestion {
    /// Creates a new command suggestion
    pub fn new(
        command: String,
        common_args: Vec<String>,
        usage_hint: Option<String>,
        description: Option<String>,
        score: SuggestionScore,
    ) -> Self {
        Self {
            command,
            common_args,
            usage_hint,
            description,
            score,
        }
    }
    
    /// Returns a formatted string representation of this suggestion
    pub fn formatted(&self) -> String {
        let args_str = if self.common_args.is_empty() {
            String::new()
        } else {
            format!(" {}", self.common_args.join(" "))
        };
        
        let hint = match &self.usage_hint {
            Some(hint) => format!(" - {}", hint),
            None => String::new(),
        };
        
        format!("{}{}{}", self.command, args_str, hint)
    }
}

/// Context information used to generate relevant suggestions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SuggestionContext {
    /// Previous command in the sequence (if any)
    pub previous_command: Option<String>,
    
    /// Current working directory
    pub current_directory: Option<String>,
    
    /// Time of day (as hour in 24-hour format)
    pub hour_of_day: Option<u8>,
    
    /// Current project context (if any)
    pub project: Option<String>,
    
    /// Any partial command the user has started typing
    pub partial_command: Option<String>,
    
    /// Additional contextual metadata
    pub metadata: HashMap<String, String>,
}

impl SuggestionContext {
    /// Creates a new empty suggestion context
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Creates a suggestion context with the current time of day
    pub fn with_current_time() -> Self {
        let now = chrono::Local::now();
        Self {
            hour_of_day: Some(now.hour() as u8),
            ..Default::default()
        }
    }
    
    /// Sets the previous command
    pub fn with_previous_command(mut self, command: impl Into<String>) -> Self {
        self.previous_command = Some(command.into());
        self
    }
    
    /// Sets the current directory
    pub fn with_current_directory(mut self, dir: impl Into<String>) -> Self {
        self.current_directory = Some(dir.into());
        self
    }
    
    /// Sets the current project
    pub fn with_project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }
    
    /// Sets the partial command
    pub fn with_partial_command(mut self, partial: impl Into<String>) -> Self {
        self.partial_command = Some(partial.into());
        self
    }
    
    /// Adds metadata to the context
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Manages command suggestions based on history and context
#[derive(Debug)]
pub struct CommandSuggestions {
    /// Command history for generating suggestions
    history: Arc<CommandHistory>,
    
    /// Command description cache
    command_descriptions: Arc<RwLock<HashMap<String, String>>>,
    
    /// Command sequence patterns (which commands are often run after others)
    command_sequences: Arc<RwLock<HashMap<String, HashMap<String, usize>>>>,
    
    /// Common argument patterns for commands
    argument_patterns: Arc<RwLock<HashMap<String, Vec<Vec<String>>>>>,
    
    /// Confidence threshold for suggestions
    confidence_threshold: f64,
    
    /// Maximum number of suggestions to return
    max_suggestions: usize,
}

impl CommandSuggestions {
    /// Creates a new command suggestions manager with default settings
    pub fn new(history: Arc<CommandHistory>) -> Self {
        Self {
            history,
            command_descriptions: Arc::new(RwLock::new(HashMap::new())),
            command_sequences: Arc::new(RwLock::new(HashMap::new())),
            argument_patterns: Arc::new(RwLock::new(HashMap::new())),
            confidence_threshold: DEFAULT_CONFIDENCE_THRESHOLD,
            max_suggestions: DEFAULT_MAX_SUGGESTIONS,
        }
    }
    
    /// Creates a new command suggestions manager with custom settings
    pub fn with_options(
        history: Arc<CommandHistory>,
        confidence_threshold: f64,
        max_suggestions: usize,
    ) -> Self {
        Self {
            history,
            command_descriptions: Arc::new(RwLock::new(HashMap::new())),
            command_sequences: Arc::new(RwLock::new(HashMap::new())),
            argument_patterns: Arc::new(RwLock::new(HashMap::new())),
            confidence_threshold,
            max_suggestions,
        }
    }
    
    /// Adds a command description
    pub fn add_command_description(&self, command: impl Into<String>, description: impl Into<String>) -> SuggestionResult<()> {
        let mut descriptions = self.command_descriptions.write().map_err(|e| {
            CommandError::ResourceError(format!("Failed to acquire write lock: {}", e))
        })?;
        
        descriptions.insert(command.into(), description.into());
        Ok(())
    }
    
    /// Gets suggestions based on the given context
    pub fn get_suggestions(&self, context: &SuggestionContext) -> SuggestionResult<Vec<CommandSuggestion>> {
        // Get command history
        let history_entries = self.history.get_last(100)?;
        
        // If there's a partial command, filter for commands that start with it
        let command_filter = context.partial_command.as_ref().map(|p| p.to_lowercase());
        
        // Build frequency map
        let mut frequency_map: HashMap<String, usize> = HashMap::new();
        let mut last_used_map: HashMap<String, u64> = HashMap::new();
        let mut command_set: HashSet<String> = HashSet::new();
        
        // Process history entries
        for entry in &history_entries {
            let command = entry.command.clone();
            
            // Skip if doesn't match partial command filter
            if let Some(ref partial) = command_filter {
                if !command.to_lowercase().starts_with(partial) {
                    continue;
                }
            }
            
            // Update frequency count
            *frequency_map.entry(command.clone()).or_insert(0) += 1;
            
            // Update last used timestamp (keep the most recent)
            last_used_map.entry(command.clone())
                .and_modify(|ts| *ts = (*ts).max(entry.timestamp))
                .or_insert(entry.timestamp);
            
            // Add to command set
            command_set.insert(command.clone());
            
            // Update command sequences
            if let Some(ref prev_cmd) = context.previous_command {
                let mut sequences = self.command_sequences.write().map_err(|e| {
                    CommandError::ResourceError(format!("Failed to acquire write lock: {}", e))
                })?;
                
                sequences.entry(prev_cmd.clone())
                    .or_insert_with(HashMap::new)
                    .entry(command.clone())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
            
            // Update argument patterns
            let mut arg_patterns = self.argument_patterns.write().map_err(|e| {
                CommandError::ResourceError(format!("Failed to acquire write lock: {}", e))
            })?;
            
            arg_patterns.entry(command.clone())
                .or_insert_with(Vec::new)
                .push(entry.args.clone());
        }
        
        // Get current timestamp
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        // Generate suggestions
        let mut suggestions = Vec::new();
        
        // Access descriptions
        let descriptions = self.command_descriptions.read().map_err(|e| {
            CommandError::ResourceError(format!("Failed to acquire read lock: {}", e))
        })?;
        
        // Calculate max frequency for normalization
        let max_frequency = frequency_map.values().max().cloned().unwrap_or(1);
        
        // Access command sequences and arg patterns
        let sequences = self.command_sequences.read().map_err(|e| {
            CommandError::ResourceError(format!("Failed to acquire read lock: {}", e))
        })?;
        
        let arg_patterns = self.argument_patterns.read().map_err(|e| {
            CommandError::ResourceError(format!("Failed to acquire read lock: {}", e))
        })?;
        
        // Process each command in the history
        for command in command_set {
            // Calculate recency score (higher for more recent commands)
            let last_used = last_used_map.get(&command).cloned().unwrap_or(0);
            let age_seconds = current_time.saturating_sub(last_used);
            let recency_score = if age_seconds > MAX_AGE_SECONDS {
                0.0
            } else {
                1.0 - (age_seconds as f64 / MAX_AGE_SECONDS as f64)
            };
            
            // Calculate frequency score (higher for more frequent commands)
            let frequency = *frequency_map.get(&command).unwrap_or(&0);
            let frequency_score = frequency as f64 / max_frequency as f64;
            
            // Calculate context score based on various factors
            let mut context_score = 0.0;
            
            // Factor 1: Command sequence patterns
            if let Some(ref prev_cmd) = context.previous_command {
                if let Some(next_commands) = sequences.get(prev_cmd) {
                    if let Some(count) = next_commands.get(&command) {
                        // Normalize by total count for the previous command
                        let total = next_commands.values().sum::<usize>() as f64;
                        context_score += (*count as f64 / total) * 0.5;
                    }
                }
            }
            
            // Factor 2: Time of day patterns
            // Additional context scoring could be added here
            
            // Factor 3: Current directory relevance
            // Additional context scoring could be added here
            
            // Calculate overall score
            let score = SuggestionScore::new(recency_score, frequency_score, context_score);
            
            // Only include suggestions that meet the threshold
            if score.meets_threshold(self.confidence_threshold) {
                // Find common args for this command
                let common_args = if let Some(args_list) = arg_patterns.get(&command) {
                    // Simple approach: use the most recent arguments
                    if !args_list.is_empty() {
                        args_list[0].clone()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };
                
                // Generate usage hint
                let usage_hint = if !common_args.is_empty() {
                    Some(format!("Example: {} {}", command, common_args.join(" ")))
                } else {
                    None
                };
                
                // Get description if available
                let description = descriptions.get(&command).cloned();
                
                // Create suggestion
                let suggestion = CommandSuggestion::new(
                    command,
                    common_args,
                    usage_hint,
                    description,
                    score,
                );
                
                suggestions.push(suggestion);
            }
        }
        
        // Sort by confidence score (highest first)
        suggestions.sort_by(|a, b| b.score.confidence.partial_cmp(&a.score.confidence).unwrap_or(std::cmp::Ordering::Equal));
        
        // Limit to max suggestions
        suggestions.truncate(self.max_suggestions);
        
        Ok(suggestions)
    }
    
    /// Gets command completion suggestions for a partial command
    pub fn get_completions(&self, partial: &str) -> SuggestionResult<Vec<String>> {
        let context = SuggestionContext::default().with_partial_command(partial);
        let suggestions = self.get_suggestions(&context)?;
        
        Ok(suggestions.into_iter().map(|s| s.command).collect())
    }
    
    /// Gets argument suggestions for a specific command
    pub fn get_argument_suggestions(&self, command: &str) -> SuggestionResult<Vec<Vec<String>>> {
        let arg_patterns = self.argument_patterns.read().map_err(|e| {
            CommandError::ResourceError(format!("Failed to acquire read lock: {}", e))
        })?;
        
        match arg_patterns.get(command) {
            Some(patterns) => Ok(patterns.clone()),
            None => Ok(Vec::new()),
        }
    }
    
    /// Updates suggestion patterns based on recent history
    pub fn update_patterns(&self) -> SuggestionResult<()> {
        info!("Updating command suggestion patterns");
        
        // Get recent history entries
        let history_entries = self.history.get_last(500)?;
        
        // Update command sequences
        let mut sequences = HashMap::new();
        let mut prev_command: Option<String> = None;
        
        for entry in &history_entries {
            if let Some(prev) = prev_command {
                sequences.entry(prev)
                    .or_insert_with(HashMap::new)
                    .entry(entry.command.clone())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
            
            prev_command = Some(entry.command.clone());
        }
        
        // Update the shared sequences map
        let mut shared_sequences = self.command_sequences.write().map_err(|e| {
            CommandError::ResourceError(format!("Failed to acquire write lock: {}", e))
        })?;
        
        *shared_sequences = sequences;
        
        // Update argument patterns
        let mut arg_patterns = HashMap::new();
        
        for entry in &history_entries {
            arg_patterns.entry(entry.command.clone())
                .or_insert_with(Vec::new)
                .push(entry.args.clone());
        }
        
        // Limit patterns per command (keep only the N most recent)
        for patterns in arg_patterns.values_mut() {
            if patterns.len() > 10 {
                patterns.truncate(10);
            }
        }
        
        // Update the shared patterns map
        let mut shared_patterns = self.argument_patterns.write().map_err(|e| {
            CommandError::ResourceError(format!("Failed to acquire write lock: {}", e))
        })?;
        
        *shared_patterns = arg_patterns;
        
        debug!("Updated suggestion patterns from {} history entries", history_entries.len());
        Ok(())
    }
    
    /// Sets the confidence threshold for suggestions
    pub fn set_confidence_threshold(&mut self, threshold: f64) {
        self.confidence_threshold = threshold;
    }
    
    /// Sets the maximum number of suggestions to return
    pub fn set_max_suggestions(&mut self, max: usize) {
        self.max_suggestions = max;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::CommandHistory;
    use tempfile::tempdir;
    
    fn create_test_history() -> Arc<CommandHistory> {
        let dir = tempdir().unwrap();
        let history_file = dir.path().join("test-history.json");
        
        let history = Arc::new(CommandHistory::with_options(100, &history_file).unwrap());
        
        // Add some test entries
        history.add(
            "find".to_string(),
            vec!["file.txt".to_string()],
            true,
            None,
            None,
        ).unwrap();
        
        history.add(
            "grep".to_string(),
            vec!["pattern".to_string(), "file.txt".to_string()],
            true,
            None,
            None,
        ).unwrap();
        
        history.add(
            "find".to_string(),
            vec!["another.txt".to_string()],
            true,
            None,
            None,
        ).unwrap();
        
        history.add(
            "git".to_string(),
            vec!["status".to_string()],
            true,
            None,
            None,
        ).unwrap();
        
        history.add(
            "git".to_string(),
            vec!["commit".to_string(), "-m".to_string(), "update".to_string()],
            true,
            None,
            None,
        ).unwrap();
        
        // Add a command sequence pattern: git status followed by git commit
        history.add(
            "git".to_string(),
            vec!["status".to_string()],
            true,
            None,
            None,
        ).unwrap();
        
        history.add(
            "git".to_string(),
            vec!["commit".to_string(), "-m".to_string(), "update".to_string()],
            true,
            None,
            None,
        ).unwrap();
        
        history
    }
    
    #[test]
    fn test_suggestion_creation() {
        let score = SuggestionScore::new(0.8, 0.5, 0.3);
        assert!(score.confidence > 0.0);
        
        let suggestion = CommandSuggestion::new(
            "git".to_string(),
            vec!["status".to_string()],
            Some("Check repository status".to_string()),
            Some("Git status command".to_string()),
            score,
        );
        
        assert_eq!(suggestion.command, "git");
        assert_eq!(suggestion.common_args, vec!["status".to_string()]);
        assert!(suggestion.usage_hint.is_some());
        assert!(suggestion.description.is_some());
    }
    
    #[test]
    fn test_context_creation() {
        let context = SuggestionContext::default()
            .with_previous_command("git status")
            .with_current_directory("/home/user/project")
            .with_partial_command("git");
            
        assert_eq!(context.previous_command, Some("git status".to_string()));
        assert_eq!(context.current_directory, Some("/home/user/project".to_string()));
        assert_eq!(context.partial_command, Some("git".to_string()));
    }
    
    #[test]
    fn test_basic_suggestions() {
        let history = create_test_history();
        let suggestions = CommandSuggestions::new(history);
        
        // Update patterns
        suggestions.update_patterns().unwrap();
        
        // Get suggestions with no context
        let context = SuggestionContext::default();
        let results = suggestions.get_suggestions(&context).unwrap();
        
        // Should have some suggestions
        assert!(!results.is_empty());
    }
    
    #[test]
    fn test_completion_suggestions() {
        let history = create_test_history();
        let suggestions = CommandSuggestions::new(history);
        
        // Update patterns
        suggestions.update_patterns().unwrap();
        
        // Get completions for "g"
        let completions = suggestions.get_completions("g").unwrap();
        
        // Should contain "git"
        assert!(completions.contains(&"git".to_string()));
        
        // Should not contain "find"
        assert!(!completions.contains(&"find".to_string()));
    }
    
    #[test]
    fn test_context_aware_suggestions() {
        let history = create_test_history();
        let suggestions = CommandSuggestions::new(history);
        
        // Update patterns
        suggestions.update_patterns().unwrap();
        
        // Create context with previous command
        let context = SuggestionContext::default()
            .with_previous_command("git status");
            
        let results = suggestions.get_suggestions(&context).unwrap();
        
        // Git commit should be one of the high-ranking suggestions
        let has_git_commit = results.iter().any(|s| 
            s.command == "git" && s.common_args.contains(&"commit".to_string())
        );
        
        assert!(has_git_commit);
    }
    
    #[test]
    fn test_argument_suggestions() {
        let history = create_test_history();
        let suggestions = CommandSuggestions::new(history);
        
        // Update patterns
        suggestions.update_patterns().unwrap();
        
        // Get argument suggestions for "git"
        let arg_suggestions = suggestions.get_argument_suggestions("git").unwrap();
        
        // Should have some argument patterns
        assert!(!arg_suggestions.is_empty());
        
        // Should contain "status" in at least one pattern
        let has_status = arg_suggestions.iter().any(|args|
            args.contains(&"status".to_string())
        );
        
        assert!(has_status);
    }
} 
