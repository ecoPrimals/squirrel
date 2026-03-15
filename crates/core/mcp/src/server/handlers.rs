// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Server Handler Traits
//!
//! Trait definitions for command and connection handlers.

use crate::error::Result;
use crate::message::Message;
use super::connection::ClientConnection;

/// Handler for processing MCP commands
pub trait CommandHandler: Send + Sync + std::fmt::Debug {
    /// Handle a command message
    fn handle_command<'a>(
        &'a self, 
        command: &'a Message
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<Message>>> + Send + 'a>>;
    
    /// Get the command types this handler can process
    fn supported_commands(&self) -> Vec<String>;

    /// Clone the handler into a new box
    fn clone_box(&self) -> Box<dyn CommandHandler>;
}

impl Clone for Box<dyn CommandHandler> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Connection handler for managing client connections
pub trait ConnectionHandler: Send + Sync {
    /// Handle a new client connection
    fn handle_connection<'a>(
        &'a self,
        client: ClientConnection
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
    
    /// Handle client disconnection
    fn handle_disconnection<'a>(
        &'a self,
        client_id: &'a str
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
    
    /// Clone the handler into a new box
    fn clone_box(&self) -> Box<dyn ConnectionHandler>;
}

impl Clone for Box<dyn ConnectionHandler> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
