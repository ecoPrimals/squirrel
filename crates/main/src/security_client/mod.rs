// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Universal Security Client Module
//!
//! This module provides modular, capability-based security integration.

pub mod client;
pub mod providers;
pub mod types;

#[cfg(test)]
mod client_tests;
#[cfg(test)]
mod providers_tests;
#[cfg(test)]
mod types_tests;
// Removed ai_metadata - was over-engineered early implementation

pub use client::UniversalSecurityClient;
pub use providers::*;
pub use types::*;
