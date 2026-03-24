// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! PrimalPulse - AI-Powered Ecosystem Intelligence
//!
//! **LEGACY MODULE** - being evolved to capability-based architecture
//!
//! NOTE: This module is not actively maintained. Future rebuild will use capability_ai
//! and Universal Transport abstractions. HTTP API was removed in favor of socket-based
//! communication. See: crates/universal-patterns/src/transport.rs

pub mod neural_graph;
mod schemas;

#[cfg(test)]
mod tests;
