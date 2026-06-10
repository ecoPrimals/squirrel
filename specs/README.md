<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Specifications

Architectural specifications and design documents for the Squirrel AI Coordination Primal.

## Directory Structure

```
specs/
├── historical/     # All specs preserved as fossil record
│   ├── mcp-protocol/  # Gen2 MCP protocol docs
│   ├── AI_DEVELOPMENT_GUIDE.md
│   ├── UNIVERSAL_PATTERNS_SPECIFICATION.md
│   └── ...
└── SOCKET_REGISTRY_SPEC.md  # Active socket discovery standard
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
| Socket Registry | `SOCKET_REGISTRY_SPEC.md` | active |
| Universal Patterns | `historical/UNIVERSAL_PATTERNS_SPECIFICATION.md` | fossil record |
| AI Development | `historical/AI_DEVELOPMENT_GUIDE.md` | fossil record |
| MCP Protocol (gen2) | `historical/mcp-protocol/` | fossil record |
| Deployment (gen2) | `historical/DEPLOYMENT_GUIDE.md` | fossil record |
| Testing (gen2) | `historical/TESTING.md` | fossil record |
| Security (gen2) | `historical/SECURITY.md` | fossil record |

See `README.md` and `CURRENT_STATUS.md` at repo root for the authoritative current state.
