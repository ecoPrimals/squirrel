# Squirrel Integration Examples

This directory contains examples that demonstrate the real-world usage of the Squirrel integration components.

## Context-MCP Integration Example

The `context_mcp_use_case.rs` example demonstrates how to use the Context-MCP adapter to synchronize data between the Squirrel Context system and the Machine Context Protocol (MCP).

### Features Demonstrated

1. Creating and initializing the Context-MCP adapter
2. Creating contexts in the Squirrel Context system
3. Automatic bidirectional synchronization
4. Manual synchronization
5. Status monitoring
6. Error handling
7. Context updates and deletion

### Running the Example

From the `crates/integration` directory, run:

```bash
cargo run --example context_mcp_use_case
```

For detailed logs, set the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug cargo run --example context_mcp_use_case
```

### Example Output

The example will:

1. Create a user preferences context in the Squirrel Context system
2. Wait for automatic synchronization to MCP
3. Update the context and verify synchronization
4. Create a second context and perform a manual sync
5. Retrieve and display context data
6. Delete a context and verify synchronization
7. Display continuous status updates in the background

### Notes

- The example uses a short sync interval (5 seconds) for demonstration purposes
- In a production environment, you may want to use a longer interval
- The example requires both the Context system and MCP to be properly configured
- If running in a CI environment without actual services, some operations may fail gracefully 