---
description: Plan for consolidating duplicate context adapter crates
version: 1.1.0
last_updated: 2024-03-26
status: completed
---

# Context Adapter Consolidation Plan

## Overview

Our codebase previously had two potentially duplicate context adapter crates with different naming conventions:
- `crates/context_adapter/` (underscore version)
- `crates/context-adapter/` (hyphen version)

This document outlines the consolidation plan that has been implemented to reduce confusion and maintain a clean codebase.

## Analysis of Current State

### Hyphen Version (`crates/context-adapter/`)
- **Cargo.toml**: Fully configured with dependencies
- **Source Files**: 
  - Contains `lib.rs` with proper documentation
  - Contains `adapter.rs` with substantial implementation (~9.5KB)
  - Contains a `tests/` directory with test implementations
- **Package Name**: `squirrel-context-adapter`
- **Crate Name**: `squirrel_context_adapter`

### Implementation Decision

Based on the analysis, we decided to:

1. **Keep**: `crates/context-adapter/` (hyphen version)
   - This is the main implementation with proper structure
   - Contains actual functionality and tests
   - Has a properly configured Cargo.toml

2. **Standardize**: Use hyphenated directory names with underscore crate names
   - Directory Names: Use hyphens (`context-adapter/`)
   - Package Names: Use hyphens (`squirrel-context-adapter`)
   - Crate/Module Names: Use underscores (`squirrel_context_adapter`)

## Implementation Status

The consolidation has been completed:
- The hyphenated version (`crates/context-adapter/`) has been kept and standardized
- Integration with MCP has been maintained through the MCP-specific adapter
- Workspace Cargo.toml references the correct path

## Naming Convention Standard

Moving forward, we will adhere to the following convention for all crates:

- **Directory Names**: Use hyphens (`crate-name/`)
- **Package Names**: Use hyphens (`crate-name`)
- **Crate/Module Names**: Use underscores (`crate_name`)

## Verification

- [x] Build the entire workspace with no compilation errors
- [x] Run tests for the context adapter crate
- [x] Run integration tests that rely on the context adapter

<version>1.1.0</version> 