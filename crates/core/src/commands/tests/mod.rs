use std::error::Error;
use std::sync::Arc;

use clap::{Parser, CommandFactory};
use tokio::test;

use crate::commands::{
    Command,
    CommandRegistry,
    CommandRegistryFactory,
    validation::{ValidationRule, ValidationContext},
    lifecycle::{LifecycleHook, LifecycleStage},
};
use crate::test_utils::{TestError, TestFactory};

// Test implementations

#[derive(Parser)]
#[command(name = "test")]
#[allow(dead_code)]
struct TestArgs {
    #[arg(short, long)]
    value: String,
}

#[derive(Clone)]
#[allow(dead_code)]
struct TestCommand;

impl Command for TestCommand {
    fn name(&self) -> &'static str {
        "test"
    }
    
    fn description(&self) -> &'static str {
        "A test command for registry"
    }

    fn execute(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn parser(&self) -> clap::Command {
        TestArgs::command()
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
struct TestValidationRule;

impl ValidationRule for TestValidationRule {
    fn name(&self) -> &'static str {
        "test-validation"
    }
    
    fn description(&self) -> &'static str {
        "A test validation rule"
    }
    
    fn validate(&self, _cmd: &dyn Command, _context: &ValidationContext) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn ValidationRule> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
struct TestLifecycleHandler;

impl LifecycleHook for TestLifecycleHandler {
    fn name(&self) -> &'static str {
        "test-lifecycle"
    }
    
    fn stages(&self) -> Vec<LifecycleStage> {
        vec![LifecycleStage::PreExecution]
    }
    
    fn on_stage(&self, _stage: &LifecycleStage, _command: &dyn Command) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn LifecycleHook> {
        Box::new(self.clone())
    }
}

// Helper functions for test setup

/// Creates a test command registry with DI pattern
fn create_test_registry() -> CommandRegistry {
    // ARRANGE: Create a command registry
    CommandRegistry::new()
}

/// Creates a test command for testing
fn create_test_command() -> Box<dyn Command> {
    // ARRANGE: Create a test command
    Box::new(TestCommand)
}

/// Creates a test validation rule
fn create_test_validation_rule() -> Box<dyn ValidationRule> {
    // ARRANGE: Create a test validation rule
    Box::new(TestValidationRule)
}

/// Creates a test lifecycle hook
fn create_test_lifecycle_hook() -> Box<dyn LifecycleHook> {
    // ARRANGE: Create a test lifecycle hook
    Box::new(TestLifecycleHandler)
}

// Tests with AAA pattern

#[test]
fn test_command_registration() {
    // ARRANGE: Create test registry and command
    let mut registry = create_test_registry();
    let command = create_test_command();
    
    // ACT: Register the command
    let result = registry.register_command(command);
    
    // ASSERT: Verify command was registered
    assert!(result.is_ok(), "Command registration should succeed");
    
    // Verify command is in the registry
    let commands = registry.list_commands();
    assert_eq!(commands.len(), 1, "Registry should have one command");
    assert_eq!(commands[0].name(), "test", "Command should have correct name");
}

#[test]
fn test_command_execution() {
    // ARRANGE: Create test registry and command
    let mut registry = create_test_registry();
    let command = create_test_command();
    
    // Register the command
    registry.register_command(command).unwrap();
    
    // ACT: Execute the command
    let result = registry.execute_command("test", &[]);
    
    // ASSERT: Verify command execution
    assert!(result.is_ok(), "Command execution should succeed");
}

#[test]
fn test_command_listing() {
    // ARRANGE: Create test registry with multiple commands
    let mut registry = create_test_registry();
    
    // Register several test commands
    registry.register_command(create_test_command()).unwrap();
    
    // Create and register a second test command
    #[derive(Clone)]
    struct AnotherCommand;
    impl Command for AnotherCommand {
        fn name(&self) -> &'static str { "another-test" }
        fn description(&self) -> &'static str { "Another test command" }
        fn execute(&self) -> Result<(), Box<dyn Error>> { Ok(()) }
        fn parser(&self) -> clap::Command { TestArgs::command() }
        fn clone_box(&self) -> Box<dyn Command> { Box::new(self.clone()) }
    }
    
    registry.register_command(Box::new(AnotherCommand)).unwrap();
    
    // ACT: List all commands
    let commands = registry.list_commands();
    
    // ASSERT: Verify commands are listed correctly
    assert_eq!(commands.len(), 2, "Registry should have two commands");
    
    // Check command names
    let names: Vec<&str> = commands.iter().map(|c| c.name()).collect();
    assert!(names.contains(&"test"), "Registry should contain 'test' command");
    assert!(names.contains(&"another-test"), "Registry should contain 'another-test' command");
}

#[test]
fn test_command_registry_factory_create() {
    // ARRANGE: Create factory
    let factory = CommandRegistryFactory::new();
    
    // ACT: Create registry with factory
    let registry = factory.create();
    
    // ASSERT: Verify registry is created
    assert_eq!(registry.list_commands().len(), 0, "Registry should start empty");
}

#[test]
fn test_command_registry_factory_with_validation() {
    // ARRANGE: Create factory with validation rule
    let mut factory = CommandRegistryFactory::new();
    factory.add_validation_rule(create_test_validation_rule());
    
    // ACT: Create registry with factory
    let registry = factory.create();
    
    // ASSERT: Verify registry has validation rule
    assert_eq!(registry.list_validation_rules().len(), 1, "Registry should have one validation rule");
    assert_eq!(registry.list_validation_rules()[0].name(), "test-validation", "Validation rule should have correct name");
}

#[test]
fn test_command_registry_factory_with_lifecycle() {
    // ARRANGE: Create factory with lifecycle hook
    let mut factory = CommandRegistryFactory::new();
    factory.add_lifecycle_hook(create_test_lifecycle_hook());
    
    // ACT: Create registry with factory
    let registry = factory.create();
    
    // ASSERT: Verify registry has lifecycle hook
    assert_eq!(registry.list_lifecycle_hooks().len(), 1, "Registry should have one lifecycle hook");
    assert_eq!(registry.list_lifecycle_hooks()[0].name(), "test-lifecycle", "Lifecycle hook should have correct name");
}

#[test]
fn test_command_registry_factory_with_builtins() {
    // ARRANGE: Create factory with builtins
    let mut factory = CommandRegistryFactory::new();
    factory.with_builtins(true);
    
    // ACT: Create registry with factory
    let registry = factory.create();
    
    // ASSERT: Verify registry has builtin commands
    assert!(registry.list_commands().len() > 0, "Registry should have builtin commands");
    
    // Check for help command which should be part of builtins
    let commands = registry.list_commands();
    let has_help = commands.iter().any(|c| c.name() == "help");
    assert!(has_help, "Registry should have 'help' command");
} 