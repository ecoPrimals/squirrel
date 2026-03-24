// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration module for AI tools
//!
//! This module provides configuration management for AI service integrations.

pub mod core;
pub mod defaults;
pub mod model_registry;

// Re-export key types for backward compatibility
pub use core::{AIToolsConfig, ProviderConfig};
pub use defaults::DefaultEndpoints;
pub use model_registry::ModelRegistry;
