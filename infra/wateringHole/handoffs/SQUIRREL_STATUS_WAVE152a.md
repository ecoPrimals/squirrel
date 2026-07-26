# Squirrel Status Handoff — Wave 152a

**Date**: Jul 26, 2026 | **Wave**: 152a | **From**: squirrel team on eastGate
**To**: overwatch + upstream primal teams

## Current State

| Metric | Value |
|--------|-------|
| Tests | 7,132 passing / 0 failures (16 crates, `--all-features`) |
| Clippy | 0 warnings (`pedantic + nursery + cargo`) |
| Unsafe | 0 blocks (`unsafe_code = "forbid"`) |
| Files >800L (prod) | 0 |
| Dead code attrs | 20 remaining (all Phase 2 placeholders with `reason`) |
| TODO/FIXME | 0 |
| Mocks in prod | 0 (all `#[cfg(test)]`) |
| Hardcoded hosts | 0 in prod (all via `universal-constants`) |

## Completed Since Wave 151b

- **Wave 152a**: Deep debt sweep — 11 dead_code items deleted, SDK deps aligned to workspace, pre-existing test race fixed, 3 Clippy fixes
- **Wave 151b**: BTSP client handshake for bearDog strict mode (4-step HMAC-SHA256)
- **Wave 150u**: CredentialStore integration via bearDog `secrets.*` JSON-RPC

## Active IPC Integrations

| Upstream | Method | Status |
|----------|--------|--------|
| bearDog | `secrets.store/retrieve/list/delete` | DONE — `SecurityProviderSecretStore` |
| bearDog | BTSP `ClientHello` handshake | DONE — `btsp_client.rs` |
| songBird | `http.request` delegation | WIRED — capability endpoint discovery |
| any | `capabilities.list` / `primal.announce` | WIRED — JSON-RPC handlers |

## Remaining Upstream Blockers

1. **`send_to_primal` endpoint resolution**: Squirrel can `connect_transport()` to any discovered endpoint but needs a runtime registry of primal socket paths. Currently depends on env vars (`SECURITY_ENDPOINT`, `BEARDOG_ENDPOINT`).
2. **bearDog auth/crypto integration**: `crypto.sign`, `auth.verify_token` methods not yet exercised end-to-end (bearDog transplant to eastGate pending).
3. **petalTongue visualization delegation**: Squirrel's visualization layer thinned; ready to delegate rendering to petalTongue once its IPC surface is published.

## Gaps for Upstream Review

- bearDog: Confirm `secrets.*` JSON-RPC method signatures match production (squirrel matches the handoff spec)
- songBird: Publish `http.request` capability endpoint for Squirrel HTTP delegation
- overwatch: Validate BTSP handshake interop between squirrel (client) and bearDog (server)
