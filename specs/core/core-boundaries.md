---
title: Core Boundaries Specification
version: 1.1.0
date: 2024-09-30
status: active
priority: high
---

# Core Boundaries Specification

## Overview

This specification defines the boundaries between the core crate and domain-specific crates or plugins in the Squirrel ecosystem. It establishes clear guidelines for what belongs in the minimal shared core versus what should be implemented in specialized components, ensuring a maintainable architecture with appropriate separation of concerns.

## Core Crate Responsibilities

The core crate (`squirrel-core`) serves as a minimal foundation for the entire ecosystem. It should include only:

### 1. Error Foundation

- Core error types that span the entire ecosystem
- Error creation factories and utility functions
- Common error result type definitions
- Error context and chain management
- Base error handling traits and patterns

```rust
// Core error types
pub enum SquirrelError {
    AppInitialization(AppInitializationError),
    AppOperation(AppOperationError),
    IO(std::io::Error),
    // ...other fundamental error categories
}

// Core result type
pub type Result<T> = std::result::Result<T, SquirrelError>;
```

### 2. Version and Build Information

- Version retrieval and management
- Build information collection and reporting
- Version compatibility checking utilities
- Basic application identity

```rust
// Version and build information
pub mod build_info {
    // Build information retrieval
}

pub struct VersionInfo {
    pub version: String,
    pub build_date: String,
    pub git_commit: String,
    pub rust_version: String,
}
```

### 3. Status Reporting

- Basic application status structures
- Health reporting utilities
- Minimal diagnostic information

```rust
// Status reporting
pub struct Status {
    pub uptime: Duration,
    pub memory_usage: u64,
    pub active_commands: usize,
    pub connected_clients: usize,
}
```

### 4. Shared Constants and Configuration

- System-wide constants
- Configuration value types (but not configuration handling)
- Common utility types used across all modules

```rust
// Shared constants
pub const MAX_BUFFER_SIZE: usize = 8192;
pub const DEFAULT_TIMEOUT_MS: u64 = 30000;
```

## Non-Core Responsibilities

The following responsibilities should NOT be part of the core crate and should instead be implemented in domain-specific crates or plugins:

### 1. Domain-Specific Error Types

- Specialized error types for specific domains
- Custom error handling strategies
- Recovery mechanisms for specific scenarios

These should be implemented in domain-specific crates with appropriate mapping to core error types at boundaries.

### 2. Feature Implementation

- Application features
- User interface components
- Specialized algorithms
- Domain-specific logic

These should be implemented in feature-specific crates or plugins.

### 3. Service Implementation

- Storage engines
- Network protocols
- Security frameworks
- External system integration

These should be implemented as plugins or specialized service crates.

### 4. Configuration Management

- Configuration loading
- Configuration parsing
- Configuration validation
- User preferences

These should be implemented in a dedicated configuration plugin or service.

### 5. MCP Protocol Implementation

- Protocol handlers
- Message formats
- State machines
- Protocol extensions

These should be implemented in MCP-specific plugins or crates.

## Dependency Guidelines

### Core Dependencies

Core should have minimal dependencies:

1. **Standard Library**: The core crate should primarily rely on the Rust standard library
2. **Build Utilities**: Minimal dependencies for build information (e.g., `built` crate)
3. **Semantic Versioning**: Version parsing and comparison utilities (e.g., `semver` crate)

The core crate should NOT depend on:
- GUI frameworks
- Networking libraries
- Database drivers
- Third-party services
- Domain-specific libraries

### Dependencies on Core

Other components should:

1. Depend on core for shared error types and utilities
2. Use core APIs consistently throughout the ecosystem
3. NOT try to extend core functionality through unsafe or hack approaches
4. NOT duplicate core functionality

### Circular Dependencies

To prevent circular dependencies:

1. Core must NEVER depend on domain-specific crates
2. Feature crates must not depend on each other directly
3. Cross-cutting concerns should be handled through plugin interfaces
4. Common utilities should be moved to core if truly needed everywhere

## Migration Path for Non-Core Functionality

When functionality is identified as non-core:

1. Create a new plugin or domain-specific crate
2. Implement the functionality there
3. If needed, add minimal interfaces to core
4. Migrate usage to the new implementation
5. Remove non-core functionality from core

## Decision Framework

When deciding if functionality belongs in core, ask:

1. Is it used by virtually all components in the ecosystem?
2. Does it define system-wide interfaces or types?
3. Is it stable and unlikely to change frequently?
4. Is it minimal and focused on a cross-cutting concern?

If the answer to ANY of these questions is NO, the functionality likely belongs in a plugin or domain-specific crate.

## Implementation Status

- Error foundation boundaries: **Implemented**
- Version and build information boundaries: **Implemented**
- Status reporting boundaries: **Implemented**
- Dependency guidelines: **Partially Implemented**
- Migration path documentation: **Planned**

## Future Considerations

While maintaining minimalism, future considerations include:

1. Establishing a plugin registry interface in core
2. Defining minimal lifecycle management traits
3. Creating core extension points for essential cross-cutting concerns

## Conclusion

By maintaining clear boundaries between core and domain-specific functionality, the Squirrel ecosystem can achieve a balance between shared infrastructure and specialized implementations. The core crate provides essential functionality used throughout the system while remaining minimal and focused, enabling domain-specific components to evolve independently without unnecessary coupling. 