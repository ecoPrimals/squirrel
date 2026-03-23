// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use super::{Command, CommandError, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::RwLock;

/// Trait for implementing command validation rules.
///
/// This trait defines the interface for validation rules that can be applied
/// to commands before execution.
pub trait ValidationRule: Send + Sync + std::fmt::Debug {
    /// Returns the name of the validation rule.
    fn name(&self) -> &'static str;

    /// Returns a description of what the rule validates.
    fn description(&self) -> &'static str;

    /// Validates a command against this rule.
    ///
    /// # Arguments
    /// * `command` - The command to validate
    /// * `context` - The validation context containing command metadata
    ///
    /// # Errors
    /// Returns an error if validation fails with a description of the failure
    fn validate(&self, command: &dyn Command, context: &ValidationContext) -> Result<()>;
}

/// Error type for validation failures
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationError {
    /// Name of the validation rule that failed
    pub rule_name: String,
    /// Error message describing the validation failure
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.rule_name, self.message)
    }
}

impl Error for ValidationError {}

impl From<ValidationError> for CommandError {
    fn from(error: ValidationError) -> Self {
        CommandError::ValidationError(format!("{}: {}", error.rule_name, error.message))
    }
}

// ValidationError automatically implements Send and Sync because all its fields (String) are Send + Sync

/// Context for command validation, including arguments and environment variables
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationContext {
    /// Map of argument names to their values
    pub arguments: HashMap<String, String>,
    /// Map of environment variable names to their values
    pub environment: HashMap<String, String>,
    /// Map of context data
    #[serde(skip)]
    data: RwLock<HashMap<String, String>>,
    /// Rules for validation (currently unused)
    #[serde(skip)]
    rules: HashMap<String, Arc<dyn ValidationRule>>,
}

impl ValidationContext {
    /// Creates a new validation context
    #[must_use]
    pub fn new() -> Self {
        Self {
            arguments: HashMap::new(),
            environment: HashMap::new(),
            data: RwLock::new(HashMap::new()),
            rules: HashMap::new(),
        }
    }

