// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::Command;
use crate::CommandResult;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct TestCommand;

impl Command for TestCommand {
    fn name(&self) -> &'static str {
        "test"
    }

    fn description(&self) -> &'static str {
        "A test command"
    }

    fn parser(&self) -> clap::Command {
        clap::Command::new("test").about("A test command")
    }

    fn execute(&self, _args: &[String]) -> CommandResult<String> {
        Ok("Test command executed".to_string())
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
struct ShortNameCommand;

impl Command for ShortNameCommand {
    fn name(&self) -> &'static str {
        "t"
    }

    fn description(&self) -> &'static str {
        "A test command"
    }

    fn parser(&self) -> clap::Command {
        clap::Command::new("t").about("A test command")
    }

    fn execute(&self, _args: &[String]) -> CommandResult<String> {
        Ok("Test command executed".to_string())
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
struct LongNameCommand;

impl Command for LongNameCommand {
    fn name(&self) -> &'static str {
        "verylongname"
    }

    fn description(&self) -> &'static str {
        "A test command"
    }

    fn parser(&self) -> clap::Command {
        clap::Command::new("verylongname").about("A test command")
    }

    fn execute(&self, _args: &[String]) -> CommandResult<String> {
        Ok("Long name command executed".to_string())
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
struct ShortDescCommand;

impl Command for ShortDescCommand {
    fn name(&self) -> &'static str {
        "test"
    }

    fn description(&self) -> &'static str {
        "test"
    }

    fn parser(&self) -> clap::Command {
        clap::Command::new("test").about("test")
    }

    fn execute(&self, _args: &[String]) -> CommandResult<String> {
        Ok("Short description command executed".to_string())
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

#[test]
fn test_name_length_rule() {
    let rule = NameLengthRule::new(3, 10);
    let command = TestCommand;
    assert!(rule.validate(&command, &ValidationContext::new()).is_ok());

    let command = ShortNameCommand;
    assert!(rule.validate(&command, &ValidationContext::new()).is_err());

    let command = LongNameCommand;
    assert!(rule.validate(&command, &ValidationContext::new()).is_err());
}

#[test]
fn test_description_rule() {
    let rule = DescriptionRule::new(5);
    let command = TestCommand;
    assert!(rule.validate(&command, &ValidationContext::new()).is_ok());

    let command = ShortDescCommand;
    assert!(rule.validate(&command, &ValidationContext::new()).is_err());
}

#[test]
fn test_validation_context() {
    let context = ValidationContext::new();
    context
        .set("test_key", "test_value")
        .expect("should succeed");
    assert_eq!(
        context
            .get("test_key")
            .expect("should succeed")
            .expect("should succeed"),
        "test_value"
    );
}

#[test]
fn test_input_sanitization() {
    let mut patterns = HashMap::new();
    patterns.insert(
        "email".to_string(),
        r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
    );
    let rule = InputSanitizationRule::new(patterns, 100).expect("should succeed");
    let mut context = ValidationContext::new();
    context
        .arguments
        .insert("email".to_string(), "test@example.com".to_string());
    let command = TestCommand;
    assert!(rule.validate(&command, &context).is_ok());
}

#[test]
#[cfg(feature = "system-metrics")]
fn test_resource_validation_rule() {
    let current_memory_mb = universal_constants::sys_info::memory_info()
        .map(|m| (m.used / 1024 / 1024) as usize)
        .unwrap_or(0);
    let current_threads = universal_constants::sys_info::cpu_count().unwrap_or(1) as usize;

    // Set limits higher than current usage
    let rule = ResourceValidationRule::new(current_memory_mb + 1024, current_threads + 10);
    let command = TestCommand;
    assert!(rule.validate(&command, &ValidationContext::new()).is_ok());
}

#[test]
fn test_validator_rules() {
    let validator = CommandValidator::new();
    let rule = NameLengthRule::new(3, 10);
    validator.add_rule(Arc::new(rule)).expect("should succeed");
    let command = TestCommand;
    assert!(validator.validate(&command).is_ok());
}

#[test]
fn test_thread_safety() {
    let context = Arc::new(ValidationContext::new());
    let mut handles = vec![];

    for i in 0..10 {
        let context = Arc::clone(&context);
        handles.push(std::thread::spawn(move || {
            context
                .set(&format!("key{i}"), &format!("value{i}"))
                .expect("should succeed");
        }));
    }

    for handle in handles {
        handle.join().expect("should succeed");
    }

    for i in 0..10 {
        assert_eq!(
            context.get(&format!("key{i}")).expect("should succeed"),
            Some(format!("value{i}"))
        );
    }
}

#[test]
fn test_required_arguments_rule() {
    let rule = RequiredArgumentsRule::new(vec!["arg1".to_string(), "arg2".to_string()]);
    let mut context = ValidationContext::new();
    let command = TestCommand;

    // Test missing arguments
    assert!(rule.validate(&command, &context).is_err());

    // Test with all required arguments
    context
        .arguments
        .insert("arg1".to_string(), "value1".to_string());
    context
        .arguments
        .insert("arg2".to_string(), "value2".to_string());
    assert!(rule.validate(&command, &context).is_ok());
}

#[test]
fn test_argument_pattern_rule() {
    let mut patterns = HashMap::new();
    patterns.insert(
        "email".to_string(),
        r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$".to_string(),
    );
    let rule = ArgumentPatternRule::new(patterns);
    let mut context = ValidationContext::new();
    let command = TestCommand;

    // Test invalid email
    context
        .arguments
        .insert("email".to_string(), "invalid-email".to_string());
    assert!(rule.validate(&command, &context).is_err());

    // Test valid email
    context
        .arguments
        .insert("email".to_string(), "test@example.com".to_string());
    assert!(rule.validate(&command, &context).is_ok());
}

#[test]
fn test_environment_rule() {
    let rule = EnvironmentRule::new(vec!["PATH".to_string(), "HOME".to_string()]);
    let mut context = ValidationContext::new();
    let command = TestCommand;

    // Test missing environment variables
    assert!(rule.validate(&command, &context).is_err());

    // Test with all required environment variables
    context
        .environment
        .insert("PATH".to_string(), "/usr/bin".to_string());
    context
        .environment
        .insert("HOME".to_string(), "/home/user".to_string());
    assert!(rule.validate(&command, &context).is_ok());
}

#[test]
fn test_input_sanitization_edge_cases() {
    let patterns = HashMap::new();
    let rule = InputSanitizationRule::new(patterns, 0).expect("should succeed");
    let mut context = ValidationContext::new();
    context
        .arguments
        .insert("test".to_string(), "value".to_string());
    let command = TestCommand;
    assert!(rule.validate(&command, &context).is_err());
}

#[derive(Debug, Clone)]
struct EmptyNameCommand;

impl Command for EmptyNameCommand {
    fn name(&self) -> &'static str {
        ""
    }

    fn description(&self) -> &'static str {
        "desc"
    }

    fn parser(&self) -> clap::Command {
        clap::Command::new("")
    }

    fn execute(&self, _args: &[String]) -> CommandResult<String> {
        Ok(String::new())
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
struct BadCharCommand;

impl Command for BadCharCommand {
    fn name(&self) -> &'static str {
        "bad name!"
    }

    fn description(&self) -> &'static str {
        "d"
    }

    fn parser(&self) -> clap::Command {
        clap::Command::new("bad")
    }

    fn execute(&self, _args: &[String]) -> CommandResult<String> {
        Ok(String::new())
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

#[test]
fn test_command_validator_rejects_empty_name() {
    let validator = CommandValidator::new();
    let cmd = EmptyNameCommand;
    assert!(validator.validate(&cmd).is_err());
}

#[test]
fn test_command_validator_rejects_invalid_name_chars() {
    let validator = CommandValidator::new();
    assert!(validator.validate(&BadCharCommand).is_err());
}

#[test]
fn test_argument_pattern_skips_missing_argument() {
    let mut patterns = HashMap::new();
    patterns.insert("opt".to_string(), r"^\d+$".to_string());
    let rule = ArgumentPatternRule::new(patterns);
    let context = ValidationContext::new();
    let command = TestCommand;
    assert!(rule.validate(&command, &context).is_ok());
}

#[test]
fn test_argument_pattern_invalid_regex_in_map() {
    let mut patterns = HashMap::new();
    patterns.insert("x".to_string(), r"(".to_string());
    let rule = ArgumentPatternRule::new(patterns);
    let mut context = ValidationContext::new();
    context.arguments.insert("x".to_string(), "1".to_string());
    let command = TestCommand;
    assert!(rule.validate(&command, &context).is_err());
}

#[test]
fn test_validation_context_get_missing() {
    let ctx = ValidationContext::new();
    assert_eq!(ctx.get("nope").expect("should succeed"), None);
}

#[test]
fn test_validation_error_display() {
    let err = ValidationError {
        rule_name: "r".to_string(),
        message: "m".to_string(),
    };
    assert_eq!(err.to_string(), "r: m");
}
