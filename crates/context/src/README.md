# Context Management System

The Context Management System provides robust functionality for storing, tracking, and synchronizing application state across different components.

## Features

- Thread-safe state management
- Version-based state tracking
- Persistent state storage
- Snapshot-based recovery
- Concurrent access support

## Components

- **Context Manager**: Core component for managing context states
- **Context Tracker**: Handles tracking of state changes
- **Context Adapter**: Provides adapter pattern for system integration
- **Persistence Manager**: Handles state persistence
- **Recovery System**: Manages recovery points and state restoration

## Usage Examples

### Basic Usage

```rust
use squirrel_context::{create_manager, ContextState};
use std::sync::Arc;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    // Create a context manager
    let manager = create_manager();
    
    // Create a context state
    let mut state = ContextState {
        id: "example-1".to_string(),
        version: 1,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        data: HashMap::new(),
        metadata: HashMap::new(),
        synchronized: false,
    };
    
    // Add some data
    state.data.insert("key1".to_string(), "value1".to_string());
    
    // Create context
    manager.create_context("context-1", state).await.unwrap();
    
    // Get context state
    let retrieved_state = manager.get_context_state("context-1").await.unwrap();
    
    // Update context state
    let mut updated_state = retrieved_state.clone();
    updated_state.version += 1;
    updated_state.data.insert("key2".to_string(), "value2".to_string());
    
    manager.update_context_state("context-1", updated_state).await.unwrap();
}
```

### Concurrent Access

The context system is designed for safe concurrent access using tokio's asynchronous locks.
Here's an example of proper concurrent access patterns:

```rust
use squirrel_context::{create_manager, ContextState};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::task;

#[tokio::main]
async fn main() {
    // Create a shared context manager
    let manager = Arc::new(create_manager());
    
    // Create initial context
    let initial_state = ContextState {
        id: "initial".to_string(),
        version: 1,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        data: HashMap::new(),
        metadata: HashMap::new(),
        synchronized: false,
    };
    
    manager.create_context("shared-context", initial_state).await.unwrap();
    
    // Spawn multiple tasks that access the same context
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let manager_clone = manager.clone();
        
        let handle = task::spawn(async move {
            // Read the current state
            let mut state = manager_clone.get_context_state("shared-context").await.unwrap();
            
            // Modify the state
            state.version += 1;
            state.data.insert(format!("key-{}", i), format!("value-{}", i));
            
            // Update the state
            manager_clone.update_context_state("shared-context", state).await.unwrap();
            
            // The manager handles locking internally, so concurrent access is safe
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Get the final state
    let final_state = manager.get_context_state("shared-context").await.unwrap();
    println!("Final version: {}", final_state.version);
    println!("Number of keys: {}", final_state.data.len());
}
```

### Context Adapter Example

```rust
use squirrel_context::{create_manager, create_adapter, ContextState};
use std::sync::Arc;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    // Create a context manager
    let manager = create_manager();
    
    // Create a context adapter
    let adapter = create_adapter(manager.clone());
    
    // Initialize the adapter
    adapter.initialize().await.unwrap();
    
    // Create a context using the adapter
    adapter.create_and_activate_context("adapter-context").await.unwrap();
    
    // Use the current context
    let tracker = adapter.get_active_context().await.unwrap();
    
    // Update state through the tracker
    let mut state = tracker.get_state().await.unwrap();
    state.version += 1;
    state.data.insert("adapter-key".to_string(), "adapter-value".to_string());
    
    tracker.update_state(state).await.unwrap();
    
    // Sync state back to the manager
    tracker.sync_state().await.unwrap();
}
```

## Async Lock Best Practices

When working with the context system, follow these best practices for handling async locks:

1. **Minimize Lock Duration**: 
   ```rust
   // Good: Short lock duration
   let value = {
       let data = lock.read().await;
       data.get_value().clone()
   }; // Lock is released here
   
   // Process value without holding the lock
   process_value(value);
   ```

2. **Avoid Holding Locks Across Await Points**:
   ```rust
   // Bad:
   let data = lock.read().await;
   let result = some_async_operation().await; // Lock is held across await
   use_data(&data, result);
   
   // Good:
   let data_copy = {
       let data = lock.read().await;
       data.clone()
   }; // Lock is released here
   
   let result = some_async_operation().await;
   use_data(&data_copy, result);
   ```

3. **Use Separate Locks for Read and Write Operations**:
   ```rust
   // First read to check
   let should_update = {
       let data = lock.read().await;
       data.needs_update()
   }; // Read lock is released
   
   if should_update {
       // Then write to update
       let mut data = lock.write().await;
       data.update();
   } // Write lock is released
   ```

4. **Explicitly Drop Locks Before Long Operations**:
   ```rust
   let item_id = {
       let items = items_lock.read().await;
       items.get(0).map(|item| item.id.clone())
   }; // Lock is explicitly dropped here
   
   if let Some(id) = item_id {
       // Long operation without holding the lock
       process_item(&id).await;
   }
   ```

## Performance Considerations

- Using read locks for read-only operations allows concurrent reads
- Minimize the duration for which write locks are held
- Use clone or copy to avoid holding locks for long operations
- Consider using lock-free data structures for high-contention scenarios

## Thread Safety

All components in the context system are designed to be thread-safe and can be safely shared across tasks using `Arc`.

## Error Handling

The context system uses a robust error handling system with specific error types:

```rust
// All operations return a Result type
let result = manager.get_context_state("non-existent").await;

match result {
    Ok(state) => println!("Found state: {:?}", state),
    Err(err) => match err {
        ContextError::NotFound(msg) => println!("Context not found: {}", msg),
        ContextError::InvalidState(msg) => println!("Invalid state: {}", msg),
        _ => println!("Other error: {}", err),
    },
}
``` 