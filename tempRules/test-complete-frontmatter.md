---
description: "Comprehensive frontmatter test rule"
globs: ["**/*.{js,ts,jsx,tsx}"]
priority: 2
weight: 0.8
activation_threshold: 0.5
context_requirements:
  languages: ["typescript", "javascript"]
  frameworks: ["react", "next"]
  environments: ["browser", "node"]
load_conditions:
  always_load: false
  lazy_load: true
  preload: false
crossRefs:
  - "001-rule-generator.mdc"
  - "?002-rule-organization.mdc"
dependencies:
  required:
    - "core-rule.mdc"
    - "base-rule.mdc"
  optional:
    - "?optional-rule.mdc"
  versioned:
    "versioned-rule.mdc": ">=1.0.0"
metadata:
  version: "1.2.0"
  category: "development"
  tags: ["style", "formatting", "best-practices"]
  maintainers: ["@team-core", "@team-dev"]
  last_updated: "2024-03-15"
  changelog:
    - version: "1.2.0"
      changes: ["Added new context support", "Updated dependencies"]
    - version: "1.1.0"
      changes: ["Added framework detection"]
    - version: "1.0.0"
      changes: ["Initial release"]
  deprecation:
    is_deprecated: false
    replacement_rule: null
    sunset_date: null
---

# Complete Frontmatter Test Rule

## Context
- When testing comprehensive frontmatter features
- When verifying frontmatter parsing
- When validating rule loading behavior

## Requirements
- Validate all frontmatter fields
- Test priority and weight handling
- Verify dependency resolution
- Check context-aware loading
- Confirm metadata processing

## Examples
<example>
// Good example demonstrating rule application
function validateCode() {
  // Implementation following rule guidelines
}
</example>

<example type="invalid">
// Bad example showing rule violation
function badCode() {
  // Implementation violating rule guidelines
}
</example>

<version>1.2.0</version> 