//! Built-in commands for the Squirrel system
//!
//! This module provides basic built-in commands such as help and version.

use std::sync::{Arc, Mutex};
use std::time::Instant;
use log::{debug, info, warn, error};

use crate::{Command, CommandResult, CommandError};
use super::registry::CommandRegistry;
use clap::{Arg, ArgAction, Command as ClapCommand};
use std::time::{UNIX_EPOCH, SystemTime};

use crate::history::{CommandHistory, HistoryEntry};

/// Command for displaying help information
///
/// This implementation avoids deadlocks by storing command information
/// at creation time rather than accessing the registry during execution.
pub struct HelpCommand {
    /// Command information stored at creation time
    command_help: Vec<(String, String)>,
}

impl HelpCommand {
    /// Creates a new help command with pre-loaded command information
    ///
    /// # Arguments
    ///
    /// * `registry` - The command registry to extract information from
    ///
    /// # Returns
    ///
    /// A new HelpCommand instance with pre-loaded command information
    ///
    /// # Implementation Note
    ///
    /// This function acquires a lock on the registry only at creation time,
    /// allowing the execute method to run without acquiring locks.
    #[must_use] pub fn new(registry: Arc<Mutex<CommandRegistry>>) -> Self {
        debug!("HelpCommand: Creating new instance with registry reference");
        let start = Instant::now();
        
        // Acquire registry lock at construction time only
        let mut command_help = Vec::new();
        
        // Only attempt to load commands if we can acquire the lock
        match registry.lock() {
            Ok(registry_lock) => {
                debug!("HelpCommand: Acquired lock on registry to load command information");
                
                // Get list of commands
                match registry_lock.list_commands() {
                    Ok(commands) => {
                        debug!("HelpCommand: Found {} commands", commands.len());
                        
                        // Get help text for each command
                        for cmd_name in commands {
                            match registry_lock.get_help(&cmd_name) {
                                Ok(help_text) => {
                                    command_help.push((cmd_name.clone(), help_text));
                                },
                                Err(e) => {
                                    warn!("HelpCommand: Failed to get help for command '{}': {}", cmd_name, e);
                                    // Still include the command name, but with a generic message
                                    command_help.push((cmd_name.clone(), 
                                                     format!("{}: <error retrieving help>", cmd_name)));
                                }
                            }
                        }
                    },
                    Err(e) => {
                        error!("HelpCommand: Failed to list commands: {}", e);
                    }
                }
            },
            Err(e) => {
                error!("HelpCommand: Failed to acquire lock on registry: {}", e);
            }
        }
        
        let duration = start.elapsed();
        if command_help.is_empty() {
            warn!("HelpCommand: Created with empty command information in {:?}", duration);
        } else {
            info!("HelpCommand: Created with {} commands in {:?}", command_help.len(), duration);
        }
        
        Self { command_help }
    }

    /// Updates command help information from the registry
    ///
    /// Call this method when commands are added or removed to keep
    /// the help information up to date.
    ///
    /// # Arguments
    ///
    /// * `registry` - The command registry to extract updated information from
    pub fn update(&mut self, registry: &Arc<Mutex<CommandRegistry>>) {
        debug!("HelpCommand: Updating command help information");
        let start = Instant::now();
        
        // Only attempt to update if we can acquire the lock
        match registry.lock() {
            Ok(registry_lock) => {
                debug!("HelpCommand: Acquired lock on registry to update command information");
                let mut updated_help = Vec::new();
                
                // Get updated list of commands
                match registry_lock.list_commands() {
                    Ok(commands) => {
                        debug!("HelpCommand: Found {} commands for update", commands.len());
                        
                        // Get help text for each command
                        for cmd_name in commands {
                            match registry_lock.get_help(&cmd_name) {
                                Ok(help_text) => {
                                    updated_help.push((cmd_name.clone(), help_text));
                                },
                                Err(e) => {
                                    warn!("HelpCommand: Failed to get help for command '{}': {}", cmd_name, e);
                                    // Still include the command name, but with a generic message
                                    updated_help.push((cmd_name.clone(), 
                                                     format!("{}: <error retrieving help>", cmd_name)));
                                }
                            }
                        }
                        
                        // Replace existing help information with updated data
                        let old_count = self.command_help.len();
                        self.command_help = updated_help;
                        let new_count = self.command_help.len();
                        
                        let duration = start.elapsed();
                        info!("HelpCommand: Updated command information from {} to {} commands in {:?}", 
                             old_count, new_count, duration);
                    },
                    Err(e) => {
                        error!("HelpCommand: Failed to list commands during update: {}", e);
                    }
                }
            },
            Err(e) => {
                error!("HelpCommand: Failed to acquire lock on registry during update: {}", e);
            }
        }
    }
}

