# Building and Testing the Squirrel Commands Project

This document outlines the build and test process for the Squirrel Commands project.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable channel)
- PowerShell (for Windows) or Bash (for Unix-like systems)

## Quick Build

For a complete build and test, run:

```powershell
# On Windows
./build_test.ps1
```

## Build Process Details

The build process:

1. **Prepares the Environment**:
   - Stops any processes that might be locking files
   - Cleans the target directory to ensure a fresh build

2. **Core Components**:
   - Builds the `squirrel-commands` crate first
   - Runs tests for the `squirrel-commands` crate

3. **Examples**:
   - Builds the main example (`phase1_functional_demo`)
   - Runs the example to verify functionality

4. **Full Workspace**:
   - Builds all other crates in the workspace
   - Runs tests for all crates

5. **Verification**:
   - Final verification of `squirrel-commands` tests

## Manual Building

If you prefer manual building steps:

```bash
# Build the commands crate
cargo build -p squirrel-commands

# Run tests for the commands crate
cargo test -p squirrel-commands

# Build the Phase 1 demo example
cargo build --example phase1_functional_demo

# Run the Phase 1 demo example
cargo run --example phase1_functional_demo
```

## Handling Common Issues

### File Locking Issues

On Windows, you may encounter file locking issues when building examples. The `build_test.ps1` script handles these issues by:

1. Detecting and stopping processes that might be locking files
2. Cleaning build directories before starting a fresh build

If you manually encounter a file locking error like `LINK : fatal error LNK1104: cannot open file`, try:

```powershell
# Clean the target directory
cargo clean

# Kill any processes that might be locking the files
Get-Process | Where-Object { $_.Name -like "*cargo*" -or $_.Name -like "*rustc*" } | Stop-Process -Force
```

### Test Failures

If you encounter test failures, try running specific tests with:

```bash
# Run a specific test with verbose output
cargo test -p squirrel-commands test_name -- --nocapture
```

## Implementation Status

The current implementation status is tracked in `specs/commands/IMPLEMENTATION_STATUS.md`.

- Command System: 100% implemented
- Phase 1 Enhancements: 100% implemented (Transaction, Journaling, Resource Monitoring, Observability) 