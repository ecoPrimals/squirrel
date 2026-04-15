// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Types for the MCP CLI [`super::MCPClient`](crate::mcp::MCPClient).

use crate::mcp::protocol::{MCPMessage, MCPResult};

/// Callback for notification handlers
pub type NotificationCallback = Box<dyn Fn(&str, &MCPMessage) -> MCPResult<()> + Send + Sync>;
