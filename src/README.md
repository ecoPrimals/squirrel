# Squirrel Main Crate

## Overview

This directory contains the main Squirrel crate that serves as the primary integration point for the Squirrel ecosystem. While most of the actual implementation is located in the individual crates within the `crates/` directory, this main crate provides:

1. **Public API Surface**: Curated re-exports from other crates to provide a clean, unified API
2. **Integration Interfaces**: Adapters and interfaces for cross-crate communication
3. **Entry Point**: Central access point for applications using the Squirrel ecosystem

## Directory Structure

```
src/
├── adapter/         # Adapter interfaces for MCP integration
├── security/        # Security-related functionality 
├── tests/           # Integration tests for the main crate
└── lib.rs           # Main entry point and re-exports
```

## Purpose and Design Philosophy

The main crate serves a different purpose than the individual implementation crates:

- **Implementation Crates** (`crates/`): Each crate contains a focused implementation of a specific subsystem with minimal dependencies on other crates
- **Main Crate** (`src/`): Provides integration, re-exports, and adapter interfaces between subsystems

This separation of concerns allows:

1. **Loose Coupling**: Implementation crates can evolve independently
2. **Dependency Management**: Circular dependencies are avoided through adapter interfaces
3. **API Stability**: The main crate can maintain a stable API while implementation details change
4. **Discoverability**: Users have a single entry point into the ecosystem

## Key Components

### lib.rs

The main entry point that re-exports functionality from other crates, providing a clean, unified API surface for consumers. It includes:

- Direct re-exports of core functionality from implementation crates
- Public exports of adapter interfaces
- A prelude module for convenient imports

### adapter/

Contains adapter interfaces for the Machine Context Protocol (MCP) that allow integration with the system without creating circular dependencies. The primary interfaces include:

- `MCPInterface`: Trait defining the interface for MCP operations
- `MCPAdapter`: Implementation of the interface for the main crate
- `MCPConfig`: Configuration structure for MCP connections
- `Credentials`: Authentication structure for secure connections

### security/

Contains security-related functionality and interfaces that integrate with the security subsystems in implementation crates.

## Usage Guidelines

### For Consumers

If you're using the Squirrel ecosystem in your application, you should generally import from the main crate rather than individual implementation crates:

```rust
// Preferred:
use squirrel::prelude::*;

// Less preferred (more brittle):
use squirrel_mcp::tool::Tool;
use squirrel_context::manager::ContextManager;
```

### For Contributors

When contributing to the Squirrel ecosystem:

1. **New Functionality**: Should generally be implemented in the appropriate crate in `crates/`
2. **Integration Logic**: Should be implemented in this main crate
3. **Circular Dependencies**: Should be resolved through adapter interfaces in this main crate
4. **API Changes**: Consider the impact on the re-exports in `lib.rs`

## Relationship to Workspace

This main crate is part of the Squirrel workspace, which is organized as follows:

```
squirrel/
├── Cargo.toml         # Workspace manifest
├── src/               # Main crate (this directory)
└── crates/            # Implementation crates
    ├── app/           # Application core
    ├── cli/           # Command-line interface
    ├── commands/      # Command system
    ├── context/       # Context management
    ├── core/          # Core utilities
    ├── interfaces/    # Shared interfaces
    ├── mcp/           # Machine Context Protocol
    ├── monitoring/    # Monitoring system
    └── ...            # Other implementation crates
```

## Future Plans

While most new development should happen in the implementation crates, the main crate may evolve to provide:

1. **Enhanced Prelude**: More comprehensive imports for common use cases
2. **Feature Flags**: More granular control over which components are included
3. **Unified Configuration**: Centralized configuration for the entire ecosystem
4. **Integration Utilities**: More tools for connecting subsystems

## Contribution Guidelines

When working with the main crate:

1. Keep the adapter interfaces clean and focused
2. Ensure re-exports are documented and intentional
3. Maintain backward compatibility when possible
4. Add integration tests for cross-crate functionality

For more detailed contribution guidelines, please see the project's main README.md and CONTRIBUTING.md files. 