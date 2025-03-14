use std::error::Error;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use crate::core::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationContext {
    pub command_name: String,
    pub arguments: HashMap<String, String>,
    pub environment: HashMap<String, String>,
    #[serde(skip)]
    data: RwLock<HashMap<String, String>>,
}

impl ValidationContext {
    pub fn new() -> Self {
        Self {
            command_name: String::new(),
            arguments: HashMap::new(),
            environment: HashMap::new(),
            data: RwLock::new(HashMap::new()),
        }
    }

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

pub trait ValidationRule: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn validate(&self, command: &dyn Command, context: &ValidationContext) -> Result<(), Box<dyn Error>>;
}

pub struct CommandValidator {
    rules: RwLock<Vec<Box<dyn ValidationRule>>>,
    context: ValidationContext,
}

impl CommandValidator {
    pub fn new() -> Self {
        Self {
            rules: RwLock::new(Vec::new()),
            context: ValidationContext::new(),
        }
    }

    pub fn add_rule(&self, rule: Box<dyn ValidationRule>) -> Result<(), Box<dyn Error>> {
        let mut rules = self.rules.write().map_err(|_| {
            Box::new(ValidationError {
                rule_name: "validator".to_string(),
                message: "Failed to acquire write lock on rules".to_string(),
            }) as Box<dyn Error>
        })?;
        rules.push(rule);
        Ok(())
    }

    pub fn validate(&self, command: &dyn Command) -> Result<(), Box<dyn Error>> {
        let rules = self.rules.read().map_err(|_| {
            Box::new(ValidationError {
                rule_name: "validator".to_string(),
                message: "Failed to acquire read lock on rules".to_string(),
            }) as Box<dyn Error>
        })?;

        for rule in rules.iter() {
            rule.validate(command, &self.context)?;
        }

        Ok(())
    }

    pub fn context(&self) -> &ValidationContext {
        &self.context
    }
}

impl Default for CommandValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct ValidationError {
    pub rule_name: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Validation error in rule {}: {}", self.rule_name, self.message)
    }
}

impl Error for ValidationError {}

// Built-in validation rules

pub struct RequiredArgumentsRule {
    required_args: Vec<String>,
}

impl RequiredArgumentsRule {
    pub fn new(required_args: Vec<String>) -> Self {
        Self { required_args }
    }
}

