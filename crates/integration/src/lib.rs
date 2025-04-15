//! # Integration Framework
//! 
//! This crate provides integration between Squirrel and various external services.
//!
//! It handles API integrations, data synchronization, and protocol adapters
//! for communicating with external systems.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::undocumented_unsafe_blocks)]

// Main modules
pub mod ai_agent;
pub mod context_mcp;
pub mod mcp_ai_tools;
pub mod error;

// Re-export key types for the V2 traits
pub use ai_agent::types::{AIClientV2, AIClientCallbacks, AIClientWrapper};
pub use context_mcp::types::{ContextManagerV2, ContextManagerCallbacks, ContextManagerWrapper, ContextManager, SquirrelContext};
pub use mcp_ai_tools::adapter::ToolHandler;

// Skip these tests for now
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
} 