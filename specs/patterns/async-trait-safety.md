---
title: Async Trait Safety Pattern
author: DataScienceBioLab
date: 2024-06-26
status: Active
category: Design Pattern
---

# Async Trait Safety Pattern

## Context

In Rust, traits with async methods pose a challenge when you need to use them as trait objects (with `dyn` keyword). This is because async functions in Rust are transformed into methods returning `impl Future`, which is not object-safe due to the size of the return type being unknown at compile time.

This pattern provides a solution for maintaining trait object safety while still enabling async functionality through proper separation of concerns.

## Problem Statement

Consider the following trait with an async method:

```rust
#[async_trait]
pub trait Command {
    async fn execute(&self, args: Vec<String>) -> Result<String, Error>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}
```

Using this trait as a trait object will fail with an error similar to:

```
error[E0038]: the trait `Command` cannot be made into an object
 --> src/main.rs:10:13
  |
3 | pub trait Command {
  |           ------- this trait cannot be made into an object...
4 |     async fn execute(&self, args: Vec<String>) -> Result<String, Error>;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ...because method `execute` has an incompatible type
  |
  = note: for a trait to be "object safe" it needs to allow building a vtable to allow the call to be resolvable dynamically
  = note: async fn isn't object-safe because it returns an unboxed `impl Future`
```

## Solution

### 1. Split the Trait

Separate the trait into two parts:
1. A base trait containing all non-async methods (object-safe)
2. An extended trait containing async methods (not used with `dyn`)

```rust
// Base trait with non-async methods (object-safe)
pub trait CommandBase: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

// Trait with async methods (not used with dyn)
#[async_trait]
pub trait AsyncCommand: CommandBase {
    async fn execute(&self, args: Vec<String>) -> Result<String, Error>;
}
```

### 2. Use Type Erasure or Static Dispatch

#### Option A: Type Erasure

Create a wrapper struct that uses generics to erase the concrete type:

```rust
struct TypeErasedCommand<T: AsyncCommand> {
    command: T
}

impl<T: AsyncCommand> TypeErasedCommand<T> {
    fn new(command: T) -> Self {
        Self { command }
    }
    
    async fn execute(&self, args: Vec<String>) -> Result<String, Error> {
        self.command.execute(args).await
    }
    
    fn name(&self) -> &str {
        self.command.name()
    }
    
    fn description(&self) -> &str {
        self.command.description()
    }
}
```

#### Option B: Static Dispatch with Concrete Types

If you're storing the commands in a collection, use concrete types instead of trait objects:

```rust
// Instead of
struct Registry {
    commands: HashMap<String, Arc<dyn Command>>,  // Won't work!
}

// Do this
struct Registry {
    commands: HashMap<String, Arc<MyConcreteCommand>>,
}

// Or for multiple types, use an enum
enum CommandEnum {
    Type1(Arc<Type1Command>),
    Type2(Arc<Type2Command>),
    Type3(Arc<Type3Command>),
}

struct Registry {
    commands: HashMap<String, CommandEnum>,
}
```

### 3. Dynamic Dispatch with Boxed Futures

If you absolutely need dynamic dispatch, use `Box<dyn Future>` for the return type:

```rust
pub trait DynCommand: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    
    // Return a boxed future instead of using async
    fn execute(&self, args: Vec<String>) -> Box<dyn Future<Output = Result<String, Error>> + Send + Sync>;
}
```

This approach is less elegant but allows for trait objects.

## Implementation Example

### Defining the Traits

```rust
use async_trait::async_trait;
use std::sync::Arc;

// Base trait - object safe
pub trait CommandBase: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

// Async trait - not used with dyn
#[async_trait]
pub trait AsyncCommand: CommandBase {
    async fn execute(&self, args: Vec<String>) -> Result<String, String>;
}

// Concrete command implementation
struct EchoCommand {
    name: String,
    description: String,
}

impl EchoCommand {
    fn new(name: String, description: String) -> Self {
        Self { name, description }
    }
}

impl CommandBase for EchoCommand {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
}

#[async_trait]
impl AsyncCommand for EchoCommand {
    async fn execute(&self, args: Vec<String>) -> Result<String, String> {
        Ok(args.join(" "))
    }
}
```

### Registry Implementation

```rust
use std::collections::HashMap;

// Registry using specific types
struct CommandRegistry {
    commands: HashMap<String, Arc<EchoCommand>>,
}

impl CommandRegistry {
    fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }
    
    fn register(&mut self, command: Arc<EchoCommand>) -> Result<(), String> {
        let name = command.name().to_string();
        self.commands.insert(name, command);
        Ok(())
    }
    
    fn get_command(&self, name: &str) -> Option<&Arc<EchoCommand>> {
        self.commands.get(name)
    }
    
    async fn execute_command(&self, name: &str, args: Vec<String>) -> Result<String, String> {
        match self.get_command(name) {
            Some(command) => command.execute(args).await,
            None => Err(format!("Command not found: {}", name)),
        }
    }
}
```

## Considerations

### Advantages
- Maintains type safety while enabling async methods
- Clear separation between sync and async functionality
- Can use concrete types for better performance
- More maintainable and easier to understand

### Disadvantages
- More complex implementation than a single trait
- Type erasure adds some boilerplate
- Using concrete types limits flexibility
- Boxed futures can have performance implications

## When to Use

Use this pattern when:
- You need traits with async methods
- You're storing or passing around collections of objects implementing these traits
- You need to maintain object safety in your trait hierarchy
- You're designing a plugin or extensibility system

## Alternatives

- **Boxed Trait Objects**: Use `Box<dyn Future>` return types explicitly
- **Associated Types**: Use associated types instead of direct methods
- **Callback Style**: Use closure-based callbacks instead of async methods

## References

- [Rust Book: Object Safety](https://doc.rust-lang.org/book/ch17-02-trait-objects.html#object-safety-is-required-for-trait-objects)
- [async-trait crate](https://docs.rs/async-trait/latest/async_trait/)
- [Rust RFC: Object Safety](https://rust-lang.github.io/rfcs/0255-object-safety.html)
- [Solving the Async Trait Bounds Problem](https://smallcultfollowing.com/babysteps/blog/2019/10/26/async-fn-in-traits-are-hard/) 