impl ValidationRule for RequiredArgumentsRule {
    fn validate(&self, _command: &dyn Command, context: &ValidationContext) -> Result<(), Box<dyn Error>> {
        for arg in &self.required_args {
            if !context.arguments.contains_key(arg) {
                return Err(Box::new(ValidationError {
                    rule_name: self.name().to_string(),
                    message: format!("Required argument '{}' is missing", arg),
                }));
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "RequiredArguments"
    }

    fn description(&self) -> &str {
        "Validates that all required arguments are present"
    }
}

pub struct ArgumentPatternRule {
    patterns: HashMap<String, String>,
}

impl ArgumentPatternRule {
    pub fn new(patterns: HashMap<String, String>) -> Self {
        Self { patterns }
    }
}

impl ValidationRule for ArgumentPatternRule {
    fn validate(&self, _command: &dyn Command, context: &ValidationContext) -> Result<(), Box<dyn Error>> {
        for (arg, pattern) in &self.patterns {
            if let Some(value) = context.arguments.get(arg) {
                let regex = regex::Regex::new(pattern).map_err(|_| {
                    Box::new(ValidationError {
                        rule_name: self.name().to_string(),
                        message: format!("Invalid pattern for argument '{}': {}", arg, pattern),
                    }) as Box<dyn Error>
                })?;

                if !regex.is_match(value) {
                    return Err(Box::new(ValidationError {
                        rule_name: self.name().to_string(),
                        message: format!("Argument '{}' value '{}' does not match pattern '{}'", arg, value, pattern),
                    }));
                }
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "ArgumentPattern"
    }

    fn description(&self) -> &str {
        "Validates argument values against regular expression patterns"
    }
}

pub struct EnvironmentRule {
    required_vars: Vec<String>,
}

impl EnvironmentRule {
    pub fn new(required_vars: Vec<String>) -> Self {
        Self { required_vars }
    }
}

impl ValidationRule for EnvironmentRule {
    fn validate(&self, _command: &dyn Command, context: &ValidationContext) -> Result<(), Box<dyn Error>> {
        for var in &self.required_vars {
            if !context.environment.contains_key(var) {
                return Err(Box::new(ValidationError {
                    rule_name: self.name().to_string(),
                    message: format!("Required environment variable '{}' is missing", var),
                }));
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "Environment"
    }

    fn description(&self) -> &str {
        "Validates that required environment variables are present"
    }
}

pub struct NameLengthRule {
    min_length: usize,
    max_length: usize,
}

impl NameLengthRule {
    pub fn new(min_length: usize, max_length: usize) -> Self {
        Self {
            min_length,
            max_length,
        }
    }
}

impl ValidationRule for NameLengthRule {
    fn name(&self) -> &str {
        "NameLength"
    }

    fn description(&self) -> &str {
        "Validates command name length"
    }

    fn validate(&self, command: &dyn Command, _context: &ValidationContext) -> Result<(), Box<dyn Error>> {
        let name = command.name();
        if name.len() < self.min_length {
            return Err(Box::new(ValidationError {
                rule_name: self.name().to_string(),
                message: format!(
                    "Command name '{}' is too short (minimum length: {})",
                    name, self.min_length
                ),
            }));
        }
        if name.len() > self.max_length {
            return Err(Box::new(ValidationError {
                rule_name: self.name().to_string(),
                message: format!(
                    "Command name '{}' is too long (maximum length: {})",
                    name, self.max_length
                ),
            }));
        }
        Ok(())
    }
}

pub struct DescriptionRule {
    min_length: usize,
}

impl DescriptionRule {
    pub fn new(min_length: usize) -> Self {
        Self { min_length }
    }
}

impl ValidationRule for DescriptionRule {
    fn name(&self) -> &str {
        "Description"
    }

    fn description(&self) -> &str {
        "Validates command description length"
    }

    fn validate(&self, command: &dyn Command, _context: &ValidationContext) -> Result<(), Box<dyn Error>> {
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
}

pub struct InputSanitizationRule {
    patterns: HashMap<String, regex::Regex>,
    max_length: usize,
}

impl InputSanitizationRule {
    pub fn new(patterns: HashMap<String, &str>, max_length: usize) -> Result<Self, Box<dyn Error>> {
        let mut compiled_patterns = HashMap::new();
        for (key, pattern) in patterns {
            let regex = regex::Regex::new(pattern).map_err(|_| {
                Box::new(ValidationError {
                    rule_name: "InputSanitization".to_string(),
                    message: format!("Invalid pattern for key '{}': {}", key, pattern),
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
    fn name(&self) -> &str {
        "InputSanitization"
    }

    fn description(&self) -> &str {
        "Validates and sanitizes input values"
    }

    fn validate(&self, _command: &dyn Command, context: &ValidationContext) -> Result<(), Box<dyn Error>> {
        // Validate argument lengths
        for (key, value) in &context.arguments {
            if value.len() > self.max_length {
                return Err(Box::new(ValidationError {
                    rule_name: self.name().to_string(),
                    message: format!(
                        "Argument '{}' value exceeds maximum length of {}",
                        key, self.max_length
                    ),
                }));
            }
        }

        // Validate against patterns
        for (key, pattern) in &self.patterns {
            if let Some(value) = context.arguments.get(key) {
                if !pattern.is_match(value) {
                    return Err(Box::new(ValidationError {
                        rule_name: self.name().to_string(),
                        message: format!(
                            "Argument '{}' value '{}' does not match required pattern",
                            key, value
                        ),
                    }));
                }
            }
        }

        Ok(())
    }
}

pub struct ResourceValidationRule {
    max_memory_mb: usize,
    max_threads: usize,
}

impl ResourceValidationRule {
    pub fn new(max_memory_mb: usize, max_threads: usize) -> Self {
        Self {
            max_memory_mb,
            max_threads,
        }
    }

    fn check_memory_usage(&self) -> Result<(), Box<dyn Error>> {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        let total_memory = sys.total_memory();
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

    fn check_thread_usage(&self) -> Result<(), Box<dyn Error>> {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        let thread_count = sys.processes().values().map(|p| p.threads()).sum::<u32>() as usize;

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
    fn name(&self) -> &str {
        "ResourceValidation"
    }

    fn description(&self) -> &str {
        "Validates system resource usage"
    }

    fn validate(&self, _command: &dyn Command, _context: &ValidationContext) -> Result<(), Box<dyn Error>> {
        self.check_memory_usage()?;
        self.check_thread_usage()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    struct TestCommand {
        name: String,
        description: String,
    }

    impl crate::core::CommandOutput for TestCommand {
        fn execute_with_output(&self, _output: &mut dyn Write) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
    }

    impl Command for TestCommand {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn execute(&self) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
    }

    #[test]
    fn test_name_length_rule() {
        let rule = NameLengthRule::new(3, 10);
        let command = TestCommand {
            name: "test".to_string(),
            description: "Test command".to_string(),
        };
        assert!(rule.validate(&command, &ValidationContext::new()).is_ok());

        let command = TestCommand {
            name: "t".to_string(),
            description: "Test command".to_string(),
        };
        assert!(rule.validate(&command, &ValidationContext::new()).is_err());

        let command = TestCommand {
            name: "very_long_command_name".to_string(),
            description: "Test command".to_string(),
        };
        assert!(rule.validate(&command, &ValidationContext::new()).is_err());
    }

    #[test]
    fn test_description_rule() {
        let rule = DescriptionRule::new(5);
        let command = TestCommand {
            name: "test".to_string(),
            description: "Test command".to_string(),
        };
        assert!(rule.validate(&command, &ValidationContext::new()).is_ok());

        let command = TestCommand {
            name: "test".to_string(),
            description: "Test".to_string(),
        };
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
    fn test_input_sanitization_rule() {
        let mut patterns = HashMap::new();
        patterns.insert("email", r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$");
        let rule = InputSanitizationRule::new(patterns, 100).unwrap();

        let mut context = ValidationContext::new();
        context.arguments.insert("email".to_string(), "test@example.com".to_string());
        let command = TestCommand {
            name: "test".to_string(),
            description: "Test command".to_string(),
        };
        assert!(rule.validate(&command, &context).is_ok());

        context.arguments.insert("email".to_string(), "invalid-email".to_string());
        assert!(rule.validate(&command, &context).is_err());
    }

    #[test]
    fn test_resource_validation_rule() {
        let rule = ResourceValidationRule::new(1024, 100);
        let command = TestCommand {
            name: "test".to_string(),
            description: "Test command".to_string(),
        };
        assert!(rule.validate(&command, &ValidationContext::new()).is_ok());
    }

    #[test]
    fn test_command_validator() {
        let validator = CommandValidator::new();
        let rule = NameLengthRule::new(3, 10);
        validator.add_rule(Box::new(rule)).unwrap();

        let command = TestCommand {
            name: "test".to_string(),
            description: "Test command".to_string(),
        };
        assert!(validator.validate(&command).is_ok());

        let command = TestCommand {
            name: "t".to_string(),
            description: "Test command".to_string(),
        };
        assert!(validator.validate(&command).is_err());
    }

    #[test]
    fn test_validation_context_thread_safety() {
        use std::thread;

        let context = ValidationContext::new();
        let mut handles = vec![];

        for i in 0..10 {
            let context = &context;
            handles.push(thread::spawn(move || {
                context
                    .set(&format!("key{}", i), &format!("value{}", i))
                    .unwrap();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        for i in 0..10 {
            assert_eq!(
                context.get(&format!("key{}", i)).unwrap().unwrap(),
                format!("value{}", i)
            );
        }
    }

    #[test]
    fn test_required_arguments_rule() {
        let rule = RequiredArgumentsRule::new(vec!["arg1".to_string(), "arg2".to_string()]);
        let mut context = ValidationContext::new();
        let command = TestCommand {
            name: "test".to_string(),
            description: "Test command".to_string(),
        };

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
        let command = TestCommand {
            name: "test".to_string(),
            description: "Test command".to_string(),
        };

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
        let command = TestCommand {
            name: "test".to_string(),
            description: "Test command".to_string(),
        };

        // Test missing environment variables
        assert!(rule.validate(&command, &context).is_err());

        // Test with all required environment variables
        context.environment.insert("PATH".to_string(), "/usr/bin".to_string());
        context.environment.insert("HOME".to_string(), "/home/user".to_string());
        assert!(rule.validate(&command, &context).is_ok());
    }

    #[test]
    fn test_resource_validation_edge_cases() {
        let rule = ResourceValidationRule::new(0, 0);
        let command = TestCommand {
            name: "test".to_string(),
            description: "Test command".to_string(),
        };
        assert!(rule.validate(&command, &ValidationContext::new()).is_err());
    }

    #[test]
    fn test_input_sanitization_edge_cases() {
        let patterns = HashMap::new();
        let rule = InputSanitizationRule::new(patterns, 0).unwrap();
        let mut context = ValidationContext::new();
        context.arguments.insert("test".to_string(), "value".to_string());
        let command = TestCommand {
            name: "test".to_string(),
            description: "Test command".to_string(),
        };
        assert!(rule.validate(&command, &context).is_err());
    }
} 