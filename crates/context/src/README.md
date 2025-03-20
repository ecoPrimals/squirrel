# Context Module

## Overview

The Context module provides functionality for managing application state, persistence, and synchronization. This module implements dependency injection (DI) patterns to ensure testability, maintainability, and proper initialization control.

## Core Components

- `ContextManager`: Manages multiple contexts and their lifecycle
- `ContextTracker`: Tracks the active context and provides context switching
- `ContextTrackerFactory`: Creates context trackers with configurable options
- `Context`: Represents a single application context with state
- `ContextConfig`: Configuration for context behavior

## Dependency Injection Patterns

The Context module implements proper DI patterns through factories and explicit initialization:

### Using Factory Pattern

The `ContextTrackerFactory` creates instances of `ContextTracker` with proper initialization:

```rust
// Create a factory with a default context manager
let manager = Arc::new(ContextManager::new());
let factory = ContextTrackerFactory::new(Some(manager));

// Create a context tracker
let tracker = factory.create()?;
```

### Using Factory with Configuration

You can customize the factory with configuration parameters:

```rust
// Create a context config
let config = ContextConfig {
    id: "main-context".to_string(),
    name: "Main Application Context".to_string(),
    description: "Primary application context".to_string(),
    environment: "development".to_string(),
    version: "1.0".to_string(),
    metadata: HashMap::new(),
    persistence: true,
    max_entries: 100,
};

// Create a factory with config
let factory = ContextTrackerFactory::with_config(Some(manager), config);

// Create a context tracker with config
let tracker = factory.create_with_config(config)?;
```

### Using Helper Functions

For simpler use cases, helper functions are available:

```rust
// Create a context tracker with default settings
let tracker = create_context_tracker()?;

// Create a context tracker with custom configuration
let custom_tracker = create_context_tracker_with_config(config)?;
```

## Working with Context

Once you have a `ContextTracker`, you can manage contexts:

```rust
// Create a context in the manager
let context_id = "my-context";
let context = tracker.manager.create_context(context_id.to_string()).await?;

// Activate a context
tracker.activate_context(context_id).await?;

// Get the active context
if let Some(context) = tracker.get_active_context().await? {
    // Work with the context
    context.update_data("key", json!("value")).await?;
}

// Deactivate the context
tracker.deactivate_context().await?;
```

## Error Handling

The Context module provides clear error handling:

```rust
// Check if context exists before activating
match tracker.activate_context("unknown-id").await {
    Ok(_) => println!("Context activated"),
    Err(e) => {
        if let Some(ContextError::ContextNotFound(_)) = e.downcast_ref::<ContextError>() {
            println!("Context not found, creating it now");
            tracker.manager.create_context("unknown-id".to_string()).await?;
        } else {
            println!("Unexpected error: {}", e);
        }
    }
}

// Handle result from factory pattern
match factory.create() {
    Ok(tracker) => println!("Tracker created successfully"),
    Err(e) => {
        if let Some(ContextError::NotInitialized) = e.downcast_ref::<ContextError>() {
            println!("Factory not initialized with a manager");
        } else {
            println!("Error creating tracker: {}", e);
        }
    }
}
```

## Context State Management

Contexts maintain state that can be accessed and updated:

```rust
// Get current state
let state = context.get_state().await?;

// Update specific data
context.update_data("user", json!({ "name": "Alice", "role": "Admin" })).await?;

// Replace entire state
context.set_state(HashMap::from([
    ("user".to_string(), json!({ "name": "Bob" })),
    ("settings".to_string(), json!({ "theme": "dark" }))
])).await?;

// Get a snapshot
let snapshot = context.get_snapshot().await?;
```

## Migration from Global State

### Before (using global state or implicit initialization)

```rust
// Old approach using global state
let tracker = ContextTracker::new(); // No manager provided, might use global state
tracker.activate_context("default").unwrap(); // Might create context on-demand
```

### After (using explicit DI)

```rust
// Approach 1: Explicit initialization
let manager = Arc::new(ContextManager::new());
let tracker = ContextTracker::new(manager);
tracker.activate_context("default").await?; // Will error if context doesn't exist

// Approach 2: Using factory
let factory = ContextTrackerFactory::new(Some(Arc::new(ContextManager::new())));
let tracker = factory.create()?;

// Approach 3: Using helper function
let tracker = create_context_tracker()?;
```

## Testing

The module is designed to be easily testable:

```rust
#[tokio::test]
async fn test_context_tracker() {
    // Create a context tracker for testing
    let tracker = create_context_tracker().unwrap();
    
    // Create a test context
    let context_id = "test-context";
    tracker.manager.create_context(context_id.to_string()).await.unwrap();
    
    // Activate the context
    tracker.activate_context(context_id).await.unwrap();
    
    // Get the context
    let active = tracker.get_active_context().await.unwrap();
    assert!(active.is_some());
    
    // Verify context ID
    let context = active.unwrap();
    let state = context.get_state().await.unwrap();
    assert_eq!(state.id, context_id);
}
``` 