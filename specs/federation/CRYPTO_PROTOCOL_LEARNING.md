---
description: Reference guide for learning cryptographic protocols relevant to federation (Lightning, SGX attestation, Tendermint)
version: 0.1.0
last_updated: 2025-05-XX
---
# Cryptographic Protocol Learning Guide

> 💡 **Goal:** Equip developers and stewards with the conceptual and practical grounding needed to implement, audit, and extend the federation's security-critical components.

## 1. Foundational Cryptography
| Topic | Recommended Resource | Format | Why |
|-------|----------------------|--------|-----|
| Modern cryptography overview | *Serious Cryptography* – J. Aumasson | Book | Clear math + engineering focus |
| Public-Key primitives | Cryptography I (Stanford / Coursera) | Video | Interactive, good for refresh |
| Elliptic-Curve, Ed25519 | *Curve25519 and Friends* (Cloudflare blog series) | Article | Directly relevant to contract signatures |

## 2. Transport Security
| Protocol | Resource | Hands-On |
|----------|----------|----------|
| TLS 1.3 internals | *Illustrated TLS* (Ivov) | Medium article + Wireshark labs |
| WireGuard | *WireGuard Whitepaper* + *WireGuard for work & fun* | Setup tunnels between lab nodes |

## 3. Trusted Execution Environments (TEE)
| TEE | Resource | Lab Exercise |
|-----|----------|-------------|
| Intel SGX | *Intel SGX Explained* (Costan & Devadas) | Run "Hello SGX" in Fortanix EDP |
| AMD SEV-SNP | AMD SEV-SNP Developer Guide | Launch encrypted VM on Strandgate |
| WASM Sandboxing | *Inside Wasmtime* (Bytecode Alliance blog) | Build & run signed WASM module |

## 4. Ledger & Consensus
| Technology | Resource | Project Exercise |
|------------|----------|------------------|
| immudb (immutable DB) | immudb Docs + "Build tamper-evident apps" blog | Store and verify Merkle proofs |
| Tendermint Core | Tendermint Kademlia tutorial | Spin up 3-node testnet, query block hashes |
| Merkle Trees | *Merkle Trees: Concept & Setup* (A. Biryukov) | Implement simple Merkle verifier in Rust |

## 5. Payment Channels
| Stack | Resource | Lab |
|------|----------|-----|
| Bitcoin Lightning | *Mastering the Lightning Network* (O'Reilly, 2021) | Spin up 2 LND nodes on regtest, send 100 satoshi |
| BOLT specs | Official BOLT GitHub | Trace HTLC flow in logs |
| rust-lightning | API docs + example app | Integrate invoice generation into Accounting daemon |

## 6. Putting It Together
1. Write a Rust PoC that:  
   • signs a YAML contract with Ed25519,  
   • stores it in immudb,  
   • executes a Wasmtime sandbox,  
   • posts receipt,  
   • triggers Lightning payment.
2. Document each step in the federation wiki.

## 7. Advanced Reading
- *Efficient Zero-Knowledge Proof Systems* – Bootle et al. (for future private receipts)
- *Privacy-Preserving Federated Learning with SGX* – Hunt et al.
- *Token-Curated Registries* – Goldin & McCaughey (for future dataset reputation markets)

---
Happy learning — every receipt you understand deepens the federation's trustworthiness.

<version>0.1.0</version> 