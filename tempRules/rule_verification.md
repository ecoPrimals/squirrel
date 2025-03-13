# Rule Verification Checklist

## Required Structure
- [ ] YAML frontmatter with:
  - description (ACTION TRIGGER OUTCOME format)
  - globs (appropriate file patterns)
  - crossRefs (if any dependencies exist)
- [ ] Title (H1)
- [ ] Context section
- [ ] Requirements section
- [ ] Examples section (with valid and invalid examples)
- [ ] Version tag

## Cross-Reference Map

### Core Standards (0XX)
- 000-cursor-rules
  - Depends on: none
- 001-rule-generator
  - Depends on: 000-cursor-rules, 002-rule-organization, 004-mdc-rule-location, 010-rule-cross-referencing
- 002-rule-organization
  - Depends on: 001-rule-generator
- 003-code-style-guide
  - Depends on: none
- 004-mdc-rule-location
  - Depends on: 000-cursor-rules, 001-rule-generator
- 005-git-commit-automation
  - Depends on: 007-worktree-management, 009-shell-compatibility, 008-rule-update-automation
- 006-cli-standards
  - Depends on: none
- 007-worktree-management
  - Depends on: 005-git-commit-automation
- 008-rule-update-automation
  - Depends on: 000-cursor-rules, 001-rule-generator
- 009-shell-compatibility
  - Depends on: 005-git-commit-automation, 007-worktree-management
- 010-rule-cross-referencing
  - Depends on: 001-rule-generator, 002-rule-organization, 008-rule-update-automation
- 011-team-communication
  - Depends on: 007-worktree-management, 005-git-commit-automation

### Documentation Standards (4XX)
- 400-md-docs
  - Depends on: none

### Rust Standards (1XXX)
- 1001-rust-safety through 1022-rust-test-organization
  - Cross-reference each other based on functionality
  - MCP-related rules (1016-1021) should cross-reference each other
  - Testing rules (1007, 1022) should cross-reference each other

## Processing Steps
1. [ ] Move all .mdc files from .cursor/rules to tempRules/ as .md
2. [ ] Verify structure of each file
3. [ ] Update cross-references
4. [ ] Verify frontmatter format
5. [ ] Convert back to .mdc
6. [ ] Move verified files to .cursor/rules/

## Notes
- Keep original files until verification is complete
- Process core rules first (0XX series)
- Then process language-specific rules
- Update cross-references in batches by category
- Verify circular dependencies 