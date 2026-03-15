<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Specifications

Architectural specifications and design documents for the Squirrel AI Coordination Primal.

## Directory Structure

```
specs/
├── active/         # Active specifications (MCP protocol, universal patterns, ecosystem)
├── current/        # Current status and deployment guide
└── development/    # Development standards (testing, security, codebase structure)
```

Pre-alpha specs (gRPC, RBAC, resilience) are preserved as fossil record in
`ecoPrimals/archive/squirrel-pre-alpha-fossil-mar15-2026/`.

## Architecture

- **IPC**: JSON-RPC 2.0 over Unix sockets (default), tarpc binary protocol (optional)
- **Transport**: Automatic fallback — Unix sockets -> Named pipes -> TCP
- **AI**: Vendor-agnostic, capability-based provider routing
- **Pattern**: TRUE PRIMAL (self-knowledge only, runtime discovery)

## Key Specs

| Spec | Location |
|------|----------|
| MCP Protocol | `active/mcp-protocol/MCP_SPECIFICATION.md` |
| Universal Patterns | `active/UNIVERSAL_PATTERNS_SPECIFICATION.md` |
| Ecosystem Integration | `active/UNIVERSAL_SQUIRREL_ECOSYSTEM_SPEC.md` |
| Socket Registry | `SOCKET_REGISTRY_SPEC.md` |
| Deployment | `current/DEPLOYMENT_GUIDE.md` |
| Testing | `development/TESTING.md` |
| Security | `development/SECURITY.md` |
