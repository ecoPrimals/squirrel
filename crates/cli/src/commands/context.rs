//! Command context for CLI command execution
//!
//! This module provides a context for command execution, including access
//! to parsed command-line arguments and other execution state.

use clap::ArgMatches;

/// Context for command execution
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Parsed command-line arguments
    matches: ArgMatches,
}

impl CommandContext {
    /// Create a new command context
    pub fn new(matches: ArgMatches) -> Self {
        Self { matches }
    }
    
    /// Get the parsed command-line arguments
    pub fn matches(&self) -> &ArgMatches {
        &self.matches
    }
} 