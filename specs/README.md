# Squirrel Specifications

This directory contains specifications, design documents, and architectural plans for the Squirrel project.

## Directory Structure

The specifications are organized to match the crate structure of the project:

- **core/** - Core component specifications
  - **context/** - Context management specifications
  - **mcp/** - Machine Context Protocol specifications
  - **plugins/** - Plugin system specifications
  - **interfaces/** - Interface specifications
  - **patterns/** - Design pattern specifications

- **integration/** - Integration component specifications
  - **api-clients/** - API client specifications
  - **context-adapter/** - Context adapter specifications
  - **web/** - Web integration specifications
  - **mcp-pyo3-bindings/** - Python bindings specifications

- **services/** - Service component specifications
  - **app/** - Application service specifications
  - **commands/** - Command service specifications
  - **dashboard-core/** - Dashboard core specifications
  - **monitoring/** - Monitoring service specifications

- **tools/** - Tool specifications
  - **ai-tools/** - AI tools specifications
  - **cli/** - CLI tool specifications
  - **rule-system/** - Rule system specifications

- **ui/** - UI component specifications
  - **core/** - Core UI specifications
  - **implementation/** - UI implementation specifications
  - **testing/** - UI testing specifications

- **archived/** - Historical and archived specifications

## Key Documents

- [SPECS.md](SPECS.md) - Template and guidelines for writing specifications
- [SPECS_REVIEW.md](SPECS_REVIEW.md) - Process for reviewing specifications
- [TESTING.md](TESTING.md) - Testing guidelines and requirements
- [SECURITY.md](SECURITY.md) - Security requirements and best practices
- [MCP_INTEGRATION.md](MCP_INTEGRATION.md) - Machine Context Protocol integration guide
- [TEAM_RESPONSIBILITIES.md](TEAM_RESPONSIBILITIES.md) - Team ownership and responsibilities for specs

## Team Ownership

Each specification belongs to a specific team who is responsible for maintaining and updating it. For details on team responsibilities and collaboration areas, see [TEAM_RESPONSIBILITIES.md](TEAM_RESPONSIBILITIES.md).

## Adding New Specifications

When adding a new specification:

1. Place it in the appropriate directory based on the component it relates to
2. Follow the specification template in `SPECS.md`
3. Update any related specifications as needed
4. Add a reference to the specification in the relevant component README
5. Ensure the appropriate team is aware of their ownership

For more details on the specification process, see `SPECS.md`.
