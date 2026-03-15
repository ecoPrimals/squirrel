// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Task management system with JSON-RPC transport.
//!
//! This module provides task management capabilities for the MCP system
//! using JSON-RPC over Unix sockets instead of gRPC.

pub mod client;
pub mod conversion;
pub mod json_rpc_types;
pub mod manager;
pub mod server;
pub mod types;

#[cfg(test)]
mod tests;

pub use client::MCPTaskClient;
pub use manager::TaskManager;
pub use types::{AgentType, Task, TaskPriority, TaskStatus};
