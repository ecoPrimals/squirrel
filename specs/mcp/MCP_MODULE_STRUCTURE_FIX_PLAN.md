# MCP Module Structure Fix Plan

## Overview and Identified Issues

After analyzing the build errors in the MCP codebase, we've identified the following critical issues that need to be addressed:

1. **Multiple Definition Errors (E0252)**
   - Several types are defined multiple times, including:
     - `DefaultCryptoProvider` 
     - `SecurityManagerImpl`
   - This suggests duplicate implementations or conflicting module structures

2. **Unresolved Import Errors (E0432)**
   - Imports for several RBAC-related types are unresolved:
     - `self::rbac::Permission`
     - `self::rbac::Action`
     - `self::rbac::Resource`
   - This indicates either missing modules or incorrect export/visibility settings

3. **Private Import Errors (E0603)**
   - Struct imports like `Action` are marked as private
   - This suggests visibility issues in the module hierarchy

## Root Causes

1. **Module Organization Issues**
   - Inconsistent module structure, particularly in the security and RBAC modules
   - Possible circular dependencies between modules
   - Incorrect visibility settings for types that should be public

2. **Migration from `SecurityError::RBAC` to `SecurityError::RBACError`**
   - The migration appears complete in `permission_validation.rs` and `role_inheritance.rs`
   - However, there may be lingering references in other files or generated code

## Action Plan

### Phase 1: Module Structure Analysis (Estimated time: 30 minutes)
1. Map the current module structure of the security-related components
2. Identify all occurrences of duplicated type definitions
3. Trace import paths to locate circular dependencies
4. Document the intended visibility for each type/module

### Phase 2: Structure Fixes (Estimated time: 1-2 hours)
1. **Consolidate Type Definitions**
   - Ensure each type is defined exactly once in the appropriate module
   - Remove duplicate definitions
   - Address `DefaultCryptoProvider` and `SecurityManagerImpl` duplication

2. **Fix Module Hierarchy**
   - Restructure modules to eliminate circular dependencies
   - Ensure proper re-exports in `mod.rs` files
   - Adjust visibility modifiers (`pub`, `pub(crate)`, etc.) as needed

3. **Resolve RBAC-specific Issues**
   - Ensure `Permission`, `Action`, and `Resource` types are correctly exported
   - Fix any remaining issues with `SecurityError::RBAC` vs `SecurityError::RBACError`

### Phase 3: Testing and Verification (Estimated time: 30-45 minutes)
1. Run `cargo check` to verify that compilation errors are resolved
2. Run `cargo build` to ensure the codebase builds successfully
3. If applicable, run unit tests to ensure functionality is preserved
4. Address any new issues that may arise

## Implementation Strategy

We'll take an incremental approach:
1. First fix the security module structure
2. Then address the RBAC-specific issues
3. Finally resolve any remaining import/visibility issues

This approach allows us to verify progress at each step and prevents introducing new issues while fixing existing ones.

## Expected Outcome

1. A clean compilation with no E0252, E0432, or E0603 errors
2. A coherent module structure that properly exposes the required types
3. Consistent use of the correct error types throughout the codebase

## Team: DataScienceBioLab

*Document created on: [Current Date]* 