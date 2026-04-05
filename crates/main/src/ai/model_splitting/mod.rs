// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Model Splitting — RELOCATED to ToadStool & Songbird
//!
//! This module existed as backward-compatibility stubs during migration.
//! All model-splitting logic now lives in the proper primals:
//!
//! - **ToadStool** — layer distribution, VRAM calculation, GPU execution
//! - **Songbird** — cross-tower coordination, tensor routing
//!
//! Squirrel's role is AI *orchestration* (user intent, routing). It discovers
//! ToadStool and Songbird at runtime via capability-based IPC — it never
//! manages GPU layers or VRAM splits directly.
//!
//! The deprecated stub types have been removed. If downstream code still
//! references them, migrate to the ToadStool / Songbird APIs.
