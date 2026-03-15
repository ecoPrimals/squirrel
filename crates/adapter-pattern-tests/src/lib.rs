// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Adapter Pattern Implementation and Tests
//!
//! This crate demonstrates the adapter pattern in Rust with a command-based
//! architecture. Three main adapters are implemented:
//!
//! 1. Registry Adapter - Basic adapter for command registry operations
//! 2. MCP Adapter - Adapter with authentication and authorization
//! 3. Plugin Adapter - Adapter for plugin system integration

#![cfg_attr(not(test), forbid(unsafe_code))]
#![warn(missing_docs)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

mod auth;
mod commands;
mod integration;
mod types;

#[cfg(test)]
mod tests;

pub use auth::McpAdapter;
pub use commands::{CommandAdapter, CommandRegistry, RegistryAdapter};
pub use integration::{MockAdapter, PluginAdapter, test_polymorphic_adapter};
pub use types::{
    Auth, AuthUser, Command, CommandError, CommandLogEntry, CommandResult, TestCommand, UserRole,
};
