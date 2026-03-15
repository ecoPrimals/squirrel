<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Examples

Runnable examples demonstrating Squirrel's capabilities.

## JSON-RPC Client

```bash
cargo run --example rpc_client
```

Connects to Squirrel's Unix socket and sends sample JSON-RPC requests
(health check, list providers, query AI, announce capabilities).

## tarpc Client

```bash
cargo run --example tarpc_client_example --features tarpc-rpc
```

Demonstrates binary tarpc client connecting to Squirrel.

## Other Examples

| Example | Description |
|---------|-------------|
| `infant_discovery_demo` | Ecosystem discovery during primal startup |
| `unified_plugin_system_demo` | Plugin loading and management |
| `universal_ai_demo` | Vendor-agnostic AI provider routing |
| `production_security_demo` | Security and auth patterns |
| `zero_copy_demo` | Zero-copy serialization |
| `universal_system_demo` | System health and metrics |

Run any example with:

```bash
cargo run --example <name>
```

## Prerequisites

Squirrel server must be running for the client examples:

```bash
./target/release/squirrel server
```
