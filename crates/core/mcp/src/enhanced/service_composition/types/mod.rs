// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Service Composition Types
//!
//! This module organizes all type definitions for the service composition system
//! into focused, maintainable submodules.

pub mod service;
pub mod dependency;
pub mod composition;
pub mod orchestration;
pub mod config;

// Re-export all types for backward compatibility
pub use service::*;
pub use dependency::*;
pub use composition::*;
pub use orchestration::*;
pub use config::*; 