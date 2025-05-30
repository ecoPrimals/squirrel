# Squirrel Team Responsibilities for Specs

This document outlines the responsibilities of different teams for maintaining specifications in the Squirrel project. It helps teams understand which parts of the specification directory they own and are responsible for updating.

## Team Ownership

### Core Team

**Primary Ownership:**
- `specs/core/` - Core component specifications
  - `specs/core/context/` - Context management specifications
  - `specs/core/mcp/` - Machine Context Protocol specifications
  - `specs/core/plugins/` - Plugin system specifications
  - `specs/core/interfaces/` - Interface specifications
  - `specs/core/patterns/` - Design pattern specifications
  - `specs/core/modules/` - Specialized modules specifications

**Collaboration Areas:**
- `specs/integration/core-mcp-integration.md`
- `specs/integration/plugin-core-integration.md`
- `specs/integration/context-management-integration.md`

### Integration Team

**Primary Ownership:**
- `specs/integration/` - Integration component specifications
  - `specs/integration/api-clients/` - API client specifications
  - `specs/integration/context-adapter/` - Context adapter specifications
  - `specs/integration/web/` - Web integration specifications
  - `specs/integration/mcp-pyo3-bindings/` - Python bindings specifications

**Collaboration Areas:**
- `specs/core/mcp/adapters/` - MCP adapter specifications
- `specs/services/monitoring/integration.md` - Monitoring service integration

### Services Team

**Primary Ownership:**
- `specs/services/` - Service component specifications
  - `specs/services/app/` - Application service specifications
  - `specs/services/commands/` - Command service specifications
  - `specs/services/dashboard-core/` - Dashboard core specifications
  - `specs/services/monitoring/` - Monitoring service specifications

**Collaboration Areas:**
- `specs/integration/dashboard-monitoring-integration.md`
- `specs/integration/mcp-monitoring-integration.md`

### Tools Team

**Primary Ownership:**
- `specs/tools/` - Tool specifications
  - `specs/tools/ai-tools/` - AI tools specifications
  - `specs/tools/cli/` - CLI tool specifications
  - `specs/tools/rule-system/` - Rule system specifications

**Collaboration Areas:**
- `specs/integration/ai-agent-integration.md`
- `specs/core/context/rule-system.md`

### UI Team

**Primary Ownership:**
- `specs/ui/` - UI component specifications
  - `specs/ui/core/` - Core UI specifications
  - `specs/ui/implementation/` - UI implementation specifications
  - `specs/ui/testing/` - UI testing specifications

**Collaboration Areas:**
- `specs/integration/ui-mcp-integration.md`
- `specs/services/monitoring/monitoring-dashboard-integration.md`

## Cross-Team Specification Files

Some specification files require input and maintenance from multiple teams:

| Specification | Primary Owner | Collaborators |
|---------------|--------------|--------------|
| `specs/SECURITY.md` | Core Team | All Teams |
| `specs/TESTING.md` | Integration Team | All Teams |
| `specs/MCP_INTEGRATION.md` | Core Team | Integration Team, UI Team |
| `specs/integration/resilience-framework.md` | Integration Team | Core Team, Services Team |
| `specs/integration/thread_safety_pattern_guide.md` | Core Team | All Teams |

## Process for Updating Specifications

1. **Primary Owners**:
   - Can directly update specifications in their ownership areas
   - Should notify collaborator teams of significant changes
   - Must update specifications before each sprint to reflect current status
   - Should use the SPECS_REVIEW_CHECKLIST.md to ensure quality

2. **Collaborators**:
   - Should propose changes to areas they don't own through pull requests
   - Should include primary owners as reviewers
   - Should actively participate in cross-team specification reviews

3. **Cross-Team Specifications**:
   - Require approval from all relevant teams before merging changes
   - Should be discussed in cross-team meetings before major changes
   - Must be reviewed by all teams before the start of each sprint

## Adding New Specifications

When adding a new specification:

1. Place it in the appropriate directory based on the component it relates to
2. Follow the specification template in `SPECS.md`
3. Update the relevant team's README if necessary
4. If the specification affects multiple teams, include a section describing responsibilities

## Version Control and Reviews

To ensure specifications remain accurate and up-to-date:

1. All specification changes should be reviewed by at least one team member
2. Major changes should have cross-team reviews
3. Specifications should be updated whenever related implementation changes
4. Regular specification audits should be performed quarterly

## Sprint Preparation Requirements

Before each sprint:

1. Each team must review and update their owned specifications
2. Implementation status percentages must be updated to reflect actual progress
3. Any new components or features must have corresponding specifications
4. Cross-team specifications must be reviewed and updated collaboratively
5. The SPECS.md main file must be kept in sync with individual specification files

By following these guidelines, we can ensure specifications remain current, accurate, and useful to all teams. 