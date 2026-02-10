// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! PrimalPulse - AI-Powered Ecosystem Intelligence
//!
//! **LEGACY MODULE** - being evolved to capability-based architecture
//!
//! NOTE: This module is not actively maintained. Future rebuild will use capability_ai
//! and Universal Transport abstractions. HTTP API was removed in favor of socket-based
//! communication. See: crates/universal-patterns/src/transport.rs

// Legacy modules REMOVED - used deleted HTTP API (api::ai)
// pub(crate) mod handlers; // DELETED
// mod tools;                // DELETED
// pub use tools::register_primal_pulse_tools; // DELETED

// Remaining modules (may need updates)
pub mod neural_graph;
mod schemas;

#[cfg(test)]
mod tests;
