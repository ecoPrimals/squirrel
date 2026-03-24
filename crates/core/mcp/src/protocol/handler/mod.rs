// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! MCP protocol handler module
//!
//! Provides team workflow management, message routing, and related types.

mod message_router;
mod processor;
mod router;
mod team_types;
mod workflow_manager;

// Re-export for backward compatibility
pub use message_router::{MessageHandler, MessageRouter};
pub use processor::*;
pub use router::*;
pub use workflow_manager::TeamWorkflowManager;
