// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Sync Tests
//!
//! This module imports all sync test modules.
//! The original 627-line file has been refactored into focused modules.

// Import all sync test modules from the sync_modules directory
#[path = "sync_modules/mod.rs"]
pub mod sync_modules;

// Re-export the modules for backward compatibility
pub use sync_modules::*;
