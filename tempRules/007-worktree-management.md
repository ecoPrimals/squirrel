---
description: ENFORCE technical standards for git worktree management, synchronization, and testing in parallel development, with specific focus on IDE state management
globs: ["**/*.rs", "**/*.toml", ".gitignore", ".git/**/*"]
crossRefs:
  - 002-rule-organization.mdc
  - 005-git-commit-automation.mdc
  - 009-shell-compatibility.mdc
alwaysApply: true
---
# Git Worktree Technical Standards

## IDE State Management

### Context
- When switching between parallel worktrees
- When managing multiple IDE instances
- When handling workspace synchronization
- When recovering from inconsistent states

### Requirements

#### Pre-Switch Procedure
1. Save all changes in current worktree
2. Close all IDE windows associated with current worktree
3. Verify no pending changes:
   ```bash
   git status
   ```
4. Document current IDE state if needed

#### Switching Procedure
1. Execute worktree switch from terminal only:
   ```bash
   cd groundhog-worktrees/[target-worktree]
   ```
2. Verify switch success:
   ```bash
   git worktree list
   pwd
   ```

#### Post-Switch Validation
1. File System Recognition
   - Verify correct worktree path
   - Check file visibility
   - Validate directory structure

2. Development Environment
   - Confirm Cargo.toml recognition
   - Verify dependency resolution
   - Check feature flag status

3. IDE Features
   - Test code completion
   - Verify git integration
   - Check build tools
   - Validate debugger

#### Recovery Procedures

##### For Minor Inconsistencies
1. Save any unsaved work
2. Close IDE window
3. Clear IDE workspace state:
   ```bash
   # Location varies by OS
   rm -rf .idea/workspace.xml  # Example for JetBrains
   ```
4. Reopen IDE in correct worktree

##### For Major Inconsistencies
1. Close all IDE windows
2. Verify worktree status:
   ```bash
   git worktree list
   ```
3. Clean IDE caches:
   ```bash
   # Example paths - adjust for your IDE
   rm -rf ~/.config/Cursor/Cache/*
   rm -rf ~/.config/Cursor/Workspaces/*
   ```
4. Restart IDE in correct worktree
5. Verify environment setup

### Best Practices
1. One IDE window per worktree
2. No branch switching within IDE
3. Regular workspace state validation
4. Document worktree-specific IDE settings
5. Maintain separate IDE configurations per worktree

### Technical Metadata
- Category: Development Environment
- Priority: High
- Dependencies:
  - Git 2.25+
  - Cursor IDE
  - Cargo
- Validation Requirements:
  - File system checks
  - IDE feature tests
  - Git integration verification

<version>1.0.0</version> 