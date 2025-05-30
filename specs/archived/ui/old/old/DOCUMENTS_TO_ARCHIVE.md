# Documents to Archive

The following documents should be moved to `specs/ui/old` as they refer to the previous UI architecture or are superseded by newer versions:

1. `implementation-progress-tauri-react.md` - Superseded by newer IMPLEMENTATION_PROGRESS_TAURI_REACT.md

2. `separate-terminal-uis.md` - No longer relevant after UI consolidation

3. Any remaining documentation that refers solely to the standalone `ui-web` implementation

Additionally, after full migration is confirmed, the entire `web` crate should be archived to `crates/archived` similar to what was done with the previous `ui-terminal` version.

## Next Steps

After archiving these documents:

1. Update all cross-references to point to the current documentation
2. Review remaining documents for any outdated information
3. Update README.md to reflect the current architecture

## Timeline

These archival tasks should be completed before August 15, 2024, as specified in the MIGRATION_PLAN.md document. 