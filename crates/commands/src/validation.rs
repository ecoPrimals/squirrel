use std::collections::HashMap;
use std::error::Error;
use std::sync::RwLock;
#[allow(unused_imports)]
use std::sync::Arc;
use regex::Regex;
use sysinfo::System;
use serde::{Serialize, Deserialize};
use super::Command;
use crate::CommandResult;

/// Trait for implementing command validation rules.
/// 
/// This trait defines the interface for validation rules that can be applied
/// to commands before execution.
pub trait ValidationRule: Send + Sync + std::fmt::Debug {
    /// Returns the name of the validation rule.
    fn name(&self) -> &'static str;
    
    /// Returns a description of what the rule validates.
    #[allow(dead_code)]
    fn description(&self) -> &'static str;
    
    /// Validates a command against this rule.
    /// 
    /// # Arguments
    /// * `command` - The command to validate
    /// * `context` - The validation context containing command metadata
    /// 
    /// # Errors
    /// Returns an error if validation fails with a description of the failure
    #[allow(dead_code)]
    fn validate(&self, command: &dyn Command, context: &ValidationContext) -> Result<(), Box<dyn Error + Send + Sync>>;
    
    /// Clone the rule into a new Box.
    #[allow(dead_code)]
    fn clone_box(&self) -> Box<dyn ValidationRule>;
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

// Safety: ValidationError is just a simple error type that contains only primitive types
// and strings, so it's safe to share between threads.
unsafe impl Send for ValidationError {}

// Safety: ValidationError is just a simple error type that contains only primitive types
// and strings, so it's safe to access from multiple threads.
unsafe impl Sync for ValidationError {}

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
    #[allow(dead_code)]
    rules: HashMap<String, Box<dyn ValidationRule>>,
}

