<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Specifications

Architectural specifications and design documents for the Squirrel AI Coordination Primal.

## Directory Structure

```
specs/
├── current/        # Deployment guide and production status
├── active/         # Active specifications (MCP protocol, universal patterns)
├── development/    # Development standards (testing, security, codebase)
└── archive/        # Historical specs
```

## Architecture Summary

- **IPC**: JSON-RPC 2.0 over Unix sockets (default), tarpc binary protocol (optional)
- **Transport**: Automatic fallback — Unix sockets -> Named pipes -> TCP
- **AI**: Vendor-agnostic, capability-based provider routing
- **Pattern**: TRUE PRIMAL (self-knowledge only, runtime discovery)

## Key Specs

| Spec | Location |
|------|----------|
| MCP Protocol | `active/mcp-protocol/` |
| Universal Patterns | `active/UNIVERSAL_PATTERNS_SPECIFICATION.md` |
| Deployment | `current/DEPLOYMENT_GUIDE.md` |
| Testing | `development/TESTING.md` |
| Security | `development/SECURITY.md` |
