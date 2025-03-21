# Context Management System

A robust context management system for tracking, persisting, and synchronizing application state.

## Overview

The Context Management System provides a comprehensive solution for managing application context. It handles state tracking, persistence, recovery, and synchronization across different components of an application.

## Key Components

### Context Manager

The Context Manager serves as the central component for managing contexts. It provides functionality for:

- Creating, updating, and deleting contexts
- Loading and saving state from/to persistence
- Creating and managing recovery points
- Listing available contexts

### Context Adapter

The Context Adapter provides integration support between the context system and external components. It handles:

- Context activation and deactivation
- Management of currently active contexts
- Status tracking of contexts (active, inactive, non-existent)
- Context switching

### Context Tracker

The Context Tracker is responsible for tracking context state changes. It provides:

- Thread-safe access to the current state
- State update functionality with version checks
- Automatic synchronization with persistence
- Recovery point creation

### State Management

The State structure manages the actual context data:

- Versioned state with timestamps
- Key-value based data storage
- Metadata support
- Snapshot creation for recovery points

### Context Factory

The factory pattern is implemented through the ContextTrackerFactory, which creates preconfigured Context Tracker instances.

## Usage

Here's a basic example of how to use the context system:

```rust
use context::{create_manager, create_adapter, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create a context manager
    let manager = create_manager();
    
    // Create a context adapter
    let adapter = create_adapter(manager.clone());
    
    // Initialize the adapter - this will also activate the default context
    adapter.initialize().await?;
    
    // Get the current context tracker
    let tracker = adapter.get_current_tracker()?;
    
    // Get the current state
    let mut state = tracker.get_state()?;
    
    // Update the state
    state.set("key".to_string(), "value".to_string());
    
    // Update the tracker with the new state
    tracker.update_state(state)?;
    
    // Create a new context and activate it
    let new_tracker = adapter.create_and_activate_context("new-context").await?;
    
    // Switch back to the default context
    adapter.switch_context("default").await?;
    
    Ok(())
}
```

## Features

- **Thread-Safe State Management**: All components are designed to be thread-safe using appropriate concurrency primitives.
- **Versioned State**: States are versioned to prevent conflicts and ensure consistency.
- **Automatic Recovery**: The system can automatically recover from failures using recovery points.
- **Configurable Components**: Each component provides configuration options for customization.
- **Factory Pattern**: Factory implementations for creating preconfigured components.
- **Comprehensive Error Handling**: Detailed error types for better error handling and debugging.

## Architecture

The system is designed with a modular architecture, with clear separation of concerns between components:

```
┌───────────────────┐     ┌───────────────────┐     ┌───────────────────┐
│   ContextAdapter  │────▶│   ContextManager  │────▶│  PersistenceManager│
└───────────────────┘     └───────────────────┘     └───────────────────┘
         │                         │                          │
         ▼                         ▼                          ▼
┌───────────────────┐     ┌───────────────────┐     ┌───────────────────┐
│  ContextTracker   │────▶│    ContextState   │────▶│   StateStorage    │
└───────────────────┘     └───────────────────┘     └───────────────────┘
```

## Error Handling

The system provides comprehensive error handling through the `ContextError` enum, which covers various error types such as:

- State errors
- Persistence errors
- Recovery errors
- Synchronization errors
- Lock acquisition failures
- Version conflicts

## Thread Safety

All components in the system are designed to be thread-safe, using appropriate concurrency primitives such as:

- `Arc` for shared ownership
- `Mutex` for exclusive access
- `RwLock` for reader-writer access
- `AsyncMutex` for asynchronous exclusive access

## License

This project is licensed under the MIT License - see the LICENSE file for details. 