#[allow(dead_code)]
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
    pub fn set(&self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        let mut data = self.data.write().map_err(|_| {
            Box::new(ValidationError {
                rule_name: "context".to_string(),
                message: "Failed to acquire write lock on context data".to_string(),
            }) as Box<dyn Error>
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
    pub fn get(&self, key: &str) -> Result<Option<String>, Box<dyn Error>> {
        let data = self.data.read().map_err(|_| {
            Box::new(ValidationError {
                rule_name: "context".to_string(),
                message: "Failed to acquire read lock on context data".to_string(),
            }) as Box<dyn Error>
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
    #[allow(dead_code)]
    patterns: HashMap<String, Regex>,
    /// Context for validation operations containing metadata about the command being validated
    #[allow(dead_code)]
    context: ValidationContext,
    /// List of validation rules
    #[allow(dead_code)]
    rules: RwLock<Vec<Box<dyn ValidationRule>>>,
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
    #[allow(dead_code)]
    pub fn add_rule(&self, rule: Box<dyn ValidationRule>) -> Result<(), Box<dyn Error>> {
        let mut rules = self.rules.write().map_err(|e| Box::new(ValidationError {
            rule_name: "RuleAddition".to_string(),
            message: format!("Failed to acquire write lock: {e}"),
        }))?;
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
    pub fn validate(&self, command: &dyn Command) -> Result<(), Box<dyn Error>> {
        // Validate command name
        Self::validate_name(command.name())?;

        // Validate system requirements
        Self::validate_system_requirements()?;

        Ok(())
    }

    /// Validates the command name against a set of criteria
    fn validate_name(name: &str) -> Result<(), Box<dyn Error>> {
        if name.is_empty() {
            return Err(Box::new(ValidationError {
                rule_name: "name".to_string(),
                message: "Command name cannot be empty".to_string(),
            }));
        }

        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return Err(Box::new(ValidationError {
                rule_name: "name".to_string(),
                message: "Command name must only contain alphanumeric characters, underscores, or hyphens".to_string(),
            }));
        }

        Ok(())
    }

    /// Validates system requirements including memory and thread usage
    fn validate_system_requirements() -> Result<(), Box<dyn Error>> {
        let mut sys = System::new_all();
        sys.refresh_all();

        // Check available memory
        let total_memory = sys.total_memory();
        let available_memory = sys.available_memory();
        
        if available_memory < total_memory / 10 {
            return Err(Box::new(ValidationError {
                rule_name: "system".to_string(),
                message: "Insufficient available memory".to_string(),
            }));
        }

        Ok(())
    }

    /// Returns the number of rules in the validator
    #[allow(dead_code)]
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

#[allow(dead_code)]
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
    fn validate(&self, _command: &dyn Command, context: &ValidationContext) -> Result<(), Box<dyn Error + Send + Sync>> {
        for arg in &self.required_args {
            if !context.arguments.contains_key(arg) {
                return Err(Box::new(ValidationError {
                    rule_name: self.name().to_string(),
                    message: format!("Required argument '{arg}' is missing"),
                }));
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

    fn clone_box(&self) -> Box<dyn ValidationRule> {
        Box::new(self.clone())
    }
}

/// Rule that validates argument patterns against defined regex patterns
#[derive(Debug, Clone)]
pub struct ArgumentPatternRule {
    /// Map of argument names to their expected regex patterns
    patterns: HashMap<String, String>,
}

#[allow(dead_code)]
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
    fn validate(&self, _command: &dyn Command, context: &ValidationContext) -> Result<(), Box<dyn Error + Send + Sync>> {
        for (arg, pattern) in &self.patterns {
            if !context.arguments.contains_key(arg) {
                continue;
            }

            if Regex::new(pattern).is_err() {
                return Err(Box::new(ValidationError {
                    rule_name: self.name().to_string(),
                    message: format!("Invalid regex pattern '{pattern}' for argument '{arg}'"),
                }));
            }

            let empty_string = String::new();
            let value = context.arguments.get(arg).unwrap_or(&empty_string);
            let regex = Regex::new(pattern).map_err(|e| ValidationError {
                rule_name: self.name().to_string(),
                message: format!("Failed to compile regex pattern '{pattern}': {e}"),
            })?;

            if !regex.is_match(value) {
                return Err(Box::new(ValidationError {
                    rule_name: self.name().to_string(),
                    message: format!("Argument '{arg}' value '{value}' does not match pattern '{pattern}'"),
                }));
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

    fn clone_box(&self) -> Box<dyn ValidationRule> {
        Box::new(self.clone())
    }
}

/// Rule that validates required environment variables
#[derive(Debug, Clone)]
pub struct EnvironmentRule {
    /// List of environment variable names that must be present
    required_vars: Vec<String>,
}

#[allow(dead_code)]
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
    fn validate(&self, _command: &dyn Command, context: &ValidationContext) -> Result<(), Box<dyn Error + Send + Sync>> {
        for var in &self.required_vars {
            if !context.environment.contains_key(var) {
                return Err(Box::new(ValidationError {
                    rule_name: self.name().to_string(),
                    message: format!("Required environment variable '{var}' is missing"),
                }));
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

    fn clone_box(&self) -> Box<dyn ValidationRule> {
        Box::new(self.clone())
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
    #[allow(dead_code)]
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
    fn validate_name(name: &str, min_length: usize, max_length: usize) -> Result<(), Box<dyn Error + Send + Sync>> {
        let name_len = name.len();
        if name_len < min_length {
            return Err(Box::new(ValidationError {
                rule_name: "NameLength".to_string(),
                message: format!("Command name must be at least {min_length} characters long"),
            }));
        }
        if name_len > max_length {
            return Err(Box::new(ValidationError {
                rule_name: "NameLength".to_string(),
                message: format!("Command name cannot be longer than {max_length} characters"),
            }));
        }
        Ok(())
    }
}

impl ValidationRule for NameLengthRule {
    fn validate(&self, command: &dyn Command, _context: &ValidationContext) -> Result<(), Box<dyn Error + Send + Sync>> {
        Self::validate_name(command.name(), self.min_length, self.max_length)
    }

    fn name(&self) -> &'static str {
        "name_length"
    }

    fn description(&self) -> &'static str {
        "Validates command name length"
    }
    
    fn clone_box(&self) -> Box<dyn ValidationRule> {
        Box::new(self.clone())
    }
}

/// Rule that validates command description length
#[derive(Debug, Clone)]
pub struct DescriptionRule {
    /// Minimum required length for command description
    min_length: usize,
}

#[allow(dead_code)]
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
    fn validate(&self, command: &dyn Command, _context: &ValidationContext) -> Result<(), Box<dyn Error + Send + Sync>> {
        let description = command.description();
        if description.len() < self.min_length {
            return Err(Box::new(ValidationError {
                rule_name: self.name().to_string(),
                message: format!(
                    "Command description is too short (minimum length: {})",
                    self.min_length
                ),
            }));
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "description"
    }

    fn description(&self) -> &'static str {
        "Validates command description length"
    }
    
    fn clone_box(&self) -> Box<dyn ValidationRule> {
        Box::new(self.clone())
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

#[allow(dead_code)]
impl InputSanitizationRule {
    /// Creates a new `InputSanitizationRule` with the specified patterns and length limit
    /// 
    /// # Arguments
    /// * `patterns` - Map of input names to regex patterns they must match
    /// * `max_length` - Maximum allowed input length
    /// 
    /// # Errors
    /// Returns an error if any input doesn't match its pattern or exceeds the length limit
    pub fn new(patterns: HashMap<String, &str>, max_length: usize) -> Result<Self, Box<dyn Error>> {
        let mut compiled_patterns = HashMap::new();
        for (key, pattern) in patterns {
            let regex = regex::Regex::new(pattern).map_err(|e| {
                Box::new(ValidationError {
                    rule_name: "InputSanitization".to_string(),
                    message: format!("Invalid pattern for key '{key}': {e}"),
                }) as Box<dyn Error>
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
    fn validate(&self, _command: &dyn Command, context: &ValidationContext) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Check max length for all arguments
        for (key, value) in &context.arguments {
            if value.len() > self.max_length {
                return Err(Box::new(ValidationError {
                    rule_name: self.name().to_string(),
                    message: format!(
                        "Argument '{}' value exceeds maximum length of {} characters",
                        key,
                        self.max_length
                    ),
                }));
            }
        }

        // Check patterns
        for (key, pattern) in &self.patterns {
            if let Some(value) = context.arguments.get(key) {
                if !pattern.is_match(value) {
                    return Err(Box::new(ValidationError {
                        rule_name: self.name().to_string(),
                        message: format!("Argument '{key}' value '{value}' does not match required pattern"),
                    }));
                }
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
    
    fn clone_box(&self) -> Box<dyn ValidationRule> {
        Box::new(self.clone())
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

#[allow(dead_code)]
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

    /// Checks if the current memory usage is within the specified limits
    /// 
    /// # Returns
    /// * `Ok(())` if memory usage is within limits
    /// * `Err(Box<dyn Error>)` if memory usage exceeds limits
    fn check_memory_usage(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut sys = System::new_all();
        sys.refresh_all();
        let used_memory = sys.used_memory();
        let memory_usage_mb = (used_memory / 1024 / 1024) as usize;

        if memory_usage_mb > self.max_memory_mb {
            return Err(Box::new(ValidationError {
                rule_name: self.name().to_string(),
                message: format!(
                    "Memory usage ({}) exceeds maximum allowed ({} MB)",
                    memory_usage_mb, self.max_memory_mb
                ),
            }));
        }

        Ok(())
    }

    /// Checks if the current thread usage is within the specified limits
    /// 
    /// # Returns
    /// * `Ok(())` if thread usage is within limits
    /// * `Err(Box<dyn Error>)` if thread usage exceeds limits
    fn check_thread_usage(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut sys = System::new_all();
        sys.refresh_all();
        let thread_count = sys.cpus().len();

        if thread_count > self.max_threads {
            return Err(Box::new(ValidationError {
                rule_name: self.name().to_string(),
                message: format!(
                    "Thread count ({}) exceeds maximum allowed ({})",
                    thread_count, self.max_threads
                ),
            }));
        }

        Ok(())
    }
}

impl ValidationRule for ResourceValidationRule {
    fn validate(&self, _command: &dyn Command, _context: &ValidationContext) -> Result<(), Box<dyn Error + Send + Sync>> {
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
    
    fn clone_box(&self) -> Box<dyn ValidationRule> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            clap::Command::new("test")
                .about("A test command")
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
            clap::Command::new("t")
                .about("A test command")
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
            clap::Command::new("verylongname")
                .about("A test command")
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
            clap::Command::new("test")
                .about("test")
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
        context.set("test_key", "test_value").unwrap();
        assert_eq!(
            context.get("test_key").unwrap().unwrap(),
            "test_value"
        );
    }

    #[test]
    fn test_input_sanitization() {
        let mut patterns = HashMap::new();
        patterns.insert("email".to_string(), r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$");
        let rule = InputSanitizationRule::new(patterns, 100).unwrap();
        let mut context = ValidationContext::new();
        context.arguments.insert("email".to_string(), "test@example.com".to_string());
        let command = TestCommand;
        assert!(rule.validate(&command, &context).is_ok());
    }

    #[test]
    fn test_resource_validation_rule() {
        let mut sys = System::new_all();
        sys.refresh_all();
        let current_memory_mb = (sys.used_memory() / 1024 / 1024) as usize;
        let current_threads = sys.cpus().len();

        // Set limits higher than current usage
        let rule = ResourceValidationRule::new(current_memory_mb + 1024, current_threads + 10);
        let command = TestCommand;
        assert!(rule.validate(&command, &ValidationContext::new()).is_ok());
    }

    #[test]
    fn test_validator_rules() {
        let validator = CommandValidator::new();
        let rule = NameLengthRule::new(3, 10);
        validator.add_rule(Box::new(rule)).unwrap();
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
                    .unwrap();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        for i in 0..10 {
            assert_eq!(
                context.get(&format!("key{i}")).unwrap().map(|s| s.to_string()),
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
        context.arguments.insert("arg1".to_string(), "value1".to_string());
        context.arguments.insert("arg2".to_string(), "value2".to_string());
        assert!(rule.validate(&command, &context).is_ok());
    }

    #[test]
    fn test_argument_pattern_rule() {
        let mut patterns = HashMap::new();
        patterns.insert("email".to_string(), r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$".to_string());
        let rule = ArgumentPatternRule::new(patterns);
        let mut context = ValidationContext::new();
        let command = TestCommand;

        // Test invalid email
        context.arguments.insert("email".to_string(), "invalid-email".to_string());
        assert!(rule.validate(&command, &context).is_err());

        // Test valid email
        context.arguments.insert("email".to_string(), "test@example.com".to_string());
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
        context.environment.insert("PATH".to_string(), "/usr/bin".to_string());
        context.environment.insert("HOME".to_string(), "/home/user".to_string());
        assert!(rule.validate(&command, &context).is_ok());
    }

    #[test]
    fn test_input_sanitization_edge_cases() {
        let patterns = HashMap::new();
        let rule = InputSanitizationRule::new(patterns, 0).unwrap();
        let mut context = ValidationContext::new();
        context.arguments.insert("test".to_string(), "value".to_string());
        let command = TestCommand;
        assert!(rule.validate(&command, &context).is_err());
    }
}