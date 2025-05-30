# Pruning Strategy for `ui-terminal` Crate

## Overview

This document outlines the precise steps for pruning the legacy `ui-terminal` crate once the Tauri-based UI has reached feature parity. Since this is a home project, we can perform a hard prune as soon as we're satisfied with the Tauri UI implementation.

## Prerequisites

Before pruning, verify that the following conditions have been met:

1. All features in the checklist from `UI_ARCHITECTURE_PLAN.md` are implemented in the Tauri UI
2. All tests for the Tauri UI are passing
3. The Tauri UI has been successfully tested on target environments

## Pruning Steps

### 1. Documentation Updates

```bash
# Create a backup of documentation
mkdir -p archive/ui-terminal-docs
cp crates/ui-terminal/README.md archive/ui-terminal-docs/
cp crates/ui-terminal/USAGE.md archive/ui-terminal-docs/ # If exists
```

Update relevant documentation to point to the new UI:

- Update main README.md to remove ui-terminal references
- Update any documentation that references the terminal UI

### 2. Dependency Removal

Identify and update any crates that depend on `ui-terminal`:

```bash
# Find dependencies (manual check)
grep -r "ui-terminal" --include="Cargo.toml" .
```

For each dependency found, remove or replace the dependency:

```toml
# Example Cargo.toml update
[dependencies]
# Remove or comment out this line
# ui-terminal = { path = "../ui-terminal" }

# Add this if needed
ui-tauri-react = { path = "../ui-tauri-react" }
```

### 3. Update Workspace Configuration

Edit the root `Cargo.toml` to remove `ui-terminal` from the workspace:

```toml
# Before
members = [
    "crates/ui-terminal",
    "crates/ui-tauri-react",
    # other crates...
]

# After
members = [
    "crates/ui-tauri-react",
    # other crates...
]
```

### 4. Script Updates

Update any scripts that reference or launch the terminal UI:

```bash
# Find scripts that reference ui-terminal
grep -r "ui-terminal" --include="*.sh" --include="*.ps1" .
```

For each script found, update to use the new UI:

```bash
# Example update in launch-ui.sh
# Replace:
# cargo run -p ui-terminal --bin squirrel-dashboard
# With:
cd crates/ui-tauri-react && npm run tauri dev
```

### 5. Build Configuration Updates

Update any build configurations that include ui-terminal:

```bash
# Find build configurations
grep -r "ui-terminal" --include="build.rs" --include="*.yml" --include="*.json" .
```

For each configuration, remove or update references to ui-terminal.

### 6. Migration of Essential Code

If any unique functionality exists in ui-terminal that hasn't been migrated:

```bash
# Create a temporary directory for code migration
mkdir -p temp/ui-terminal-migration

# Copy unique code for reference
cp crates/ui-terminal/src/widgets/unique_widget.rs temp/ui-terminal-migration/
```

Ensure the functionality is implemented in the Tauri UI before proceeding.

### 7. Archiving

Archive the ui-terminal code for reference:

```bash
# Create archive directory if it doesn't exist
mkdir -p archive/crates

# Move the entire ui-terminal directory to archive
mv crates/ui-terminal archive/crates/
```

### 8. Cleanup

Remove any remaining references to ui-terminal:

```bash
# Find any remaining references
grep -r "ui-terminal" . --exclude-dir=archive
```

Update or remove these references as needed.

### 9. Testing After Pruning

Run the test suite to ensure everything still works:

```bash
# Run tests
cargo test
cd crates/ui-tauri-react && npm test
```

### 10. Update CI/CD Configuration

Update any CI/CD configurations to remove ui-terminal build and test steps:

```bash
# Find CI configurations
find .github -type f -name "*.yml" | xargs grep -l "ui-terminal"
```

Update each identified file to remove ui-terminal steps.

## Fallback Plan

If issues are discovered after pruning:

1. The archived code in `archive/crates/ui-terminal` can be restored
2. Revert the workspace and dependency changes
3. Rebuild and verify functionality

## Post-Pruning Tasks

1. Update the UI architecture documentation to reflect the removal
2. Mark the pruning as complete in project tracking
3. Notify any team members of the change

## Conclusion

This pruning strategy provides a systematic approach to removing the legacy `ui-terminal` crate once the Tauri UI reaches feature parity. By following these steps, we can ensure a clean removal without losing functionality or breaking dependencies. 