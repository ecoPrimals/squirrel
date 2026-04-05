// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! API module for AI routing and request types
//!
//! **LEGACY MODULE** - Partially evolved to modern architecture
//!
//! # Current State (v1.6.0)
//!
//! After v1.6.0 HTTP debt cleanup:
//! - ❌ HTTP API endpoints **DELETED** (health, metrics, ecosystem, server, service_mesh, management)
//! - ✅ AI routing **KEPT** (used by tarpc RPC server)
//! - ✅ Types **KEPT** (shared request/response types)
//!
//! # Architecture
//!
//! The remaining modules support tarpc-based AI routing:
//! - `ai/`: AI provider routing and selection (used by `rpc/tarpc_server`)
//! - `types`: Shared request/response types
//!
//! # Usage
//!
//! This module is primarily used internally by `crates/main/src/rpc/tarpc_server.rs`
//! for AI request routing over tarpc RPC.

// AI routing module (used by tarpc RPC server)
pub(crate) mod ai;

// Re-export for tarpc_server
pub use ai::AiRouter;
