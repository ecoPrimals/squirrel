// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! # Core Plugin Framework
//!
//! This module contains the core plugin framework components including
//! the plugin lifecycle management, plugin context, and base plugin traits.

pub mod context;
pub mod plugin;

// Re-export commonly used types
pub use context::{ContextData, PluginContext};
pub use plugin::{BasePlugin, PluginInfo, PluginStats, WasmPlugin};
