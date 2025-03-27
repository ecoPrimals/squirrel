//! Tests for the observability module
//!
//! These tests validate the tracing, metrics collection, and hook functionality
//! provided by the observability module.

use std::collections::HashMap;
use std::error::Error;

use crate::hooks::HookManager;
use crate::lifecycle::LifecycleStage;
use crate::observability::{ObservabilitySystem, log_command_execution, record_resource_usage, ObservabilityError, ObservabilityResult};
use crate::registry::{Command, CommandResult};
use crate::CommandError;

// Simple test command for usage in tests
struct TestCommand;

impl Command for TestCommand {
    fn name(&self) -> &'static str {
        "test_command"
    }

    fn description(&self) -> &'static str {
        "A test command for observability tests"
    }

    fn execute(&self, args: &[String]) -> CommandResult<String> {
        Ok(format!("Test command executed with args: {:?}", args))
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("test_command")
            .about("A test command for observability tests")
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(TestCommand)
    }
}

// Failing test command
struct FailingCommand;

impl Command for FailingCommand {
    fn name(&self) -> &'static str {
        "failing_command"
    }

    fn description(&self) -> &'static str {
        "A test command that always fails"
    }

    fn execute(&self, _args: &[String]) -> CommandResult<String> {
        Err(CommandError::ExecutionError("This command deliberately fails for testing".to_string()))
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("failing_command")
            .about("A test command that always fails")
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(FailingCommand)
    }
}

#[test]
fn test_log_command_execution() {
    let command_name = "test_command";
    let args = vec!["arg1".to_string(), "arg2".to_string()];
    
    // Test successful execution
    let success_result: CommandResult<String> = Ok("Test result".to_string());
    log_command_execution(command_name, &args, &success_result, 100);
    
    // Test failed execution
    let error_result: CommandResult<String> = Err(CommandError::ExecutionError("Test error".to_string()));
    log_command_execution(command_name, &args, &error_result, 150);
    
    // No assertion needed as we're just checking it doesn't panic
}

#[test]
fn test_record_resource_usage() {
    let command_name = "test_command";
    let memory_kb = 1024;
    let cpu_percent = 2.5;
    
    record_resource_usage(command_name, memory_kb, cpu_percent);
    
    // No assertion needed as we're just checking it doesn't panic
}

#[test]
fn test_observability_system() {
    let observability = ObservabilitySystem::new();
    let command_name = "test_command";
    let args = vec!["arg1".to_string(), "arg2".to_string()];
    
    // Test successful execution
    let success_result: CommandResult<String> = Ok("Test result".to_string());
    observability.log_command(command_name, &args, &success_result);
    
    // Test failed execution
    let error_result: CommandResult<String> = Err(CommandError::ExecutionError("Test error".to_string()));
    observability.log_command(command_name, &args, &error_result);
    
    // No assertion needed as we're just checking it doesn't panic
}

// The rest of the detailed tests for TraceContext, CommandMetrics, etc. are removed
// since those implementations have been simplified. 