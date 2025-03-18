// MCP Session module
pub mod manager;

use std::sync::Arc;
use crate::error::Result;
use tokio::sync::RwLock;

/// MCP Session interface
pub trait MCPSession: Send + Sync {
    /// Get session ID
    fn get_id(&self) -> &str;
    
    /// Check if session is active
    fn is_active(&self) -> bool;
}

// Re-export important types
pub use manager::MCPSessionManager; 