# Rule Update Suggestions

This document tracks issues encountered during development and suggests corresponding rule updates or new rules.
Each entry should include:
- Issue encountered
- Related existing rule (if any)
- Error or problem description
- Proposed rule change or new rule
- Priority level
- Status

## Current Suggestions

### 1. PowerShell Command Separator Handling
- **Issue Date**: 2025-03-08
- **Related Rule**: `005-git-commit-automation.mdc`
- **Error**: PowerShell doesn't support `&&` command separator
  ```powershell
  git add . && git commit  # Fails in PowerShell
  ```
- **Proposed Change**: Update rule to include shell-specific command chaining:
  ```powershell
  # PowerShell
  git add .; git commit  # Use semicolon
  
  # Bash/Zsh
  git add . && git commit  # Use &&
  ```
- **Priority**: High
- **Status**: Pending

### 2. Automated Rule Update Tracking
- **Issue Date**: 2025-03-08
- **Related Rule**: None (New Rule)
- **Problem**: Need systematic way to track and apply rule updates
- **Proposed Rule**: `008-rule-update-automation.mdc`
  ```markdown
  # Rule Update Automation Standards
  
  ## Context
  - When encountering repeated issues
  - When rules need updates based on experience
  - When tracking technical debt in rules
  
  ## Requirements
  - Track issues in tempRules/ruleUpdates.md
  - Auto-suggest rule updates after X occurrences
  - Include error context and solutions
  - Track priority and status
  - Regular rule review process
  ```
- **Priority**: Medium
- **Status**: Proposed

### 3. Shell-Specific Command Standards
- **Issue Date**: 2025-03-08
- **Related Rule**: None (New Rule)
- **Problem**: Different shells require different command syntax
- **Proposed Rule**: `009-shell-compatibility.mdc`
  ```markdown
  # Shell Compatibility Standards
  
  ## Context
  - When writing shell commands
  - When dealing with cross-platform scripts
  - When handling shell-specific features
  
  ## Requirements
  - Document shell-specific command formats
  - Provide alternatives for common operations
  - Handle PowerShell vs Bash differences
  - Include cross-platform testing
  ```
- **Priority**: High
- **Status**: Proposed

## Template for New Suggestions

```markdown
### Title
- **Issue Date**: YYYY-MM-DD
- **Related Rule**: rule-name.mdc or None
- **Error/Problem**: Description
- **Proposed Change**: Details
- **Priority**: High/Medium/Low
- **Status**: Proposed/In Review/Approved/Implemented
```

## Status Definitions
- **Proposed**: Initial suggestion
- **In Review**: Being evaluated
- **Approved**: Accepted for implementation
- **Implemented**: Change applied to rules

## Priority Levels
- **High**: Blocks work or causes errors
- **Medium**: Improves workflow significantly
- **Low**: Nice to have improvements

<version>1.0</version> 