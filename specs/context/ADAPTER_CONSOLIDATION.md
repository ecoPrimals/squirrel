---
description: Plan for consolidating duplicate context adapter crates
version: 1.0.0
last_updated: 2024-03-21
status: draft
---

# Context Adapter Consolidation Plan

## Overview

Our codebase currently has two potentially duplicate context adapter crates with different naming conventions:
- `crates/context_adapter/` (underscore version)
- `crates/context-adapter/` (hyphen version)

This document outlines the plan to consolidate these into a single crate to reduce confusion and maintain a clean codebase.

## Analysis of Current State

### Underscore Version (`crates/context_adapter/`)
- **Cargo.toml**: Fully configured with dependencies
- **Source Files**: 
  - Contains `lib.rs` with proper documentation
  - Contains `adapter.rs` with substantial implementation (~9.5KB)
  - Contains a `tests/` directory with test implementations
- **Package Name**: `squirrel-context-adapter`
- **Crate Name**: `squirrel_context_adapter`

### Hyphen Version (`crates/context-adapter/`)
- **Cargo.toml**: Not found in the directory root
- **Source Files**:
  - Contains minimal `lib.rs` with only linting configurations
  - Contains minimal `adapter.rs` with little to no implementation
- **Package/Crate Name**: Unknown due to missing Cargo.toml

## Consolidation Decision

Based on the analysis:

1. **Keep**: `crates/context_adapter/` (underscore version)
   - Appears to be the main implementation with proper structure
   - Contains actual functionality and tests
   - Has a properly configured Cargo.toml

2. **Remove**: `crates/context-adapter/` (hyphen version)
   - Appears to be either a new placeholder or an incomplete copy
   - Lacks proper Cargo.toml configuration
   - Contains minimal implementation

## Implementation Plan

### 1. Code Verification

- Verify that no unique functionality exists in the hyphen version
- Ensure all necessary functionality is contained in the underscore version
- Check for any references to the hyphen version throughout the codebase

### 2. Rename Standardization

To maintain consistency with Rust's package naming conventions while aligning with our project standards:

- Update package paths to use hyphens: `squirrel-context-adapter`
- Keep module/crate names with underscores: `squirrel_context_adapter`
- Update the directory to use hyphens for consistency: rename `context_adapter` to `context-adapter`

### 3. Codebase Updates

- Update any imports or references to `context_adapter` in other crates
- Update any documentation references to the crate
- Ensure workspace Cargo.toml references the correct path

### 4. Remove Duplicate

- Remove the duplicate `crates/context-adapter/` directory after ensuring all functionality is preserved

### 5. Documentation Updates

- Update `specs/SPECS_REVIEW.md` to reflect the consolidation
- Remove the duplicate entry from the crates list
- Document the standard naming convention for future crates

## Naming Convention Standard

Moving forward, we will adopt the following convention for all crates:

- **Directory Names**: Use hyphens (`crate-name/`)
- **Package Names**: Use hyphens (`crate-name`)
- **Crate/Module Names**: Use underscores (`crate_name`)

## Testing Plan

After consolidation:

1. Build the entire workspace to ensure no compilation errors
2. Run tests for the context adapter crate
3. Run integration tests that rely on the context adapter

## Timeline

- Code verification: 1 day
- Implementation: 1 day
- Testing: 1 day
- Documentation updates: 1 day

## Responsible Team

The Context Management Team will be responsible for implementing this consolidation. 