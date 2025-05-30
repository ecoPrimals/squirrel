# Web Crate Deprecation Steps

## Implementation Status: READY FOR ARCHIVAL

The migration from the ui-web crate to the unified ui-tauri-react implementation is now complete. All functionality has been successfully migrated, and the web crate is ready for archival.

## Completed Steps

As part of the ui-web crate deprecation and migration to the unified ui-tauri-react implementation, the following steps have been completed:

1. ✅ Updated root `Cargo.toml` to indicate ui-web is deprecated
2. ✅ Updated `plugins/Cargo.toml` to replace "web" feature with "tauri-ui" feature
3. ✅ Added deprecation comment to ui-tauri-react's Cargo.toml regarding web crate dependency
4. ✅ Added prominent deprecation warning to `crates/web/src/bin/web_server.rs`
5. ✅ Added crate-level deprecation notice to `crates/web/src/lib.rs`
6. ✅ Created `DEPRECATED.md` file in web crate root directory
7. ✅ Verified `README.md` in web crate has appropriate deprecation notice
8. ✅ Created archival script for Week 7+ (`scripts/archive_web_crate.sh`)
9. ✅ Added documentation about the deprecation process (`specs/ui/WEB_DEPRECATION_STEPS.md`)
10. ✅ Fixed duplicate lints section in root Cargo.toml
11. ✅ Verified all core functionality has been migrated to ui-tauri-react
12. ✅ Fully tested the web crate integration with the unified UI
13. ✅ Updated all documentation to reflect the new architecture

## Archival Process

The web crate is now ready for archival. The following steps will be performed to remove it:

1. ✅ Preparation completed
2. ⏩ Execute the archival script:
   ```
   ./scripts/archive_web_crate.sh archive
   ```
3. ⏩ Update the workspace configuration to remove the web crate
4. ⏩ Update the ui-tauri-react implementation to use archived code
5. ⏩ Run comprehensive tests to verify functionality

## Known Issues

During the deprecation process, the following known issues were identified and have been addressed:

1. ✅ **Build Dependency Issue**: The web crate dependency on `warning = "^0.1.1"` will be isolated in the archived code.
2. ✅ **Conditional Builds**: No longer required as the archival is being executed.

## Deprecation Timeline

| Phase | Week | Status | Actions |
|-------|------|--------|---------|
| Planning | 1 | ✅ Complete | Create migration documents |
| Deprecation | 2-3 | ✅ Complete | Add deprecation notices |
| Dual Support | 3-6 | ✅ Complete | Support both implementations |
| Removal | 7+ | ⏳ In Progress | Archive web crate |

## Final Archival Steps

Run the following steps in order to complete the archival process:

1. Verify no critical dependencies remain:
   ```
   ./scripts/migration/web_to_tauri_migration.sh check
   ```

2. Perform a final dry run:
   ```
   ./scripts/archive_web_crate.sh dryrun
   ```

3. Archive the web crate:
   ```
   ./scripts/archive_web_crate.sh archive
   ```

4. Update the root Cargo.toml to exclude the web crate:
   - Remove `crates/web` from members
   - Ensure `archive/web` is in exclude list

5. Run tests to verify everything still works:
   ```
   cargo test --workspace
   ```

## Post-Archival Tasks

After the web crate has been archived, the following tasks should be completed:

1. 📝 Update README.md to reflect the completed archival
2. 📝 Notify all development teams of the completed archival
3. 📝 Update build scripts and CI/CD pipeline
4. 📝 Create a final migration report

## Additional Resources

- [MIGRATION_PLAN_WEB_TO_TAURI.md](MIGRATION_PLAN_WEB_TO_TAURI.md): Detailed migration plan
- [WEB_CONSOLIDATION.md](WEB_CONSOLIDATION.md): Consolidation strategy
- [UI_STATUS_UPDATE.md](UI_STATUS_UPDATE.md): Overall UI consolidation status

---

Last Updated: 2024-08-03 