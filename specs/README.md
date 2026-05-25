<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Specifications

Architectural specifications and design documents for the Squirrel AI Coordination Primal.

## Directory Structure

```
specs/
├── active/         # Active specifications (universal patterns)
├── development/    # Development standards (AI guide)
├── historical/     # Gen2 specs preserved as fossil record (MCP, deployment, testing)
└── SOCKET_REGISTRY_SPEC.md
```

Pre-alpha specs (gRPC, RBAC, resilience) are preserved in
`ecoPrimals/archive/squirrel-pre-alpha-fossil-mar15-2026/`.

Gen2-era specs (MCP protocol, WebSocket transport, deployment guide) were moved to
`historical/` as part of Wave 49 ecosystem tightening (May 2026).

## Architecture

- **IPC**: JSON-RPC 2.0 over Unix sockets (default), tarpc binary protocol (optional)
- **Transport**: Automatic fallback — Unix sockets → Named pipes → TCP
- **AI**: Vendor-agnostic, capability-based provider routing
- **Pattern**: TRUE PRIMAL (self-knowledge only, runtime discovery)

## Key Specs

| Spec | Location | Status |
|------|----------|--------|
| Universal Patterns | `active/UNIVERSAL_PATTERNS_SPECIFICATION.md` | active |
| Socket Registry | `SOCKET_REGISTRY_SPEC.md` | active |
| AI Development | `development/AI_DEVELOPMENT_GUIDE.md` | active |
| MCP Protocol (gen2) | `historical/mcp-protocol/` | archived |
| Deployment (gen2) | `historical/DEPLOYMENT_GUIDE.md` | archived |
| Testing (gen2) | `historical/TESTING.md` | archived |
| Security (gen2) | `historical/SECURITY.md` | archived |

See `README.md` and `CURRENT_STATUS.md` at repo root for the authoritative current state.
