# Interfaces Crate Migration Plan

## Overview

The interfaces crate is the foundation of the system, providing core interfaces and contracts with no dependencies on other crates. This makes it the ideal first candidate for migration.

## Steps

1. **Copy Source Files**
   - Copy all files from `crates/interfaces/src` to `new-structure/core/interfaces/src`
   - Ensure all module files are included

2. **Create Cargo.toml**
   - Create a new Cargo.toml using workspace inheritance
   - Keep direct dependencies as needed
   - Update any path references

3. **Fix Module Structure**
   - Ensure no duplicate module files (mod.rs vs named.rs)
   - Fix any visibility issues
   - Ensure exports are correct

4. **Build and Test**
   - Run `cargo check` on just this crate
   - Fix any issues that arise
   - Run tests if available

## Dependencies

The interfaces crate should have:
- No dependencies on other Squirrel crates
- Minimal external dependencies

## Potential Issues

- Module organization may need to be updated
- Some interface definitions might implicitly depend on other crates
- Feature flags might need adjustment

## Post-Migration

After successful migration:
1. Update the migration tracker
2. Prepare for the next crate (core)
3. Document any issues encountered 