# Squirrel IPC Architecture — AAR + Upstream Demand

**Date**: Jul 18, 2026 | **Wave**: 150d | **From**: squirrel team on eastGate
**To**: overwatch + upstream primal teams (songBird, bearDog, petalTongue, nestGate)
**Priority**: P1 — root blocker for composition wiring

---

## What Squirrel Has Done (Waves 144a–150b)

Squirrel is **converged and stable**. All standing debt swept:

| Metric | Value |
|--------|-------|
| Tests | 7,108 passing, 0 failures |
| Clippy | Clean (`pedantic + nursery + cargo`, `-D warnings`) |
| Unsafe | 0 (`forbid` workspace-wide) |
| `.unwrap()` in prod | 0 |
| Files >800L (prod) | 0 |
| Debt markers | 0 |
| Mocks in prod | 0 |
| `dead_code` attrs | 37 remaining — all documented Phase 2 stubs |
| Windows cross-compile | Green |
| Transport | Phase 2 COMPLETE — `TransportEndpoint` ecosystem-wide |
| Visualization | Thinned — petalTongue domain code deleted (~1,800L) |
| Learning | Cleaned — dead fields removed, wiring fixed |
| Capability naming | `"visualization"` canonical (code + deploy graphs agree) |
| SecretStore | **INTEGRATED** — `SecurityProvider` delegates to bearDog `secrets.*` JSON-RPC over IPC (Wave 150u) |
| Deprecated code | `core/mcp/src/constants.rs` deleted (303L, zero consumers) |
| Hardcoding | Evolved to `universal_constants::network::*` throughout |
| `null` params | Fixed — JSON-RPC accepts `null` params per spec (esotericWebb P1) |

**Squirrel has nothing left to do locally.** All remaining evolution requires
upstream primal wiring.

---

## The Root Blocker: `send_to_primal` Is Unwired

```rust
// crates/main/src/universal_primal_ecosystem/mod.rs:400-416
pub async fn send_to_primal(
    &self,
    primal_id: &str,
    _request: PrimalRequest,
) -> UniversalResult<PrimalResponse> {
    Err(PrimalError::OperationFailed(format!(
        "Inter-primal IPC to '{primal_id}' not yet wired"
    )))
}
```

This is the **single function** that blocks all inter-primal communication.
Three production clients call it and get errors:

| Client | File | What it needs |
|--------|------|---------------|
| `compute_client` | `compute_client/client.rs:184` | Send compute requests to toadStool/barraCuda |
| `security_client` | `security_client/client.rs:182` | Send auth/crypto requests to bearDog |
| `storage_client` | `storage_client/client.rs:172` | Send storage requests to nestGate |

**All three follow the same pattern**: discover a provider via capability
resolution → build a `PrimalRequest` → call `send_to_primal` → deserialize
`PrimalResponse`. The transport layer (`TransportEndpoint` + `connect_transport`)
is ready. The missing piece is the dispatch logic inside `send_to_primal`.

---

## What Squirrel Needs From Upstream

### 1. songBird: Capability → Endpoint Resolution (P1)

Squirrel discovers capabilities via `find_services_by_capability()` which
returns `DiscoveredService { endpoint, capabilities, ... }`. But the
`endpoint` field contains a Unix socket path or TCP address that needs
to be resolvable at runtime.

**Need**: songBird's `capabilities.resolve` or equivalent JSON-RPC method
that takes a capability ID and returns a `TransportEndpoint` (UDS path or
TCP host:port). This is the missing link between capability discovery and
transport connection.

**squirrel has**: `TransportEndpoint::parse()`, `connect_transport()`,
`connect_transport_with_timeout()` — all ready to use once we have an
endpoint to connect to.

### 2. bearDog: JSON-RPC Method Signatures (P1)

`security_client` needs to send requests like `auth.verify_token`,
`crypto.sign`, `crypto.verify` to bearDog. bearDog confirmed crypto
JSON-RPC sigs exist (Wave 150a).

**Need**: Documented JSON-RPC method names and param/result schemas so
squirrel can build `PrimalRequest` payloads correctly. Current code
uses generic `capability_id` + `serde_json::Value` — needs grounding
to bearDog's actual API surface.