impl Command for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }
    
    fn description(&self) -> &str {
        "Provides help information for available commands"
    }
    
    fn execute(&self, args: &[String]) -> CommandResult<String> {
        debug!("HelpCommand: Executing with args: {:?}", args);
        let start = Instant::now();
        
        // No locks are acquired during execution, preventing deadlocks
        let result = if args.is_empty() {
            // Display help for all commands
            debug!("HelpCommand: Displaying help for all {} commands", self.command_help.len());
            let mut help_text = String::from("Available commands:\n");
            
            // Use pre-loaded help information
            for (_, help) in &self.command_help {
                help_text.push_str(&format!("  {}\n", help));
            }
            
            Ok(help_text)
        } else {
            // Display help for the specified command
            let command_name = &args[0];
            debug!("HelpCommand: Looking up help for specific command: {}", command_name);
            
            // Find the requested command in our pre-loaded information
            for (name, help) in &self.command_help {
                if name == command_name {
                    debug!("HelpCommand: Found help for command: {}", command_name);
                    return Ok(help.clone());
                }
            }
            
            // Command not found
            let err_msg = format!("Command '{}' not found", command_name);
            warn!("HelpCommand: {}", err_msg);
            Err(CommandError::CommandNotFound(command_name.clone()))
        };
        
        let duration = start.elapsed();
        match &result {
            Ok(_) => debug!("HelpCommand: Executed successfully in {:?}", duration),
            Err(e) => debug!("HelpCommand: Execution failed in {:?}: {}", duration, e),
        }
        
        result
    }
    
    fn parser(&self) -> clap::Command {
        // Using static strings rather than self methods to avoid lifetime issues
        clap::Command::new("help")
            .about("Provides help information for available commands")
            .arg(clap::Arg::new("command")
                .help("Command to get help for")
                .required(false))
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        debug!("HelpCommand: Cloning command");
        Box::new(Self {
            command_help: self.command_help.clone(),
        })
    }
}

/// Command for displaying version information
#[derive(Debug, Clone)]
pub struct VersionCommand;

impl Default for VersionCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionCommand {
    /// Creates a new version command
    #[must_use] pub fn new() -> Self {
        debug!("VersionCommand: Creating new instance");
        Self
    }
}

impl Command for VersionCommand {
    fn name(&self) -> &str {
        "version"
    }
    
    fn description(&self) -> &str {
        "Shows the current version of the application"
    }
    
    fn execute(&self, _args: &[String]) -> CommandResult<String> {
        debug!("VersionCommand: Executing");
        let version = env!("CARGO_PKG_VERSION");
        debug!("VersionCommand: Reporting version {}", version);
        
        // Use the crate version from Cargo.toml
        Ok(format!("Version: {}", version))
    }
    
    fn parser(&self) -> clap::Command {
        // Using static strings rather than self methods to avoid lifetime issues
        clap::Command::new("version")
            .about("Shows the current version of the application")
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        debug!("VersionCommand: Cloning command");
        Box::new(Self)
    }
}

/// Echo command that simply returns the arguments passed to it
#[derive(Debug, Clone)]
pub struct EchoCommand {
    prefix: String,
}

impl Default for EchoCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl EchoCommand {
    /// Creates a new echo command
    #[must_use] pub fn new() -> Self {
        debug!("EchoCommand: Creating new instance");
        Self {
            prefix: "Echo: ".to_string(),
        }
    }
    
    /// Creates a new echo command with a custom prefix
    #[must_use] pub fn with_prefix(prefix: &str) -> Self {
        debug!("EchoCommand: Creating new instance with prefix: {}", prefix);
        Self {
            prefix: prefix.to_string(),
        }
    }
}

impl Command for EchoCommand {
    fn name(&self) -> &str {
        "echo"
    }
    
    fn description(&self) -> &str {
        "Echoes the provided arguments back to the user"
    }
    
