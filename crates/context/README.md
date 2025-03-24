# Context Management System

The Context Management System provides robust support for tracking, persisting, and synchronizing application state for the Squirrel AI Coding Assistant.

## Features

- Thread-safe state management using asynchronous locks
- Persistent storage with automatic recovery
- Flexible context tracking and synchronization
- Change notification system
- Plugin support for extensibility

## Core Components

The context system consists of several key components:

### Context Manager

The core management component that handles context creation, tracking, and persistence.

```rust
// Create a manager with default configuration
let manager = create_manager();

// Initialize the manager
manager.initialize().await?;

// Store context data
manager.store_context("user1", context_data).await?;

// Retrieve context data
let data = manager.get_context("user1").await?;
```

### Context Tracker

Tracks changes to context and provides notifications.

```rust
// Create a tracker
let tracker = create_tracker(manager.clone());

// Register for notifications
tracker.subscribe("user1", |event| {
    println!("Context updated: {:?}", event);
}).await?;
```

### Context Persistence

Manages storing and retrieving context data from persistent storage.

```rust
// Persistence is automatically managed by the context manager
// but can be accessed directly if needed
let persistence = manager.get_persistence_manager().await?;

// Store directly
persistence.store("key", data).await?;

// Retrieve directly
let data = persistence.retrieve("key").await?;
```

### Context Plugins

Extends the context system with custom functionality through plugins.

```rust
// Plugins are automatically loaded by the context manager
// but can be accessed directly if needed
let plugin_manager = manager.get_plugin_manager().await.unwrap();

// Transform data using a plugin
let transformed = manager.transform_data("context.standard", data).await?;

// Convert data format
let converted = manager.convert_data("json.to.mcp", data).await?;

// Validate data against schema
let is_valid = manager.validate_data(&schema, &data).await?;
```

## Usage Patterns

### Basic Context Management

```rust
use squirrel_context::{create_manager, ContextState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a manager
    let manager = create_manager();
    
    // Initialize the manager
    manager.initialize().await?;
    
    // Create a context
    let context_id = manager.create_context().await?;
    
    // Update context data
    let data = serde_json::json!({
        "user": "user1",
        "settings": {
            "theme": "dark",
            "font_size": 14
        }
    });
    
    manager.update_context(&context_id, data).await?;
    
    // Retrieve context data
    let context = manager.get_context(&context_id).await?;
    println!("Context: {:?}", context);
    
    Ok(())
}
```

### Using Context Plugins

```rust
use squirrel_context::{create_manager_with_config, ContextManagerConfig};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a manager with plugins enabled
    let config = ContextManagerConfig {
        enable_plugins: true,
        ..Default::default()
    };
    
    let manager = create_manager_with_config(config);
    
    // Initialize the manager
    manager.initialize().await?;
    
    // Transform data using a plugin
    let data = json!({
        "data": {
            "key": "value"
        }
    });
    
    let transformed = manager.transform_data("context.standard", data).await?;
    println!("Transformed: {:?}", transformed);
    
    // Get available transformations
    if let Some(plugin_manager) = manager.get_plugin_manager().await {
        let transformations = plugin_manager.get_transformations().await;
        println!("Available transformations: {:?}", transformations);
    }
    
    Ok(())
}
```

## Advanced Features

### Recovery Management

The context system provides automatic recovery capabilities for handling errors or crashes.

```rust
// Create a recovery point
manager.create_recovery_point(&context_id).await?;

// Restore from last recovery point
manager.restore_last_recovery_point(&context_id).await?;

// List available recovery points
let recovery_points = manager.list_recovery_points(&context_id).await?;
```

### Format Conversion

Convert between different data formats using adapter plugins.

```rust
// Convert JSON to MCP format
let mcp_data = manager.convert_data("json.to.mcp", json_data).await?;

// Convert MCP back to JSON
let json_data = manager.convert_data("mcp.to.json", mcp_data).await?;
```

## Plugin System

The context system includes plugin support for extending functionality:

### Context Plugin

Provides transformations and validation for context data.

```rust
// Register a custom context plugin
let context_plugin = create_custom_context_plugin(
    "My Plugin",
    "Custom transformation plugin",
    vec![my_transformation]
);

plugin_manager.register_plugin(context_plugin).await?;
```

### Context Adapter Plugin

Provides format conversion between different data representations.

```rust
// Register a custom adapter plugin
let adapter_plugin = create_custom_context_adapter_plugin(
    "My Adapter",
    "Custom format adapter",
    vec![my_adapter]
);

plugin_manager.register_adapter_plugin(adapter_plugin).await?;
```

## License

TBD 