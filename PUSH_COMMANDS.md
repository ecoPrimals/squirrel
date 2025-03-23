# Git Commands for Pushing Context Plugins

The following commands should be used to push the context plugins implementation to the main repository:

## PowerShell Commands

```powershell
# Stage all changes
git add crates/plugins/src/context/
git add crates/plugins/src/context_adapter/
git add crates/plugins/examples/context_plugins.rs
git add crates/plugins/docs/CONTEXT_MIGRATION_REPORT.md
git add specs/plugins/context-plugins.md
git add CONTEXT_PLUGINS_PUSH_SUMMARY.md

# Commit with proper message format
git commit -m "feat(plugins): complete context and context-adapter plugins

- Implemented Context Plugin for data transformation
- Implemented Context Adapter Plugin for format conversion
- Added comprehensive tests and examples
- Created detailed documentation and specifications
- Fixed borrowing issues and optimized performance

State: InProgress -> Done
Components: plugins/context, plugins/context_adapter
Reviewers: @plugin-team @context-team"

# Push to appropriate branch
git push origin feature/plugins/context-integration
```

## Bash/Zsh Commands

```bash
# Stage all changes
git add crates/plugins/src/context/ \
      crates/plugins/src/context_adapter/ \
      crates/plugins/examples/context_plugins.rs \
      crates/plugins/docs/CONTEXT_MIGRATION_REPORT.md \
      specs/plugins/context-plugins.md \
      CONTEXT_PLUGINS_PUSH_SUMMARY.md

# Commit with proper message format
git commit -m "feat(plugins): complete context and context-adapter plugins

- Implemented Context Plugin for data transformation
- Implemented Context Adapter Plugin for format conversion
- Added comprehensive tests and examples
- Created detailed documentation and specifications
- Fixed borrowing issues and optimized performance

State: InProgress -> Done
Components: plugins/context, plugins/context_adapter
Reviewers: @plugin-team @context-team"

# Push to appropriate branch
git push origin feature/plugins/context-integration
```

## Post-Push Tasks

1. Open a pull request from `feature/plugins/context-integration` to the main branch
2. Fill in the PR template with information from `CONTEXT_PLUGINS_PUSH_SUMMARY.md`
3. Tag the appropriate reviewers from the plugin team and context team
4. Attach the context-plugins specification and migration report for reference 