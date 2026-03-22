// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Message routing implementation
//!
//! This module provides the MessageRouter for directing MCP messages
//! to appropriate handlers based on message type.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::protocol::types::MessageType;
use crate::security::manager::SecurityManagerImpl;

/// Placeholder trait — actual handler implementation would go in a separate module
pub trait MessageHandler: Send + Sync {}

/// Handler map: message type -> ordered handler chain
type HandlerMap = HashMap<MessageType, Vec<Arc<dyn MessageHandler>>>;

/// Routes MCP messages to the appropriate handler based on message type.
#[derive(Clone)]
pub struct MessageRouter {
    #[expect(
        dead_code,
        reason = "Handler map reserved for future MCP dispatch wiring"
    )]
    handlers: Arc<RwLock<HandlerMap>>,
    #[expect(
        dead_code,
        reason = "Security manager reserved for per-message auth in routing"
    )]
    security: Arc<SecurityManagerImpl>,
}

impl MessageRouter {
    /// Create a new message router
    #[must_use]
    pub fn new(security: Arc<SecurityManagerImpl>) -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            security,
        }
    }
}
