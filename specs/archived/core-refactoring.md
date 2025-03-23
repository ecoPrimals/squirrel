---
title: Core Specifications Refactoring Plan
version: 1.0.0
date: 2024-05-15
status: draft
priority: high
---

# Core Specifications Refactoring Plan

## Overview

This document outlines a plan to refactor the `specs/core` directory to align with the minimal core crate, identifying which specs to keep, which to move, and why.

## Current State Assessment

The `specs/core` directory currently contains:

1. **error-recovery.md**: Error recovery mechanisms for the MCP system
2. **storage-manager.md**: Storage interface for MCP components
3. **error-handler.md**: Error handling strategies
4. **llm-system-prompt.md**: LLM system prompts documentation
5. **AI-TOOLS.md**: AI tool integration documentation

However, the `crates/core` implementation is intentionally minimal, containing only:
- Shared error types and utilities
- Build information
- Basic status functionality

## Refactoring Strategy

### 1. Core Specifications to Retain

Based on the minimal nature of the core crate, we should retain only specifications that describe truly core, shared functionality:

**error-foundation.md** (to be created)
- Will define the foundational error system used throughout the ecosystem
- Will include error type hierarchy, error creation patterns, and error handling principles
- Aligns with the actual error handling implementation in `crates/core/src/error.rs`

**versioning.md** (to be created)
- Will document the build information and versioning system
- Will include versioning strategy, build metadata, and version reporting
- Aligns with the build info implementation in `crates/core/src/lib.rs`

### 2. Specifications to Relocate

The following specifications should be moved to more domain-appropriate locations:

1. **error-recovery.md** → `specs/plugins/error-handling/error-recovery.md`
   - Rationale: This describes MCP-specific error recovery mechanisms, which are not part of the minimal core but should be in the plugin system.

2. **storage-manager.md** → `specs/plugins/storage/storage-manager.md`
   - Rationale: Storage management is a specialized service that should be implemented as a plugin, not in core.

3. **error-handler.md** → `specs/plugins/error-handling/error-handler.md`
   - Rationale: While related to errors, this describes specific handling strategies that go beyond the core error types and should be plugin functionality.

4. **llm-system-prompt.md** → `specs/plugins/ai/llm-system-prompt.md`
   - Rationale: LLM prompts are specific to AI functionality and should be part of an AI plugin, not core.

5. **AI-TOOLS.md** → `specs/plugins/ai/ai-tools.md`
   - Rationale: AI tool integration is a specialized domain that should be part of an AI plugin, not core.

### 3. New Core Specifications to Create

To properly document the minimal core functionality, we need to create:

1. **error-foundation.md**
   ```
   # Error Foundation Specification
   
   ## Overview
   This specification defines the foundational error system used throughout the Squirrel ecosystem.
   
   ## Error Type Hierarchy
   - SquirrelError: Root error type
     - AppInitializationError: Application startup errors
     - AppOperationError: Runtime operation errors
     - IOError: System I/O errors
     - SecurityError: Security-related errors
     - etc.
   
   ## Error Creation Patterns
   - Factory functions for creating specific error types
   - Context addition for error enrichment
   - Error chaining for maintaining error history
   
   ## Error Handling Principles
   - When to return errors vs when to handle them
   - Logging requirements for errors
   - Error translation across component boundaries
   ```

2. **versioning.md**
   ```
   # Versioning Specification
   
   ## Overview
   This specification defines the versioning system and build information management for the Squirrel ecosystem.
   
   ## Versioning Strategy
   - Semantic versioning requirements
   - Version number components
   - Version compatibility guarantees
   
   ## Build Metadata
   - Build identification
   - Build timestamp handling
   - Environment tagging
   
   ## Version Reporting
   - API for retrieving version information
   - Version display formatting
   - Version compatibility checking
   ```

3. **core-boundaries.md**
   ```
   # Core Boundaries Specification
   
   ## Overview
   This specification defines what belongs in the core crate versus domain-specific crates or plugins.
   
   ## Core Responsibilities
   - Error type definitions
   - Build and version information
   - Minimal status reporting
   
   ## Non-Core Responsibilities
   - Domain-specific errors
   - Specialized services
   - Feature implementations
   
   ## Dependency Guidelines
   - What can depend on core
   - What core can depend on
   - Circular dependency prevention
   ```

## Implementation Plan

### Phase 1: Directory Structure Preparation (Week 1)

1. Create new directories in the plugins structure:
   ```bash
   mkdir -p specs/plugins/error-handling
   mkdir -p specs/plugins/storage
   mkdir -p specs/plugins/ai
   ```

2. Create the new core specification files:
   ```bash
   touch specs/core/error-foundation.md
   touch specs/core/versioning.md
   touch specs/core/core-boundaries.md
   ```

### Phase 2: Content Migration (Week 1-2)

1. Develop the content for the new core specification files based on the templates above and the actual implementation in `crates/core`.

2. Move existing specifications to their new locations:
   ```bash
   git mv specs/core/error-recovery.md specs/plugins/error-handling/
   git mv specs/core/storage-manager.md specs/plugins/storage/
   git mv specs/core/error-handler.md specs/plugins/error-handling/
   git mv specs/core/llm-system-prompt.md specs/plugins/ai/
   git mv specs/core/AI-TOOLS.md specs/plugins/ai/ai-tools.md
   ```

3. Update references across specifications to point to the new locations.

### Phase 3: Documentation Updates (Week 2)

1. Update the main plugin documentation to reference the new specification locations.

2. Create cross-reference links between related specifications.

3. Update any implementation code that may reference these specifications.

### Phase 4: Validation (Week 2)

1. Review all specifications to ensure they correctly represent their proper domains.

2. Verify that core specifications align with the actual `crates/core` implementation.

3. Check that all cross-references are valid and working.

## Result

After refactoring, the `specs/core` directory will contain only:

- **error-foundation.md**: Defining the core error type system
- **versioning.md**: Defining the versioning and build info system
- **core-boundaries.md**: Defining what belongs in core vs. elsewhere

This alignment will ensure that the specifications accurately reflect the minimal but essential role of the core crate, while domain-specific functionality is properly documented within the plugin architecture.

## Conclusion

By refining the `specs/core` directory to focus only on truly core functionality, we can maintain the benefits of a minimal shared foundation while enabling the plugin architecture to evolve appropriately. This refactoring ensures that specifications are located where they are most relevant, making the documentation more accurate and easier to navigate.

Domain-specific functionality will be properly documented within the plugin architecture, aligning with our overall strategy of building a robust, maintainable system of loosely coupled components that follow clear responsibility boundaries. 