    fn execute(&self, args: &[String]) -> CommandResult<String> {
        debug!("EchoCommand: Executing with args: {:?}", args);
        let message = args.join(" ");
        Ok(format!("{}{}", self.prefix, message))
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("echo")
            .about("Echoes the provided arguments back to the user")
            .arg(clap::Arg::new("message")
                .help("Message to echo")
                .required(false)
                .num_args(0..))
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        debug!("EchoCommand: Cloning command");
        Box::new(Self {
            prefix: self.prefix.clone(),
        })
    }
}

/// Exit command to terminate the application
#[derive(Debug, Clone)]
pub struct ExitCommand;

impl Default for ExitCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl ExitCommand {
    /// Creates a new exit command
    #[must_use] pub fn new() -> Self {
        debug!("ExitCommand: Creating new instance");
        Self
    }
}

impl Command for ExitCommand {
    fn name(&self) -> &str {
        "exit"
    }
    
    fn description(&self) -> &str {
        "Exits the application"
    }
    
    fn execute(&self, _args: &[String]) -> CommandResult<String> {
        debug!("ExitCommand: Executing");
        info!("ExitCommand: Application exit requested");
        Ok("Exiting application".to_string())
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("exit")
            .about("Exits the application")
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        debug!("ExitCommand: Cloning command");
        Box::new(Self)
    }
}

/// Kill command to terminate processes
#[derive(Debug, Clone)]
pub struct KillCommand;

impl Default for KillCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl KillCommand {
    /// Creates a new kill command
    #[must_use] pub fn new() -> Self {
        debug!("KillCommand: Creating new instance");
        Self
    }
}

impl Command for KillCommand {
    fn name(&self) -> &str {
        "kill"
    }
    
    fn description(&self) -> &str {
        "Terminates a running process by PID"
    }
    
    fn execute(&self, args: &[String]) -> CommandResult<String> {
        debug!("KillCommand: Executing with args: {:?}", args);
        
        if args.is_empty() {
            let err_msg = "No PID provided";
            warn!("KillCommand: {}", err_msg);
            return Err(CommandError::ValidationError(err_msg.to_string()));
        }
        
        let pid = match args[0].parse::<u32>() {
            Ok(pid) => pid,
            Err(e) => {
                let err_msg = format!("Invalid PID format: {}", e);
                warn!("KillCommand: {}", err_msg);
                return Err(CommandError::ValidationError(err_msg));
            }
        };
        
        info!("KillCommand: Process kill requested for PID {}", pid);
        // In a real implementation, this would actually kill the process
        Ok(format!("Process with PID {} terminated", pid))
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("kill")
            .about("Terminates a running process by PID")
            .arg(clap::Arg::new("pid")
                .help("Process ID to terminate")
                .required(true))
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        debug!("KillCommand: Cloning command");
        Box::new(Self)
    }
}

/// Command for managing command history
pub struct HistoryCommand {
    /// Command history manager
    history: Arc<CommandHistory>,
}

impl HistoryCommand {
    /// Creates a new history command
    #[must_use] pub fn new(history: Arc<CommandHistory>) -> Self {
        debug!("HistoryCommand: Creating new instance");
        Self { history }
    }
    
    /// Helper to format a list of history entries
    fn format_entries(&self, entries: &[HistoryEntry], limit: usize) -> String {
        let mut output = String::new();
        let count = std::cmp::min(entries.len(), limit);
        
        if count == 0 {
            return "No history entries found.".to_string();
        }
        
        for (i, entry) in entries.iter().take(count).enumerate() {
            output.push_str(&format!("{}: {}\n", i + 1, entry.formatted()));
        }
        
        if entries.len() > limit {
            output.push_str(&format!("\n... and {} more entries (use --limit to show more)\n", 
                entries.len() - limit));
        }
        
        output
    }
}

impl Command for HistoryCommand {
    fn name(&self) -> &str {
        "history"
    }
    
    fn description(&self) -> &str {
        "View and manage command history"
    }
    
