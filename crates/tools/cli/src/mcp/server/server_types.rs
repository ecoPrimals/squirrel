// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! MCP server type definitions.

use crate::mcp::protocol::{MCPMessage, MCPResult};

/// MCP command handler function
pub type MCPCommandHandler = Box<dyn Fn(MCPMessage) -> MCPResult<MCPMessage> + Send + Sync>;
