//! Squirrel Application library
//!
//! This crate provides the application-level components for the Squirrel platform.
//! It includes state management, UI components, and business logic.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![doc(html_root_url = "https://docs.rs/squirrel-app")]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::module_name_repetitions)]
#![warn(clippy::todo)]
#![cfg_attr(test, allow(clippy::unwrap_used))]

pub use crate::core::Core;
pub use crate::adapter::AppAdapter;

/// The current application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Core application functionality
pub mod core;

/// Application adapter for interfacing with the system
pub mod adapter;

/// Application context functionality
pub mod context;

/// Application commands
pub mod command;

/// Application error handling
pub mod error;

/// Application event system
pub mod event;

/// Application event handling
pub mod events;

/// Application metrics
pub mod metrics;

/// Application monitoring
pub mod monitoring;

/// Application plugin system
pub mod plugin;

/// Re-exports
#[doc = "Common types for convenience"]
pub mod prelude {
    pub use crate::core::Core;
    pub use crate::adapter::AppAdapter;
    pub use crate::core::AppConfig;
    pub use crate::plugin::{Plugin, PluginManager, PluginLoader};
}

/// Module containing application tests
#[cfg(test)]
pub mod tests;

/// Public modules
pub mod config;

/// Application state
pub mod state;

/// User interface components
pub mod ui;

/// Domain models
pub mod models;

/// API client for MCP 
pub mod client;

/// Re-export core error handling
pub use squirrel_core::error::{Result, SquirrelError};

// Re-export commands crate
pub use commands as commands_crate;