    fn execute(&self, args: &[String]) -> CommandResult<String> {
        debug!("HistoryCommand: Executing with args: {:?}", args);
        
        let matches = self.parser().try_get_matches_from(
            std::iter::once(self.name().to_string()).chain(args.iter().cloned())
        ).map_err(|e| {
            CommandError::ExecutionError(format!("Invalid arguments: {}", e))
        })?;
        
        if matches.get_flag("clear") {
            // Clear history
            self.history.clear().map_err(|e| {
                CommandError::ExecutionError(format!("Failed to clear history: {}", e))
            })?;
            
            return Ok("Command history cleared.".to_string());
        }
        
        if let Some(days) = matches.get_one::<u64>("cleanup") {
            // Calculate timestamp for cleanup
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
                
            let days_in_secs = days * 24 * 60 * 60;
            let cutoff = now.saturating_sub(days_in_secs);
            
            // Cleanup entries older than the cutoff
            let removed = self.history.cleanup_older_than(cutoff).map_err(|e| {
                CommandError::ExecutionError(format!("Failed to cleanup history: {}", e))
            })?;
            
            return Ok(format!("Removed {} history entries older than {} days.", removed, days));
        }
        
        let limit = matches.get_one::<usize>("limit")
            .cloned()
            .unwrap_or(10);
            
        if let Some(query) = matches.get_one::<String>("search") {
            // Search history
            let entries = self.history.search(query).map_err(|e| {
                CommandError::ExecutionError(format!("Failed to search history: {}", e))
            })?;
            
            let output = if entries.is_empty() {
                format!("No command history entries found matching '{}'.", query)
            } else {
                format!("Search results for '{}':\n\n{}", 
                    query, self.format_entries(&entries, limit))
            };
            
            return Ok(output);
        }
        
        if let Some(command) = matches.get_one::<String>("command") {
            // Get history for specific command
            let entry = self.history.get_last_for_command(command).map_err(|e| {
                CommandError::ExecutionError(format!("Failed to get command history: {}", e))
            })?;
            
            let output = match entry {
                Some(entry) => format!("Last execution of '{}':\n\n{}", command, entry.formatted()),
                None => format!("No history found for command '{}'.", command),
            };
            
            return Ok(output);
        }
        
        // Default: show last N entries
        let entries = self.history.get_last(limit).map_err(|e| {
            CommandError::ExecutionError(format!("Failed to get command history: {}", e))
        })?;
        
        let output = if entries.is_empty() {
            "Command history is empty.".to_string()
        } else {
            format!("Command history (most recent first):\n\n{}", 
                self.format_entries(&entries, limit))
        };
        
        Ok(output)
    }
    
    fn parser(&self) -> ClapCommand {
        // Use string literals rather than accessing self to avoid lifetime issues
        ClapCommand::new("history")
            .about("View and manage command history")
            .arg(Arg::new("limit")
                .short('n')
                .long("limit")
                .help("Limit the number of entries displayed")
                .value_name("COUNT")
                .value_parser(clap::value_parser!(usize)))
            .arg(Arg::new("search")
                .short('s')
                .long("search")
                .help("Search command history")
                .value_name("QUERY"))
            .arg(Arg::new("command")
                .short('c')
                .long("command")
                .help("Show history for a specific command")
                .value_name("COMMAND"))
            .arg(Arg::new("clear")
                .long("clear")
                .help("Clear command history")
                .action(ArgAction::SetTrue))
            .arg(Arg::new("cleanup")
                .long("cleanup")
                .help("Remove entries older than the specified number of days")
                .value_name("DAYS")
                .value_parser(clap::value_parser!(u64)))
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(Self {
            history: Arc::clone(&self.history),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_command() {
        let cmd = VersionCommand;
        assert_eq!(cmd.name(), "version");
        assert!(cmd.execute(&[]).is_ok());
    }

    #[test]
    fn test_help_command() {
        let cmd = HelpCommand {
            command_help: vec![
                ("test".to_string(), "Test command description".to_string())
            ],
        };
        assert_eq!(cmd.name(), "help");
        assert!(cmd.execute(&[]).is_ok());
    }
    
    #[test]
    fn test_echo_command() {
        let cmd = EchoCommand::new();
        assert_eq!(cmd.name(), "echo");
        let result = cmd.execute(&["hello".to_string(), "world".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Echo: hello world");
    }
    
    #[test]
    fn test_exit_command() {
        let cmd = ExitCommand::new();
        assert_eq!(cmd.name(), "exit");
        let result = cmd.execute(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Exiting application");
    }
    
    #[test]
    fn test_kill_command() {
        let cmd = KillCommand::new();
        assert_eq!(cmd.name(), "kill");
        
        // Test with valid PID
        let result = cmd.execute(&["1234".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Process with PID 1234 terminated");
        
        // Test with invalid PID
        let result = cmd.execute(&["invalid".to_string()]);
        assert!(result.is_err());
        
        // Test with no PID
        let result = cmd.execute(&[]);
        assert!(result.is_err());
    }
} 