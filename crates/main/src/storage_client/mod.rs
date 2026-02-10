// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Universal Storage Client Module
//!
//! This module provides modular, capability-based storage integration.

pub mod client;
pub mod providers;
pub mod types;

#[cfg(test)]
mod providers_tests;
#[cfg(test)]
mod types_tests;
// Removed ai_metadata - was over-engineered early implementation

pub use client::UniversalStorageClient;
pub use providers::*;
pub use types::*;
