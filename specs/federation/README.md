---
description: High-level overview of federation, data-stewardship, and value-settlement architecture
version: 0.1.0
last_updated: 2025-05-XX
---
# Federation Specifications

## Purpose
These documents define the architectural, protocol, and governance primitives required to extend the Squirrel platform beyond a single cluster into a **compute-to-data, receipt-metered federation**.  They complement _MCP core_ specs by covering:

1. Data-contract schema & stewardship duties
2. Provenance & receipt ledger
3. Automatic credit settlement
4. Node onboarding & attestation (TEE / WASM)
5. Governance and formula-update processes

## Document Map
| Doc | Status | Description |
| ---- | ------ | ----------- |
| `ROADMAP_Q2_2025.md` | Draft | Phased implementation plan for federation alpha | 
| `DATA_CONTRACT_SPEC.md` | TODO | YAML/JSON-LD structure for dataset stewardship contracts |
| `ACCOUNTING_DAEMON.md` | TODO | Micro-payout formula & implementation notes |
| `LEDGER_ARCHITECTURE.md` | TODO | immudb/Tendermint design & APIs |
| `CRYPTO_PROTOCOL_LEARNING.md` | Draft | Curated resources to bootstrap Lightning, SGX attestation, and cryptographic ledger know-how |

## Scope Boundaries
* **In scope:** Control-plane gRPC, task scheduling heuristics, receipt format, reward formulas.
* **Out of scope:** Foundation-model training specifics, per-domain ML algorithms, UI mock-ups (covered in `ui/`).

## Versioning Strategy
Follows semantic versioning aligned with Squirrel global spec versions.  Major bumps reflect breaking changes to wire formats or economic formulas.

<version>0.1.0</version> 