+++
title = "squirrel Validation Summary"
description = "AI inference routing, context management, capability discovery, signal composition. 7,089+ tests, 38 IPC methods, 90% coverage."
date = 2026-05-20

[taxonomies]
primals = ["squirrel"]
springs = []
+++

## Status

- **Gate**: CLEAR (stadial readiness confirmed May 17, 2026)
- **Phase**: 3 (BTSP Phase 3 AEAD encrypted framing)
- **Edition**: 2024 (Rust 1.94+)
- **Tests**: 7,089 passing across 22 workspace crates
- **Source**: ~1,001 `.rs` files, ~326k lines
- **Clippy**: 0 warnings (`pedantic` + `nursery` + `cargo`, `-D warnings`)
- **deny.toml**: ring, openssl, reqwest, native-tls, aws-lc-sys all banned; pure Rust enforced
- **Coverage**: 90.1% region / 89.6% line (cargo-llvm-cov)
- **Binary**: 3.5 MB static-pie musl, stripped, BLAKE3 checksummed, zero host paths

## Capabilities

| Capability | Description |
|-----------|-------------|
| `inference` | Multi-provider AI inference routing (complete, embed, models) |
| `context` | Session context creation, update, and summarization |
| `discovery` | Capability-based peer discovery (zero hardcoded names) |
| `signal` | Neural API composition collapse (signal.plan) |
| `tool` | Plugin tool execution and listing |
| `health` | Standard health triad (liveness, readiness, check) |
| `btsp` | Phase 3 cipher negotiation + encrypted framing |
| `graph` | Dependency graph parsing and validation (primalSpring BYOB) |
| `lifecycle` | biomeOS lifecycle registration + heartbeat |

## Methods (38 — registered in capability_registry.toml)

- `inference.complete`, `inference.embed`, `inference.models`, `inference.register_provider`, `inference.unregister_provider`
- `ai.query`, `ai.list_providers`, `ai.complete`, `ai.chat`
- `signal.plan`
- `capabilities.list`, `capabilities.announce`, `capabilities.discover`, `capability.announce`, `capability.discover`, `capability.list`, `primal.announce`, `primal.capabilities`
- `identity.get`
- `context.create`, `context.update`, `context.summarize`
- `system.metrics`, `system.health`, `system.status`, `system.ping`
- `health.check`, `health.liveness`, `health.readiness`
- `discovery.peers`, `discovery.list`
- `tool.execute`, `tool.list`
- `btsp.negotiate`
- `lifecycle.register`, `lifecycle.status`
- `graph.parse`, `graph.validate`

## Composition Role

Squirrel is the **intelligence router** for all compositions requiring AI inference:
- Meta-tier member (biomeOS + squirrel + petalTongue)
- Provides inference routing to any composition needing LLM/embedding access
- Delegates compute to neuralSpring providers, routes via capability discovery
- Context management for multi-turn conversations across compositions

## Downstream Pairing

- esotericWebb (inference consumer — web UI)
- projectFOUNDATION (inference consumer — code generation)
- neuralSpring (inference provider — model hosting)
- primalSpring (graph validation, coordination)
- wetSpring (sovereign pipeline — inference for Barrick clone)
- NestGate (model weight storage)

## Degradation

When squirrel is down: AI inference unavailable, context operations fail.
Other primals continue operating — squirrel is intelligence routing, not a gate.
Discovery, health, lifecycle registrations degrade gracefully (standalone mode).
