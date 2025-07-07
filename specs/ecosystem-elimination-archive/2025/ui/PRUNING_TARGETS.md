# UI Components Ready for Pruning

This document tracks UI components that are candidates for removal as part of code cleanup and consolidation efforts.

## Terminal UI Components

### 1. `crates/ui-terminal`

**Status**: Ready for pruning

**Rationale**:
- Duplicates functionality now available in the Tauri-based UI
- Uses ratatui for terminal-based UI rendering
- Will be replaced by the more fully-featured Tauri+React implementation
- Core functionality should be preserved in the new implementation

**Migration Path**:
1. Ensure all features from ui-terminal are properly mapped to equivalents in ui-tauri-react
2. Document any unique terminal UI capabilities that might need preservation
3. Create tests to verify functionality is preserved in the new implementation
4. Once verified, mark for removal in a future release

**Dependencies**:
- Consumers: (List any components that depend on this)
- Required by: (List services or features that require this component)

## Next Steps

1. Focus development on the ui-tauri-react implementation
2. Ensure the web and desktop UI portions are fully functional
3. Create a migration guide for any users of the terminal UI 