// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Message routing implementation
//!
//! This module provides the MessageRouter for directing MCP messages
//! to appropriate handlers based on message type.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::mcp::MessageType;
use crate::security::manager::SecurityManagerImpl;

// Placeholder trait - actual handler implementation would go in a separate module
pub trait MessageHandler: Send + Sync {
    // Handler methods would be defined here
}

/// MessageRouter is responsible for routing messages to the appropriate handler
/// based on message type and content.
#[derive(Clone)]
pub struct MessageRouter {
    /// Map of handlers for different message types
    handlers: Arc<RwLock<HashMap<MessageType, Vec<Arc<dyn MessageHandler>>>>>,
    /// Security manager for permission checking
    security: Arc<SecurityManagerImpl>,
}

impl MessageRouter {
    /// Create a new message router
    pub fn new(security: Arc<SecurityManagerImpl>) -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            security,
        }
    }

    // Additional routing methods would be implemented here as needed
}

