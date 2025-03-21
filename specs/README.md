---
description: Guide for using the Squirrel specification documentation
version: 1.0.0
last_updated: 2024-03-21
status: active
---

# Squirrel Specifications

## Overview

This directory contains the specifications for the Squirrel project. After our recent refactoring effort, we're in the process of updating and aligning these specifications with the current codebase structure and future development plans.

## Directory Structure

- `specs/SPECS.md`: Main specifications overview document
- `specs/SPECS_REVIEW.md`: Tracking document for the specifications review process
- `specs/app/`: Application specifications
- `specs/commands/`: Command system specifications
- `specs/context/`: Context management specifications
- `specs/integration/`: Integration specifications
- `specs/mcp/`: MCP protocol specifications
- `specs/monitoring/`: Monitoring system specifications
- `specs/plugins/`: Plugin system specifications (post-MVP)
- `specs/MVP/`: Core MVP requirements
- `specs/teams/`: Team organization documentation
- `specs/patterns/`: Standard design patterns documentation
- `specs/archived/`: Archived specifications

## How to Use This Documentation

### For New Team Members

1. Start with `SPECS.md` for a high-level overview
2. Review `specs/teams/WORKTEAMS.md` to understand team organization
3. Review `specs/teams/AGENTS.md` to understand agent roles
4. Read the specifications for your specific area of focus

### For Implementation Work

1. Identify the relevant crate for your task
2. Review the corresponding specifications in the matching `specs/` directory
3. Check the `SPECS_REVIEW.md` document for update status
4. Follow the patterns documented in `specs/patterns/`

### For Architecture Changes

1. Review the current specifications in the relevant directory
2. Create or update specification documents
3. Update the `SPECS_REVIEW.md` tracking document
4. Coordinate with affected teams

## Specifications Status

We're currently reviewing all specifications to ensure they align with our refactored codebase. See `SPECS_REVIEW.md` for:

- Current review status of each specs directory
- Mapping of specs directories to crates
- Gaps and action items

## Standard Patterns

All new code should follow the standard patterns documented in `specs/patterns/`:

- Dependency Injection Pattern (`specs/patterns/dependency-injection.md`)
- Error Handling Pattern
- Async Programming Pattern
- Testing Pattern
- Command Pattern
- Adapter Pattern

## Contributing to Specifications

When updating specifications:

1. Follow the Markdown Documentation Standards in our coding guidelines
2. Include proper frontmatter with description, version, and status
3. Keep language clear and concise
4. Include examples where appropriate
5. Document interfaces, not implementations
6. Focus on the "what" and "why", not just the "how"
7. Update version history when making changes

## Integration with Crates

Each specification directory generally corresponds to one or more crates:

| Specs Directory | Corresponding Crates |
|-----------------|----------------------|
| specs/app/ | crates/app/, crates/core/ |
| specs/commands/ | crates/commands/ |
| specs/context/ | crates/context/, crates/context-adapter/ |
| specs/integration/ | (Spans multiple crates) |
| specs/mcp/ | crates/mcp/ |
| specs/monitoring/ | crates/monitoring/ |
| specs/plugins/ | (Post-MVP, no crate yet) |
| specs/MVP/ | (Spans multiple crates) |

## Specification Review Status

See `SPECS_REVIEW.md` for detailed tracking of the specification review process.

## Questions and Feedback

If you have questions about specifications or want to provide feedback:

1. For team-specific questions: Contact your team lead
2. For cross-team issues: Use the TEAMCHAT.md format
3. For specification improvement suggestions: Update the relevant section in `SPECS_REVIEW.md` 