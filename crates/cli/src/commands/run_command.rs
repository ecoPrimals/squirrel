use std::error::Error;
use std::path::Path;
use std::fs;
use std::process::Command as ProcessCommand;

use clap::{Command as ClapCommand, Arg};
use log::{info, error};
use commands::{Command, CommandResult};
use crate::commands::error::CommandError;
use std::sync::Arc;

use crate::commands::ExecutionContext;

/// Command for running external commands
#[derive(Debug, Clone)]
pub struct RunCommand {
    // No state needed for this command
}

impl Default for RunCommand {
    fn default() -> Self {
        Self {}
    }
}

impl RunCommand {
    /// Create a new run command
    pub fn new() -> Self {
        Self {}
    }

    /// Get the clap parser for the run command
    pub fn create_parser(&self) -> ClapCommand {
        // Using hardcoded strings to avoid lifetime issues
        ClapCommand::new("run")
            .about("Run external commands")
            .arg(Arg::new("cmd")
                .help("Command to execute")
                .required(true)
                .index(1))
            .arg(Arg::new("args")
                .help("Arguments to pass to the command")
                .num_args(1..)
                .index(2))
    }

    /// Run multiple commands and combine their output
    pub fn run_commands(&self, commands: &[&str]) -> Result<String, String> {
        let mut results = Vec::new();
        
        for cmd in commands {
            let output = ProcessCommand::new(cmd)
                .output()
                .map_err(|e| format!("Failed to execute {}: {}", cmd, e))?;
                
            if !output.status.success() {
                return Err(format!("Command {} failed: {}", 
                    cmd, 
                    String::from_utf8_lossy(&output.stderr)));
            }
            
            results.push(String::from_utf8_lossy(&output.stdout).to_string());
        }
        
        Ok(results.join("\n\n"))
    }

    /// Execute a script file
    async fn run_script(&self, script_path: &str, _args: &[String], dry_run: bool, exec_context: Arc<ExecutionContext>) -> Result<String, Box<dyn Error>> {
        let path = Path::new(script_path);
        
        // Check if file exists
        if !path.exists() {
            return Err(format!("Script file not found: {}", script_path).into());
        }
        
        // Check file extension
        if let Some(ext) = path.extension() {
            if ext != "sq" {
                return Err(format!("Unsupported script file type: {}", ext.to_string_lossy()).into());
            }
        } else {
            return Err(format!("Script file has no extension: {}", script_path).into());
        }
        
        // Read script file
        let script_content = fs::read_to_string(path)?;
        
        // Process script - each line is a command to execute
        let mut results = Vec::new();
        
        for (line_num, line) in script_content.lines().enumerate() {
            // Skip empty lines and comments
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Parse line into command and arguments
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            
            let command_name = parts[0].to_string();
            let command_args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
            
            // Log execution
            info!("Executing command from script: {} (line {})", command_name, line_num + 1);
            
            if dry_run {
                // Just log what would be executed without actually running it
                results.push(format!("Would execute: {} {}", command_name, command_args.join(" ")));
            } else {
                // Execute command
                match exec_context.execute_command_with_args(&command_name, command_args.clone()).await {
                    Ok(output) => {
                        results.push(format!("Command '{}' output:\n{}", command_name, output));
                    },
                    Err(e) => {
                        error!("Command '{}' failed: {}", command_name, e);
                        return Err(Box::new(e));
                    }
                }
            }
        }
        
        Ok(results.join("\n\n"))
    }
}

impl Command for RunCommand {
    fn name(&self) -> &str {
        "run"
    }
    
    fn description(&self) -> &str {
        "Run external commands"
    }
    
    fn execute(&self, args: &[String]) -> CommandResult<String> {
        if args.is_empty() {
            return Err(CommandError::InvalidArguments("No command specified".to_string()).into());
        }
        
        let cmd = &args[0];
        let cmd_args = &args[1..];
        
        info!("Executing command: {} with args: {:?}", cmd, cmd_args);
        
        let mut command = ProcessCommand::new(cmd);
        command.args(cmd_args);
        
        match command.output() {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(stdout.to_string())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(CommandError::ExecutionFailed(stderr.to_string()).into())
                }
            },
            Err(e) => {
                error!("Failed to execute command: {}", e);
                Err(CommandError::ExecutionFailed(e.to_string()).into())
            }
        }
    }
    
    fn help(&self) -> String {
        "Run external commands\n\nUsage:\n  run <command> [args...]".to_string()
    }
    
    fn parser(&self) -> ClapCommand {
        self.create_parser()
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_name() {
        let cmd = RunCommand::new();
        assert_eq!(cmd.name(), "run");
    }
    
    #[test]
    fn test_help() {
        let cmd = RunCommand::new();
        let help = cmd.help();
        assert!(help.contains("Run external commands"));
    }
    
    // More tests would be implemented for script execution
    // These would require mocking the execution context
} 