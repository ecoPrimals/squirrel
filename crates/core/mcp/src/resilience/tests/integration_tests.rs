// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Resilience Integration Tests
//!
//! This module imports all resilience integration test modules.
//! The original 694-line file has been refactored into focused modules.

// Import all resilience integration test modules from the integration directory
#[path = "integration/mod.rs"]
pub mod integration_modules;

// Re-export the modules for backward compatibility
pub use integration_modules::*; 