---
description: Implementation roadmap for federation alpha (Q2 2025)
version: 0.1.0
last_updated: 2025-05-XX
---
# Federation Roadmap — Q2 2025

## Milestone Overview
| Phase | Target Date | Deliverables | Success Criteria |
|-------|-------------|--------------|------------------|
| 1. Contract Schema & CLI | Week 2 Jun | `data_contract` crate + `squirrel contract mint` CLI; sample YAML | Contracts can be parsed, signed, and validated locally |
| 2. Provenance Bus v0.1 | Week 3 Jun | gRPC service, immudb ledger, `RECEIPT_INSERT` event | Task receipts persist & can be queried by dataset ID |
| 3. Secure Executor MVP | Week 5 Jun | WASM sandbox with signature check; receipts include WASM hash | Demo task runs on steward node; receipt verifies hash |
| 4. Accounting Daemon v0.1 | Week 1 Jul | Base-fee payout logic; Lightning testnet transfer | Stewards receive testnet satoshis for each task |
| 5. Multi-Node Alpha | Week 3 Jul | Northgate + 1 external node executing daily test tasks |  >90% task success; <5% ledger discrepancy |
| 6. Quality Oracle Prototype | Week 4 Jul | Genomics data-quality plugin emitting `QUALITY_SCORE` events | Accuracy correlation >=0.8 vs manual audit |

## Detailed Work Breakdown
### Phase 1 – Contract Schema
- Define YAML & JSON-LD variants
- Ed25519 signature format (using `ring`)
- Unit tests for parsing & signing

### Phase 2 – Provenance Bus
- immudb docker-compose stack on Northgate
- gRPC proto definitions (`receipt.proto`)
- Rust server with `tonic`
- CLI: `squirrel ledger query --dataset-id ...`

### Phase 3 – Secure Executor
- Integrate `wasmtime` runtime
- Authorisation middleware: verify module signature matches steward policy
- Receipt structure: `{wasm_sha256, exit_code, cpu_ms, ram_mb}`

### Phase 4 – Accounting Daemon
- Subscribe to ledger events
- Configurable payout coefficients (TOML)
- Lightning testnet invoice settlement (via `LND` gRPC)
- CSV export fallback

### Phase 5 – Alpha Federation
- Onboarding script: WireGuard + node cert
- Dashboard tile in Tauri UI showing live receipts

### Phase 6 – Quality Oracle
- Implement FASTQ validator plugin
- Emit `QUALITY_SCORE` to ledger
- Update Accounting daemon to apply quality multiplier

## Risks & Mitigations
| Risk | Mitigation |
|------|------------|
| Ledger performance bottleneck | Batch writes; periodic compaction |
| WASM sandbox escape | Use wasmtime `WasiCtxBuilder` in confined mode + seccomp |
| Lightning node instability | Provide CSV payout fallback |

<version>0.1.0</version> 