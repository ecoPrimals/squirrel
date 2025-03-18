use std::error::Error;
use std::sync::Arc;

use clap::{Parser, CommandFactory};

use crate::commands::{
    Command,
    CommandRegistry,
    CommandRegistryFactory,
    validation::{ValidationRule, ValidationContext},
    lifecycle::{LifecycleHook, LifecycleStage},
};
use crate::test_utils::{TestError, TestFactory};

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
        "TestValidationRule"
    }
    
    fn description(&self) -> &'static str {
        "A test validation rule for unit tests"
    }
    
    fn validate(&self, _cmd: &dyn Command, _context: &ValidationContext) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn ValidationRule> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
struct TestLifecycleHandler;

impl LifecycleHook for TestLifecycleHandler {
    fn name(&self) -> &'static str {
        "TestLifecycleHandler"
    }
    
    fn stages(&self) -> Vec<LifecycleStage> {
        vec![LifecycleStage::Validation]
    }
    
    fn on_stage(&self, _stage: &LifecycleStage, _command: &dyn Command) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn LifecycleHook> {
        Box::new(self.clone())
    }
}

#[test]
fn test_command_registration() {
    let mut registry = CommandRegistry::new();
    assert!(registry.register(Box::new(TestCommand)).is_ok());
    assert!(registry.get("test").unwrap().is_some());
}

#[test]
fn test_command_execution() {
    let mut registry = CommandRegistry::new();
    registry.register(Box::new(TestCommand)).unwrap();
    
    let args = vec!["test".to_string(), "--value".to_string(), "test".to_string()];
    assert!(registry.execute("test", args).is_ok());
}

#[test]
fn test_command_listing() {
    let mut registry = CommandRegistry::new();
    registry.register(Box::new(TestCommand)).unwrap();
    
    let commands = registry.list().unwrap();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0], "test");
}

#[test]
fn test_command_registry_factory_create() {
    let factory = CommandRegistryFactory::new();
    let registry = factory.create().unwrap();
    
    // Registry should be empty
    assert!(registry.list().unwrap().is_empty());
}

#[test]
fn test_command_registry_factory_with_validation() {
    let factory = CommandRegistryFactory::new()
        .with_validation_rule(Box::new(TestValidationRule));
    
    // Just assert that we have a factory
    assert!(factory.validation_rules.len() > 0);
}

#[test]
fn test_command_registry_factory_with_lifecycle() {
    let factory = CommandRegistryFactory::new()
        .with_lifecycle_handler(Box::new(TestLifecycleHandler));
    let registry = factory.create().unwrap();
    
    // Just assert we can create a registry with the factory
    assert!(registry.list().unwrap().is_empty());
}

#[test]
fn test_command_registry_factory_with_builtins() {
    let registry = CommandRegistryFactory::new().create_with_builtins().unwrap();
    
    // Registry should have builtin commands
    assert!(!registry.list().unwrap().is_empty());
    // Should include the version command
    assert!(registry.get("version").unwrap().is_some());
}

#[test]
fn test_create_command_registry() {
    let registry = crate::commands::create_command_registry().unwrap();
    assert!(registry.list().unwrap().is_empty());
}

#[test]
fn test_create_command_registry_with_builtins() {
    let registry = crate::commands::create_command_registry_with_builtins().unwrap();
    assert!(!registry.list().unwrap().is_empty());
} 