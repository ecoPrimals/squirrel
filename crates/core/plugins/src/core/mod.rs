// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Core plugins module
//!
//! This module contains core plugins that are bundled with the application.

// Re-export core plugin types
pub use super::plugin::{Plugin, PluginMetadata};

pub mod hello_world;

// Re-export core plugins
pub use hello_world::HelloWorldPlugin;