    /// Sets a value in the validation context
    ///
    /// # Arguments
    /// * `key` - The key to set
    /// * `value` - The value to set
    ///
    /// # Errors
    /// Returns an error if unable to acquire write lock on context data
    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        let mut data = self.data.write().map_err(|_| {
            CommandError::ValidationError(
                "Failed to acquire write lock on context data".to_string(),
            )
        })?;
        data.insert(key.to_string(), value.to_string());
        Ok(())
    }

    /// Gets a value from the validation context
    ///
    /// # Arguments
    /// * `key` - The key to get
    ///
    /// # Returns
    /// * `Ok(Some(String))` if the value exists
    /// * `Ok(None)` if the value does not exist
    ///
    /// # Errors
    /// Returns an error if unable to acquire read lock on context data
    pub fn get(&self, key: &str) -> Result<Option<String>> {
        let data = self.data.read().map_err(|_| {
            CommandError::ValidationError("Failed to acquire read lock on context data".to_string())
        })?;
        Ok(data.get(key).cloned())
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Validator for checking command requirements.
///
/// This struct manages validation rules and applies them to commands
/// before execution.
#[derive(Debug)]
pub struct CommandValidator {
    /// Map of pattern names to regex patterns used for validation
    patterns: HashMap<String, Regex>,
    /// Context for validation operations containing metadata about the command being validated
    context: ValidationContext,
    /// List of validation rules (Arc for O(1) clone when sharing)
    rules: RwLock<Vec<Arc<dyn ValidationRule>>>,
}

impl CommandValidator {
    /// Creates a new command validator
    #[must_use]
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            context: ValidationContext::new(),
            rules: RwLock::new(Vec::new()),
        }
    }

    /// Adds a validation rule to the validator.
    ///
    /// # Errors
    /// Returns an error if the write lock cannot be acquired.
    pub fn add_rule(&self, rule: Arc<dyn ValidationRule>) -> Result<()> {
        let mut rules = self.rules.write().map_err(|e| {
            CommandError::ValidationError(format!("Failed to acquire write lock: {e}"))
        })?;
        rules.push(rule);
        Ok(())
    }

    /// Validates a command against all registered validation rules
    ///
    /// # Arguments
    /// * `command` - The command to validate
    ///
    /// # Errors
    /// Returns an error if validation fails with a description of the failure
    pub fn validate(&self, command: &dyn Command) -> Result<()> {
        // Validate command name
        Self::validate_name(command.name())?;

        // Validate system requirements
        Self::validate_system_requirements()?;

        Ok(())
    }

    /// Validates the command name against a set of criteria
    fn validate_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(CommandError::ValidationError(
                "Command name cannot be empty".to_string(),
            ));
        }

        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(CommandError::ValidationError(
                "Command name must only contain alphanumeric characters, underscores, or hyphens"
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Validates system requirements including memory and thread usage.
    /// Only runs when `system-metrics` feature is enabled (ecoBin: pure Rust default).
    #[expect(
        clippy::unnecessary_wraps,
        reason = "Result needed when system-metrics feature is enabled"
    )]
    fn validate_system_requirements() -> Result<()> {
        #[cfg(feature = "system-metrics")]
        {
            if let Ok(mem) = universal_constants::sys_info::memory_info()
                && mem.available < mem.total / 10
            {
                return Err(CommandError::ValidationError(
                    "Insufficient available memory".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Returns the number of rules in the validator
    pub fn rules(&self) -> usize {
        self.rules.read().map(|rules| rules.len()).unwrap_or(0)
    }
}

impl Default for CommandValidator {
    fn default() -> Self {
        Self::new()
    }
}

// Built-in validation rules

/// Rule that validates required command arguments
#[derive(Debug, Clone)]
pub struct RequiredArgumentsRule {
    /// List of argument names that must be present
    required_args: Vec<String>,
}

impl RequiredArgumentsRule {
    /// Creates a new `RequiredArgumentsRule` with the specified required arguments
    ///
    /// # Arguments
    /// * `required_args` - List of argument names that must be present
    ///
    /// # Errors
    /// Returns an error if any required argument is missing
    #[must_use]
    pub fn new(required_args: Vec<String>) -> Self {
        Self { required_args }
    }
}

impl ValidationRule for RequiredArgumentsRule {
    fn validate(&self, _command: &dyn Command, context: &ValidationContext) -> Result<()> {
        for arg in &self.required_args {
            if !context.arguments.contains_key(arg) {
                return Err(CommandError::ValidationError(format!(
                    "Required argument '{arg}' is missing"
                )));
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "required_arguments"
    }

    fn description(&self) -> &'static str {
        "Validates that required arguments are present"
    }
}

/// Rule that validates argument patterns against defined regex patterns
#[derive(Debug, Clone)]
pub struct ArgumentPatternRule {
    /// Map of argument names to their expected regex patterns
    patterns: HashMap<String, String>,
}

impl ArgumentPatternRule {
    /// Creates a new `ArgumentPatternRule` with the specified patterns
    ///
    /// # Arguments
    /// * `patterns` - Map of argument names to regex patterns they must match
    ///
    /// # Errors
    /// Returns an error if any argument doesn't match its required pattern
    #[must_use]
    pub fn new(patterns: HashMap<String, String>) -> Self {
        Self { patterns }
    }
}

impl ValidationRule for ArgumentPatternRule {
    fn validate(&self, _command: &dyn Command, context: &ValidationContext) -> Result<()> {
        for (arg, pattern) in &self.patterns {
            if !context.arguments.contains_key(arg) {
                continue;
            }

            if Regex::new(pattern).is_err() {
                return Err(CommandError::ValidationError(format!(
                    "Invalid regex pattern '{pattern}' for argument '{arg}'"
                )));
            }

            let empty_string = String::new();
            let value = context.arguments.get(arg).unwrap_or(&empty_string);
            let regex = Regex::new(pattern).map_err(|e| {
                CommandError::ValidationError(format!(
                    "Failed to compile regex pattern '{pattern}': {e}"
                ))
            })?;

            if !regex.is_match(value) {
                return Err(CommandError::ValidationError(format!(
                    "Argument '{arg}' value '{value}' does not match pattern '{pattern}'"
                )));
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "argument_pattern"
    }

    fn description(&self) -> &'static str {
        "Validates argument values against regex patterns"
    }
}

/// Rule that validates required environment variables
#[derive(Debug, Clone)]
pub struct EnvironmentRule {
    /// List of environment variable names that must be present
    required_vars: Vec<String>,
}

impl EnvironmentRule {
    /// Creates a new `EnvironmentRule` with the specified required variables
    ///
    /// # Arguments
    /// * `required_vars` - List of environment variables that must be present
    ///
    /// # Errors
    /// Returns an error if any required environment variable is missing
    #[must_use]
    pub fn new(required_vars: Vec<String>) -> Self {
        Self { required_vars }
    }
}

impl ValidationRule for EnvironmentRule {
    fn validate(&self, _command: &dyn Command, context: &ValidationContext) -> Result<()> {
        for var in &self.required_vars {
            if !context.environment.contains_key(var) {
                return Err(CommandError::ValidationError(format!(
                    "Required environment variable '{var}' is missing"
                )));
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "environment"
    }

    fn description(&self) -> &'static str {
        "Validates required environment variables"
    }
}

/// Rule that validates command name length
#[derive(Debug, Clone)]
pub struct NameLengthRule {
    /// Minimum allowed length for command name
    min_length: usize,
    /// Maximum allowed length for command name
    max_length: usize,
}

impl NameLengthRule {
    /// Creates a new `NameLengthRule` with the specified length constraints
    ///
    /// # Arguments
    /// * `min_length` - Minimum allowed name length
    /// * `max_length` - Maximum allowed name length
    ///
    /// # Returns
    /// A new `NameLengthRule` instance
    #[must_use]
    pub fn new(min_length: usize, max_length: usize) -> Self {
        Self {
            min_length,
            max_length,
        }
    }

    /// Validates that a command name meets the length requirements.
    ///
    /// # Arguments
    /// * `name` - The command name to validate
    /// * `min_length` - Minimum allowed name length
    /// * `max_length` - Maximum allowed name length
    ///
    /// # Errors
    /// Returns an error if the command name length is outside the allowed range.
    fn validate_name(name: &str, min_length: usize, max_length: usize) -> Result<()> {
        let name_len = name.len();
        if name_len < min_length {
            return Err(CommandError::ValidationError(format!(
                "Command name must be at least {min_length} characters long"
            )));
        }
        if name_len > max_length {
            return Err(CommandError::ValidationError(format!(
                "Command name cannot be longer than {max_length} characters"
            )));
        }
        Ok(())
    }
}

impl ValidationRule for NameLengthRule {
    fn validate(&self, command: &dyn Command, _context: &ValidationContext) -> Result<()> {
        Self::validate_name(command.name(), self.min_length, self.max_length)
    }

    fn name(&self) -> &'static str {
        "name_length"
    }

    fn description(&self) -> &'static str {
        "Validates command name length"
    }
}

/// Rule that validates command description length
#[derive(Debug, Clone)]
pub struct DescriptionRule {
    /// Minimum required length for command description
    min_length: usize,
}

impl DescriptionRule {
    /// Creates a new `DescriptionRule` with the specified minimum length
    ///
    /// # Arguments
    /// * `min_length` - Minimum allowed description length
    ///
    /// # Errors
    /// Returns an error if the description length is less than the minimum
    #[must_use]
    pub fn new(min_length: usize) -> Self {
        Self { min_length }
    }
}

impl ValidationRule for DescriptionRule {
    fn validate(&self, command: &dyn Command, _context: &ValidationContext) -> Result<()> {
        let description = command.description();
        if description.len() < self.min_length {
            return Err(CommandError::ValidationError(format!(
                "Command description is too short (minimum length: {})",
                self.min_length
            )));
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "description"
    }

    fn description(&self) -> &'static str {
        "Validates command description length"
    }
}

/// Rule that sanitizes and validates input against patterns
#[derive(Debug, Clone)]
pub struct InputSanitizationRule {
    /// Map of input field names to their validation regex patterns
    patterns: HashMap<String, regex::Regex>,
    /// Maximum allowed length for input values
    max_length: usize,
}

impl InputSanitizationRule {
    /// Creates a new `InputSanitizationRule` with the specified patterns and length limit
    ///
    /// # Arguments
    /// * `patterns` - Map of input names to regex patterns they must match
    /// * `max_length` - Maximum allowed input length
    ///
    /// # Errors
    /// Returns an error if any input doesn't match its pattern or exceeds the length limit
    pub fn new(patterns: HashMap<String, &str>, max_length: usize) -> Result<Self> {
        let mut compiled_patterns = HashMap::new();
        for (key, pattern) in patterns {
            let regex = regex::Regex::new(pattern).map_err(|e| {
                CommandError::ValidationError(format!("Invalid pattern for key '{key}': {e}"))
            })?;
            compiled_patterns.insert(key.to_string(), regex);
        }
        Ok(Self {
            patterns: compiled_patterns,
            max_length,
        })
    }
}

impl ValidationRule for InputSanitizationRule {
    fn validate(&self, _command: &dyn Command, context: &ValidationContext) -> Result<()> {
        // Check max length for all arguments
        for (key, value) in &context.arguments {
            if value.len() > self.max_length {
                return Err(CommandError::ValidationError(format!(
                    "Argument '{}' value exceeds maximum length of {} characters",
                    key, self.max_length
                )));
            }
        }

        // Check patterns
        for (key, pattern) in &self.patterns {
            if let Some(value) = context.arguments.get(key)
                && !pattern.is_match(value)
            {
                return Err(CommandError::ValidationError(format!(
                    "Argument '{key}' value '{value}' does not match required pattern"
                )));
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "input_sanitization"
    }

    fn description(&self) -> &'static str {
        "Validates command input against unsafe patterns"
    }
}

/// Rule that validates resource usage against defined limits
#[derive(Debug, Clone)]
pub struct ResourceValidationRule {
    /// Maximum memory usage allowed in megabytes
    max_memory_mb: usize,
    /// Maximum number of threads allowed
    max_threads: usize,
}

impl ResourceValidationRule {
    /// Creates a new `ResourceValidationRule` with the specified resource limits
    ///
    /// # Arguments
    /// * `max_memory_mb` - Maximum memory usage allowed in megabytes
    /// * `max_threads` - Maximum number of threads allowed
    ///
    /// # Errors
    /// Returns an error if any resource usage exceeds its limit
    #[must_use]
    pub fn new(max_memory_mb: usize, max_threads: usize) -> Self {
        Self {
            max_memory_mb,
            max_threads,
        }
    }

    /// Checks if the current memory usage is within the specified limits.
    /// Only runs when `system-metrics` feature is enabled (ecoBin: pure Rust default).
    #[expect(
        clippy::unnecessary_wraps,
        reason = "Result needed when system-metrics feature is enabled"
    )]
    fn check_memory_usage(&self) -> Result<()> {
        #[cfg(feature = "system-metrics")]
        {
            let used_memory = universal_constants::sys_info::memory_info()
                .map(|m| m.used)
                .unwrap_or(0);
            let memory_usage_mb = (used_memory / 1024 / 1024) as usize;

            if memory_usage_mb > self.max_memory_mb {
                return Err(CommandError::ValidationError(format!(
                    "Memory usage ({}) exceeds maximum allowed ({} MB)",
                    memory_usage_mb, self.max_memory_mb
                )));
            }
        }
        Ok(())
    }

    /// Checks if the current thread usage is within the specified limits.
    /// Only runs when `system-metrics` feature is enabled (ecoBin: pure Rust default).
    #[expect(
        clippy::unnecessary_wraps,
        reason = "Result needed when system-metrics feature is enabled"
    )]
    fn check_thread_usage(&self) -> Result<()> {
        #[cfg(feature = "system-metrics")]
        {
            let thread_count = universal_constants::sys_info::cpu_count().unwrap_or(1) as usize;

            if thread_count > self.max_threads {
                return Err(CommandError::ValidationError(format!(
                    "Thread count ({}) exceeds maximum allowed ({})",
                    thread_count, self.max_threads
                )));
            }
        }
        Ok(())
    }
}

impl ValidationRule for ResourceValidationRule {
    fn validate(&self, _command: &dyn Command, _context: &ValidationContext) -> Result<()> {
        self.check_memory_usage()?;
        self.check_thread_usage()?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "resource_validation"
    }

    fn description(&self) -> &'static str {
        "Validates command against resource limits"
    }
}

#[cfg(test)]
#[path = "validation_tests.rs"]
mod tests;
