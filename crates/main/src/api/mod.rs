// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! API module for AI routing and request types
//!
//! AI routing and shared request/response types.
//!
//! HTTP API endpoints were removed in the IPC-first migration.
//! AI routing is used by both JSON-RPC and tarpc servers.
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
