---
title: Directory Structure Guide
version: 1.1.0
date: 2025-03-21
status: approved
---

# Directory Structure Guide

## Overview

This document provides a comprehensive guide to the directory structure of the Squirrel platform. It maps the relationship between specifications and implementation, explains the organization principles, and serves as a navigation aid for developers and AI agents working with the codebase.

## Top-Level Organization

The Squirrel platform is organized into the following top-level directories:

```
squirrel/
├── .cursor/            # Cursor-specific configuration
├── .github/            # GitHub workflows and templates
├── crates/             # Rust implementation crates
├── docs/               # General documentation
├── scripts/            # Utility scripts
├── specs/              # Specifications
├── tests/              # Integration and system tests
├── tools/              # Development tools
└── ui/                 # Frontend UI code
```

## Specifications Organization

The `specs/` directory contains all specifications for the platform:

```
specs/
├── app/                # Application Core specifications
│   ├── RELATIONSHIP.md # Component relationships
│   ├── README.md       # App Core overview
│   ├── REVIEW.md       # App Core review
│   ├── VERIFICATION.md # Verification status
│   ├── command-system.md  # Command system integration
│   ├── config-management.md # Configuration management
│   ├── context-management.md # Context management
│   ├── core-priorities.md # Development priorities
│   ├── error-handling.md  # Error handling strategy
│   ├── error-recovery.md  # Error recovery mechanisms
│   ├── performance.md  # Performance requirements
│   └── thread-safety.md # Thread safety guidelines
├── commands/           # Command System specifications
│   ├── README.md       # Commands overview
│   ├── REVIEW.md       # Commands review
│   └── design/         # Command design specifications
├── context/            # Context Management specifications
│   ├── README.md       # Context overview
│   └── REVIEW.md       # Context review
├── integration/        # Integration System specifications
│   ├── README.md       # Integration overview
│   ├── REVIEW.md       # Integration review
│   └── protocols/      # Integration protocol specifications
├── mcp/                # Management Control Plane specifications
│   ├── README.md       # MCP overview
│   └── REVIEW.md       # MCP review
├── monitoring/         # Monitoring System specifications
│   ├── README.md       # Monitoring overview
│   └── REVIEW.md       # Monitoring review
├── MVP/                # Minimum Viable Product specifications
│   └── README.md       # MVP requirements
├── patterns/           # Cross-cutting design patterns
│   ├── README.md       # Patterns overview
│   ├── PATTERN_TEMPLATE.md # Template for creating new patterns
│   ├── adapter-implementation-guide.md # Adapter pattern guide
│   ├── async-programming.md # Async programming patterns
│   ├── dependency-injection.md # DI patterns and best practices
│   ├── error-handling.md # Error handling patterns
│   ├── resource-management.md # Resource management patterns
│   └── schema-design.md # Schema design patterns
├── plugins/            # Plugin System specifications
│   ├── README.md       # Plugin overview
│   ├── REVIEW.md       # Plugin review
│   └── samples/        # Sample plugin specifications
├── teams/              # Team organization specifications
│   └── README.md       # Team structure and responsibilities
├── validation/         # Validation System specifications
│   ├── README.md       # Validation overview
│   ├── REVIEW.md       # Validation review
│   └── rules/          # Validation rule specifications
├── web/                # Web Interface specifications
│   ├── API.md          # Web API documentation
│   ├── Architecture.md # Web architecture
│   ├── Integration.md  # Web integration
│   ├── Performance.md  # Web performance
│   ├── README.md       # Web overview
│   ├── REVIEW.md       # Web review
│   ├── Security.md     # Web security
│   └── Testing.md      # Web testing
├── DIRECTORY_STRUCTURE.md  # This document
├── README.md           # Specifications overview
├── SECURITY.md         # Cross-cutting security specification
├── SPECS.md            # Specifications index and status
├── SPECS_REVIEW.md     # Review status of all specifications
├── TESTING.md          # Cross-cutting testing specification
└── WORKTEAMS.md        # Work team organization
```

### Specification Document Types

Each component directory typically contains several standardized document types, though the exact files may vary by component:

1. **README.md** - Overview of the component, its purpose, and key features
2. **REVIEW.md** - Critical review of the component's specifications
3. **Architecture.md** - Detailed architecture (when applicable)
4. **API.md** - API specification (when applicable)
5. **Security.md** - Component-specific security requirements (when applicable)
6. **Performance.md** - Performance requirements (when applicable)
7. **Testing.md** - Testing strategy (when applicable)
8. **Integration.md** - Integration points (when applicable)

### Cross-Cutting Specifications

Several documents and directories address cross-cutting concerns:

1. **patterns/** - Design patterns and implementation guides used across components
2. **SECURITY.md** - Platform-wide security requirements and patterns
3. **TESTING.md** - Cross-cutting testing requirements and standards
4. **SPECS.md** - Tracks the status of all specifications
5. **SPECS_REVIEW.md** - Documents the review process for all specifications
6. **DIRECTORY_STRUCTURE.md** - This document, explaining the organization
7. **WORKTEAMS.md** - Defines the team organization and responsibilities

## Implementation Organization

The `crates/` directory contains the Rust implementation of the platform:

```
crates/
├── app/                # Application Core implementation
│   ├── src/
│   │   ├── bin/        # Executable entry points
│   │   ├── core/       # Core application logic
│   │   ├── error.rs    # Error types
│   │   ├── lib.rs      # Library entry point
│   │   └── utils/      # Utility modules
│   ├── Cargo.toml      # Dependencies and metadata
│   └── tests/          # Crate-specific tests
├── cli/                # Command Line Interface implementation
│   ├── src/
│   │   ├── bin/        # CLI executables
│   │   ├── commands/   # Command implementations
│   │   ├── lib.rs      # Library entry point
│   │   └── ui/         # Text UI components
│   ├── Cargo.toml      # Dependencies and metadata
│   └── tests/          # CLI-specific tests
├── common/             # Common utilities and shared code
│   ├── src/
│   │   ├── config/     # Configuration utilities
│   │   ├── error.rs    # Common error types
│   │   ├── lib.rs      # Library entry point
│   │   └── utils/      # Shared utilities
│   ├── Cargo.toml      # Dependencies and metadata
│   └── tests/          # Common utils tests
├── context/            # Context Management implementation
│   ├── src/
│   │   ├── lib.rs      # Library entry point
│   │   ├── manager.rs  # Context manager implementation
│   │   ├── store/      # Context storage implementations
│   │   └── types.rs    # Context type definitions
│   ├── Cargo.toml      # Dependencies and metadata
│   └── tests/          # Context-specific tests
├── commands/           # Command System implementation
│   ├── src/
│   │   ├── executor.rs # Command executor
│   │   ├── lib.rs      # Library entry point
│   │   ├── registry.rs # Command registry
│   │   └── types.rs    # Command type definitions
│   ├── Cargo.toml      # Dependencies and metadata
│   └── tests/          # Command-specific tests
├── integration/        # Integration System implementation
│   ├── src/
│   │   ├── adapters/   # Protocol adapters
│   │   ├── lib.rs      # Library entry point
│   │   ├── protocols/  # Protocol implementations
│   │   └── registry.rs # Protocol registry
│   ├── Cargo.toml      # Dependencies and metadata
│   └── tests/          # Integration-specific tests
├── mcp/                # Management Control Plane implementation
│   ├── src/
│   │   ├── api/        # MCP API implementations
│   │   ├── controllers/ # Control logic
│   │   ├── lib.rs      # Library entry point
│   │   └── state.rs    # System state management
│   ├── Cargo.toml      # Dependencies and metadata
│   └── tests/          # MCP-specific tests
├── monitoring/         # Monitoring System implementation
│   ├── src/
│   │   ├── collectors/ # Metric collectors
│   │   ├── lib.rs      # Library entry point
│   │   ├── metrics/    # Metric definitions
│   │   └── reporters/  # Report generators
│   ├── Cargo.toml      # Dependencies and metadata
│   └── tests/          # Monitoring-specific tests
├── plugins/            # Plugin System implementation
│   ├── src/
│   │   ├── lib.rs      # Library entry point
│   │   ├── loader.rs   # Plugin loader
│   │   ├── registry.rs # Plugin registry
│   │   └── sandbox.rs  # Plugin sandbox
│   ├── Cargo.toml      # Dependencies and metadata
│   └── tests/          # Plugin-specific tests
├── validation/         # Validation System implementation
│   ├── src/
│   │   ├── engine.rs   # Validation engine
│   │   ├── lib.rs      # Library entry point
│   │   ├── rules/      # Rule implementations
│   │   └── types.rs    # Validation type definitions
│   ├── Cargo.toml      # Dependencies and metadata
│   └── tests/          # Validation-specific tests
└── web/                # Web Interface implementation
    ├── src/
    │   ├── api/        # API implementation
    │   ├── auth/       # Authentication
    │   ├── bin/        # Web server executable
    │   ├── handlers/   # Request handlers
    │   ├── lib.rs      # Library entry point
    │   ├── middleware/ # Web middleware
    │   ├── routes.rs   # API routes
    │   └── state.rs    # Application state
    ├── Cargo.toml      # Dependencies and metadata
    └── tests/          # Web-specific tests
```

### Crate Organization Principles

Each crate follows these organization principles:

1. **Single Responsibility**: Each crate has a specific, well-defined responsibility
2. **Clear API Boundaries**: Public APIs are clearly marked and documented
3. **Minimal Dependencies**: Dependencies are kept to a minimum and clearly justified
4. **Consistent Structure**: Similar structure across crates for easier navigation
5. **Comprehensive Testing**: Tests are included alongside the implementation

## Specification-to-Implementation Mapping

The specification directories map directly to implementation crates:

| Specification Directory | Implementation Crate | Description                        |
|------------------------|----------------------|------------------------------------|
| `specs/app/`           | `crates/app/`        | Application Core                   |
| `specs/cli/`           | `crates/cli/`        | Command Line Interface             |
| `specs/commands/`      | `crates/commands/`   | Command System                     |
| `specs/context/`       | `crates/context/`    | Context Management                 |
| `specs/integration/`   | `crates/integration/`| Integration System                 |
| `specs/mcp/`           | `crates/mcp/`        | Management Control Plane           |
| `specs/monitoring/`    | `crates/monitoring/` | Monitoring System                  |
| `specs/plugins/`       | `crates/plugins/`    | Plugin System                      |
| `specs/validation/`    | `crates/validation/` | Validation System                  |
| `specs/web/`           | `crates/web/`        | Web Interface                      |

## Component Dependencies

The following diagram illustrates the high-level dependencies between components:

```
                   ┌─────────────┐
                   │     App     │
                   └─────┬───────┘
                         │
         ┌───────┬───────┼───────┬───────┐
         │       │       │       │       │
┌────────▼─┐ ┌───▼───┐ ┌─▼───┐ ┌─▼────┐ ┌▼────────┐
│ Commands │ │Context│ │ CLI │ │ Web  │ │Validation│
└────┬─────┘ └───┬───┘ └─────┘ └──┬───┘ └────┬────┘
     │           │               │          │
┌────▼───────────▼───────────────▼──────────▼────┐
│                  Common                        │
└────┬───────────────┬───────────────┬───────────┘
     │               │               │
┌────▼────┐     ┌────▼─────┐    ┌────▼─────┐
│Monitoring│     │Integration│    │  Plugins │
└─────┬────┘     └──────────┘    └──────────┘
      │
┌─────▼────┐
│    MCP   │
└──────────┘
```

### Key Relationships

1. **App Core**: Central component that coordinates all others
2. **Common**: Provides shared utilities and types to all components
3. **Commands & Context**: Tightly integrated for command execution
4. **CLI & Web**: Primary user interfaces that depend on App Core
5. **MCP**: Depends on Monitoring for system status information
6. **Plugins**: Extends functionality of various components
7. **Validation**: Used by multiple components for input/state validation

## File Naming Conventions

### Specification Files

- **Uppercase Names**: High-level documents (README.md, ARCHITECTURE.md, REVIEW.md)
- **Lowercase Names**: Detailed specifications (error-handling.md, core-priorities.md)
- **Directories**: All lowercase with hyphens for multi-word names

### Implementation Files

- **Rust Files**: Snake case (error_handling.rs, command_executor.rs)
- **Directories**: All lowercase with underscores for multi-word names
- **Bin Files**: Executable entry points in `src/bin/` directories

## Documentation Standards

Each specification document follows these standards:

1. **Metadata Header**: Title, version, date, and status
2. **Consistent Structure**: Clear sections and subsections
3. **Comprehensive Coverage**: All aspects of the component are covered
4. **Cross-References**: Links to related specifications
5. **Version History**: Changes tracked through version numbers

## Navigation Tips for AI Agents

When navigating the codebase, consider these approaches:

1. **Start with Specifications**: Begin in the `specs/` directory to understand requirements
2. **Check Patterns Directory**: Review `specs/patterns/` for design patterns used throughout the codebase
3. **Map to Implementation**: Use the mapping table to find corresponding code
4. **Explore Dependencies**: Understand how components interact using the dependency diagram
5. **Check Common First**: Common utilities are often used throughout the codebase
6. **Reference Cross-Cutting Concerns**: Security, performance, and testing are defined in cross-cutting specifications

### Recommended Exploration Path

For understanding a new component:

1. Read `specs/component/README.md` for overview
2. Read `specs/component/REVIEW.md` for critical analysis
3. Read architecture documentation or detailed specifications
4. Check relevant patterns in `specs/patterns/`
5. Look at `crates/component/src/lib.rs` for implementation entry points
6. Explore key implementation files based on architectural guidance

## Conclusion

This directory structure guide provides a comprehensive map of the Squirrel platform organization. By understanding the relationships between specifications and implementation, developers and AI agents can more efficiently navigate and contribute to the codebase.

Use this document as a reference when exploring the platform, and refer to the specific component documentation for more detailed information about each part of the system.

<version>1.1.0</version> 