# Context-MCP Integration

This module implements the integration between the Squirrel Context Management system and the Machine Context Protocol (MCP).

## Overview

The Context-MCP integration enables bidirectional synchronization of context data between the Squirrel Context system and MCP. It facilitates seamless data flow and ensures that context changes in one system are reflected in the other.

## Key Components

- **ContextMcpAdapter**: The central component that bridges the Context system and MCP
- **ContextMcpAdapterConfig**: Configuration options for the adapter
- **SyncDirection**: Enum defining the direction of synchronization
- **AdapterStatus**: Struct holding the current status of the adapter

## Design Patterns

The integration implements two key design patterns:

1. **Adapter Pattern**: Provides a clean interface between disparate systems
2. **Circuit Breaker Pattern**: Ensures resilience during failures

## Usage

Basic usage:

```rust
// Create an adapter with default configuration
let adapter = create_context_mcp_adapter(None).await?;

// Initialize the adapter
adapter.initialize().await?;

// Sync data in both directions
adapter.sync_all().await?;

// Get the adapter's status
let status = adapter.get_status().await;
println!("Sync status: {:#?}", status);
```

With custom configuration:

```rust
let config = ContextMcpAdapterConfig {
    sync_interval_secs: 30,
    ..Default::default()
};

let adapter = create_context_mcp_adapter(Some(config)).await?;
```

## Features

- **Bidirectional Synchronization**: Sync data from Context to MCP and vice versa
- **Real-time Updates**: Reflect changes immediately in both systems
- **Resilient Operation**: Circuit breaker pattern prevents cascading failures
- **Efficient ID Mapping**: Map between Context IDs and MCP UUIDs
- **Configurable Sync Interval**: Set how often automatic syncs occur
- **Status Monitoring**: Track sync status, error counts, and connectivity

## Testing

The module includes comprehensive tests:

- Unit tests for all components
- Integration tests with mocked dependencies
- Performance benchmarks
- Circuit breaker functionality tests

Run the tests:

```bash
cargo test --package squirrel-integration --lib context_mcp
```

## Example

For a complete use case example, see [context_mcp_use_case.rs](../../examples/context_mcp_use_case.rs) in the examples directory.

## Documentation

For detailed specifications, see [Context-MCP Integration Spec](../../../../specs/integration/context-mcp-integration.md).

## Error Handling

The module defines custom error types in `errors.rs` to handle integration-specific errors:

- Connection failures
- Sync failures
- ID mapping errors

## Performance Considerations

- Uses efficient ID mapping to reduce lookup costs
- Implements batched operations for bulk updates
- Configurable sync interval prevents overwhelming either system
- Parallel processing where appropriate 