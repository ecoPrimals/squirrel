//! Tests for the command suggestions system

use std::sync::Arc;
use tempfile::tempdir;

use crate::history::CommandHistory;
use crate::suggestions::{CommandSuggestions, SuggestionContext};

#[test]
fn test_suggestions_integration() {
    // Create a temporary directory for history file
    let dir = tempdir().unwrap();
    let history_file = dir.path().join("test-history.json");
    
    // Create history and suggestions systems
    let history = Arc::new(CommandHistory::with_options(100, &history_file).unwrap());
    let suggestions = CommandSuggestions::new(Arc::clone(&history));
    
    // Add some command history
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
    
    history.add(
        "ls".to_string(),
        vec!["-la".to_string()],
        true,
        None,
        None,
    ).unwrap();
    
    // Update suggestion patterns
    suggestions.update_patterns().unwrap();
    
    // Test basic suggestions
    let context = SuggestionContext::default();
    let basic_suggestions = suggestions.get_suggestions(&context).unwrap();
    
    // Should have at least git and ls commands
    assert!(basic_suggestions.len() >= 2);
    
    // Test command completion
    let completions = suggestions.get_completions("g").unwrap();
    assert!(completions.contains(&"git".to_string()));
    
    // Test contextual suggestions
    let context = SuggestionContext::default()
        .with_previous_command("git status");
        
    let contextual_suggestions = suggestions.get_suggestions(&context).unwrap();
    
    // The git commit command should be suggested after git status
    let has_git_commit = contextual_suggestions.iter().any(|s| 
        s.command == "git" && s.common_args.contains(&"commit".to_string())
    );
    
    assert!(has_git_commit);
    
    // Test argument suggestions
    let arg_suggestions = suggestions.get_argument_suggestions("git").unwrap();
    
    // Should contain valid argument patterns for git
    let has_valid_args = arg_suggestions.iter().any(|args| 
        args.contains(&"status".to_string()) || 
        args.contains(&"commit".to_string())
    );
    
    assert!(has_valid_args);
}

#[test]
fn test_suggestion_scoring() {
    // Create a temporary directory for history file
    let dir = tempdir().unwrap();
    let history_file = dir.path().join("score-test-history.json");
    
    // Create history and suggestions systems
    let history = Arc::new(CommandHistory::with_options(100, &history_file).unwrap());
    let suggestions = CommandSuggestions::new(Arc::clone(&history));
    
    // Add some command history with a pattern
    // First add 'git status' command
    history.add(
        "git".to_string(),
        vec!["status".to_string()],
        true,
        None,
        None,
    ).unwrap();
    
    // Then add 'git commit' command to establish a sequence
    history.add(
        "git".to_string(),
        vec!["commit".to_string(), "-m".to_string(), "update".to_string()],
        true,
        None,
        None,
    ).unwrap();
    
    // Repeat the pattern a few times to establish a stronger correlation
    history.add(
        "git".to_string(),
        vec!["status".to_string()],
        true,
        None,
        None,
    ).unwrap();
    
    history.add(
        "git".to_string(),
        vec!["commit".to_string(), "-m".to_string(), "another update".to_string()],
        true,
        None,
        None,
    ).unwrap();
    
    // Add some other commands to add noise
    history.add(
        "ls".to_string(),
        vec!["-la".to_string()],
        true,
        None,
        None,
    ).unwrap();
    
    history.add(
        "cd".to_string(),
        vec!["..".to_string()],
        true,
        None,
        None,
    ).unwrap();
    
    // Update suggestion patterns
    suggestions.update_patterns().unwrap();
    
    // Test contextual suggestions after 'git status'
    let context = SuggestionContext::default()
        .with_previous_command("git status");
        
    let context_suggestions = suggestions.get_suggestions(&context).unwrap();
    
    // Get the suggestion for 'git commit'
    let git_commit_suggestion = context_suggestions.iter()
        .find(|s| s.command == "git" && s.common_args.contains(&"commit".to_string()));
    
    // Verify the suggestion exists and has a good context score
    assert!(git_commit_suggestion.is_some());
    let suggestion = git_commit_suggestion.unwrap();
    
    // The context score should be relatively high due to the established pattern
    assert!(suggestion.score.context_score > 0.0);
}

#[test]
fn test_suggestions_with_partial_command() {
    // Create a temporary directory for history file
    let dir = tempdir().unwrap();
    let history_file = dir.path().join("partial-test-history.json");
    
    // Create history and suggestions systems
    let history = Arc::new(CommandHistory::with_options(100, &history_file).unwrap());
    let suggestions = CommandSuggestions::new(Arc::clone(&history));
    
    // Add various commands
    history.add("git".to_string(), vec!["status".to_string()], true, None, None).unwrap();
    history.add("grep".to_string(), vec!["pattern".to_string(), "file.txt".to_string()], true, None, None).unwrap();
    history.add("ls".to_string(), vec!["-la".to_string()], true, None, None).unwrap();
    history.add("find".to_string(), vec![".".to_string(), "-name".to_string(), "*.txt".to_string()], true, None, None).unwrap();
    
    // Update suggestion patterns
    suggestions.update_patterns().unwrap();
    
    // Test partial command completion for 'g'
    let context = SuggestionContext::default()
        .with_partial_command("g");
        
    let partial_suggestions = suggestions.get_suggestions(&context).unwrap();
    
    // Should contain git and grep, but not ls or find
    let has_git = partial_suggestions.iter().any(|s| s.command == "git");
    let has_grep = partial_suggestions.iter().any(|s| s.command == "grep");
    let has_ls = partial_suggestions.iter().any(|s| s.command == "ls");
    let has_find = partial_suggestions.iter().any(|s| s.command == "find");
    
    assert!(has_git);
    assert!(has_grep);
    assert!(!has_ls);
    assert!(!has_find);
} 