### 3. nestGate: CAS Read API (P1)

`storage_client` needs to store/retrieve content via nestGate's
content-addressed storage.

**Need**: JSON-RPC method signatures for `content.store`, `content.get`,
`content.list` (or whatever nestGate exposes). footPrint also needs
`PROJECTS_PATH` CAS wiring through this same interface.

### 4. petalTongue: Visualization Render API (P2)

Squirrel thinned its visualization system to emit JSON payloads. The
rendering contract is `visualization.render.*` per deployment graphs.

**Need**: petalTongue's `visualization.render.*` JSON-RPC method
signatures. Squirrel will call these via capability discovery when
petalTongue is available. Not blocking — squirrel has `JsonRenderer`
and `TerminalRenderer` for debug/CLI output.

### 5. Ecosystem: `primal-transport` Crate (P2)

`TransportEndpoint`, `connect_transport()`, `TransportStream`, and
`TransportListener` are currently defined in each primal's
`universal-patterns` crate. Publishing a shared `primal-transport`
crate would:

- Eliminate 14 copies of the same transport abstraction
- Give every primal a single dependency for inter-primal communication
- Enable `send_to_primal` to use a shared dispatch protocol

---

## Proposed `send_to_primal` Implementation

Once upstream provides endpoint resolution + method signatures:

```rust
pub async fn send_to_primal(
    &self,
    primal_id: &str,
    request: PrimalRequest,
) -> UniversalResult<PrimalResponse> {
    // 1. Resolve endpoint via capability discovery
    let endpoint = self.resolve_primal_endpoint(primal_id).await?;

    // 2. Connect via TransportEndpoint
    let mut stream = connect_transport_with_timeout(
        &endpoint,
        Duration::from_secs(10),
    ).await?;

    // 3. Send JSON-RPC request
    let jsonrpc = serde_json::json!({
        "jsonrpc": "2.0",
        "id": uuid::Uuid::new_v4().to_string(),
        "method": request.method,
        "params": request.params,
    });
    // ... frame, send, read response, deserialize ...
}
```

**squirrel can implement this in a single session** once we have:
- A working `capabilities.resolve` → `TransportEndpoint` from songBird
- Method signatures from bearDog, nestGate, petalTongue

---

## Plugin System Phase 2 (24 `dead_code` attrs — secondary blocker)

The plugin system (`core/plugins`) has 24 `dead_code` attributes across
`types.rs`, `default_manager.rs`, `web/adapter.rs`, `web/example.rs`,
`discovery.rs`, `plugin.rs`, `plugin_v2.rs`. These are all Phase 2
stubs awaiting the plugin execution sandbox.

**Not blocking IPC** — plugins are a separate composition axis. But
the plugin manager's web API returns 501 (Phase 2) and the plugin
type filtering constants are unwired. This is a lower-priority
evolution that can proceed independently.

---

## Summary for Overwatch

| What | Status | Blocker |
|------|--------|---------|
| squirrel transport layer | DONE | — |
| squirrel capability discovery | DONE | — |
| squirrel `send_to_primal` dispatch | BLOCKED | songBird endpoint resolution |
| squirrel `security_client` (secrets) | **DONE** (Wave 150u) | bearDog `secrets.*` JSON-RPC integrated |
| squirrel `security_client` (auth/crypto) | BLOCKED | bearDog `auth.*`/`crypto.*` method signatures |
| squirrel `storage_client` | BLOCKED | nestGate CAS method signatures |
| squirrel `compute_client` | BLOCKED | toadStool/barraCuda method signatures |
| squirrel visualization IPC | BLOCKED (P2) | petalTongue render method signatures |
| squirrel plugin system Phase 2 | BLOCKED (P2) | Sandbox design decision |

**Squirrel is ready.** The transport, discovery, and client infrastructure
are all in place. We need method signatures and endpoint resolution from
upstream primals to wire the last mile.

---

*AAR from squirrel team. Converged at Wave 150b, CredentialStore integrated
Wave 150u. 7,122 tests, 0 debt, 0 unsafe. Ready for IPC composition wiring.
Awaiting upstream.*
