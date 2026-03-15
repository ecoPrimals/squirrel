<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Changelog

All notable changes to Squirrel will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Pre-alpha history is preserved in `CHANGELOG.pre-alpha.md`.

## [0.1.0-alpha] - 2026-03-15

First public alpha release. Squirrel is the AI Coordination Primal of the
ecoPrimals ecosystem — a sovereign MCP service for routing AI requests,
managing context, and coordinating multiple model providers.

### Highlights

- **3,749 tests** passing across 22 crates, 0 failures
- **Zero C dependencies** in default build (pure Rust)
- **Zero unsafe code** (`#![forbid(unsafe_code)]` on all crates)
- **scyBorg license** — AGPL-3.0-or-later + ORC + CC-BY-SA 4.0

### Architecture

- TRUE PRIMAL design: self-knowledge only, runtime capability discovery
- JSON-RPC 2.0 over Unix sockets (default IPC)
- tarpc binary protocol with automatic negotiation
- Transport hierarchy: Unix sockets → named pipes → TCP
- HTTP/WebSocket feature-gated OFF by default
- Vendor-agnostic AI: OpenAI, Anthropic, Gemini, local models (Ollama, llama.cpp, vLLM)

### Feature Gates

| Feature | Purpose | Default |
|---------|---------|---------|
| `delegated-jwt` | Capability-based JWT delegation | ON |
| `mesh` | Songbird federation | OFF |
| `http-api` | Axum/Tower HTTP stack | OFF |
| `gpu-detection` | ToadStool GPU detection | OFF |
| `system-metrics` | sysinfo C dependency | OFF |
| `local-jwt` | Local JWT (brings ring C dep) | OFF |
| `websocket` | WebSocket transport | OFF |

### Cleanup from pre-alpha

- Reduced unique dependencies from 314 to 272
- Eliminated HTTP stack from default build
- Feature-gated all cross-primal code (Songbird, ToadStool, BearDog, NestGate)
- Replaced deprecated crates (`serde_yaml` → `serde_yml`, `log` → `tracing`)
- Purged PII, large artifacts, and stale code from git history
- Fixed deadlock in ExperienceReplay (RwLock re-entrance)
- Fixed all MCPError Display formatting (missing `#[error]` attributes)
- Fixed squirrel-mcp-auth feature interaction (delegated-jwt vs local-jwt)
- Resolved all build warnings across workspace
- Archived 420+ stale docs, scripts, and showcase files
