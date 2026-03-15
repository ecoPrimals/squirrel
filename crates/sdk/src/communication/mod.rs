// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Communication module for handling events, commands, and MCP protocol

// Event handling
pub mod events;
pub use events::{Event, EventBus, EventListener, SimpleEventListener};

// Command handling
pub mod commands;
pub use commands::{
    CommandContext, CommandDefinition, CommandExample, CommandHandler, CommandRegistry,
    CommandResult, SimpleCommandHandler,
};

// MCP protocol
pub mod mcp;
pub use mcp::{
    ConnectionState, McpCapabilities, McpClient, McpMessage, McpPrompt, McpResource, McpTool,
};
