# Commands Module

## Overview

The Commands module provides a robust system for registering, managing, and executing commands within the Squirrel application. This module implements dependency injection (DI) patterns to ensure testability, maintainability, and proper initialization control.

## Core Components

- `CommandRegistry`: Manages the collection of available commands
- `CommandRegistryFactory`: Creates customized command registries
- `CommandRegistryAdapter`: Provides a consistent interface for registry operations
- `Command`: Trait that defines the interface for all commands
- `ValidationRule`: Trait for command validation rules
- `LifecycleHook`: Trait for command lifecycle handlers

## Dependency Injection Patterns

The Commands module implements proper DI patterns through factories and adapters:

### Using Factory Pattern

The `CommandRegistryFactory` creates customized instances of `CommandRegistry`:

```rust
// Create a factory
let factory = CommandRegistryFactory::new();

// Create a registry using the factory
let registry = factory.create()?;

// Use the registry
registry.register(Box::new(MyCommand))?;
```

### Using Factory with Customization

You can customize the factory with validation rules and lifecycle handlers:

```rust
// Create a factory with custom validation and lifecycle hooks
let factory = CommandRegistryFactory::new()
    .with_validation_rule(Box::new(MyValidationRule))
    .with_lifecycle_handler(Box::new(MyLifecycleHandler));

// Create a registry with built-in commands
let registry = factory.create_with_builtins()?;
```

### Using Helper Functions

For simpler use cases, helper functions are available:

```rust
// Create a basic command registry
let registry = create_command_registry()?;

// Create a registry with built-in commands
let registry_with_builtins = create_command_registry_with_builtins()?;
```

### Using Adapter Pattern

The `CommandRegistryAdapter` provides a clean interface to the registry:

```rust
// Create an adapter
let adapter = CommandRegistryAdapter::new();

// Register a command
adapter.register_command(Box::new(MyCommand))?;

// Execute a command
adapter.execute_command("my-command", vec!["--option".to_string(), "value".to_string()])?;
```

## Command Implementation

To create a custom command, implement the `Command` trait:

```rust
struct MyCommand;

impl Command for MyCommand {
    fn name(&self) -> &'static str {
        "my-command"
    }
    
    fn description(&self) -> &'static str {
        "A custom command"
    }
    
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        println!("Executing my command");
        Ok(())
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("my-command")
            .about("A custom command")
            .arg(clap::Arg::new("option")
                .short('o')
                .long("option")
                .help("An option for the command")
                .required(false))
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(Self)
    }
}
```

## Error Handling

The Commands module provides clear error handling through the `CommandError` enum:

```rust
// Register a command with proper error handling
match registry.register(Box::new(MyCommand)) {
    Ok(_) => println!("Command registered successfully"),
    Err(CommandError::NotFound(name)) => {
        println!("Command '{}' not found", name);
    },
    Err(e) => println!("Error: {}", e),
}

// Execute a command with proper error handling
match registry.execute("my-command", args) {
    Ok(_) => println!("Command executed successfully"),
    Err(CommandError::Validation(e)) => {
        println!("Validation error: {}", e);
    },
    Err(CommandError::Execution(e)) => {
        println!("Execution error: {}", e);
    },
    Err(e) => println!("Error: {}", e),
}
```

## Command Validation and Lifecycle

The Commands module supports extensible validation rules and lifecycle hooks:

```rust
// Create a custom validation rule
struct MyValidationRule;

impl ValidationRule for MyValidationRule {
    fn name(&self) -> &'static str {
        "my-validation-rule"
    }
    
    fn description(&self) -> &'static str {
        "A custom validation rule"
    }
    
    fn validate(&self, cmd: &dyn Command, context: &ValidationContext) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Implement validation logic
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn ValidationRule> {
        Box::new(Self)
    }
}

// Create a custom lifecycle hook
struct MyLifecycleHandler;

impl LifecycleHook for MyLifecycleHandler {
    fn name(&self) -> &'static str {
        "my-lifecycle-hook"
    }
    
    fn stages(&self) -> Vec<LifecycleStage> {
        vec![LifecycleStage::BeforeExecution, LifecycleStage::AfterExecution]
    }
    
    fn on_stage(&self, stage: &LifecycleStage, command: &dyn Command) -> Result<(), Box<dyn Error>> {
        match stage {
            LifecycleStage::BeforeExecution => {
                println!("Before executing {}", command.name());
            }
            LifecycleStage::AfterExecution => {
                println!("After executing {}", command.name());
            }
            _ => {}
        }
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn LifecycleHook> {
        Box::new(Self)
    }
}
```

## Migration from Global State

### Before (using global state or implicit initialization)

```rust
// Old approach using global state
// This might use a global registry or create one implicitly
let command = get_command("some-command");
command.execute()?;
```

### After (using explicit DI)

```rust
// Approach 1: Using factory
let registry = CommandRegistryFactory::new().create()?;
let command = registry.get("some-command")?;
command.execute()?;

// Approach 2: Using helper function
let registry = create_command_registry()?;
registry.execute("some-command", args)?;

// Approach 3: Using adapter
let adapter = CommandRegistryAdapter::new();
adapter.execute_command("some-command", args)?;
```

## Testing

The module is designed to be easily testable:

```rust
#[test]
fn test_command_registry() {
    // Create a registry for testing
    let registry = CommandRegistryFactory::new()
        .create()
        .expect("Failed to create registry");
    
    // Register a test command
    let test_command = Box::new(TestCommand);
    registry.register(test_command.clone())
        .expect("Failed to register command");
    
    // Verify command registration
    let commands = registry.list().expect("Failed to list commands");
    assert!(commands.contains(&"test-command".to_string()));
    
    // Execute the command
    registry.execute("test-command", vec![])
        .expect("Failed to execute command");
}

// Test Command implementation
struct TestCommand;

impl Command for TestCommand {
    fn name(&self) -> &'static str { "test-command" }
    fn description(&self) -> &'static str { "Test command" }
    fn execute(&self) -> Result<(), Box<dyn Error>> { Ok(()) }
    fn parser(&self) -> clap::Command { clap::Command::new("test-command") }
    fn clone_box(&self) -> Box<dyn Command> { Box::new(Self) }
}
``` 