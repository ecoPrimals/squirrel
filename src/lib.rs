//! Squirrel MCP Core
//! 
//! Machine Context Protocol implementation focused on core functionality.
//! 
//! This is the pure MCP authority after the ecosystem realignment:
//! - Web integration → Songbird
//! - Compute/storage → ToadStool/NestGate  
//! - Complex monitoring → Distributed across ecosystem

// Core MCP modules that should remain
pub mod error;
pub mod protocol;
pub mod session;
pub mod transport;
pub mod songbird;
pub mod enhanced;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Re-export core types for convenience
pub use error::types::{MCPError, Result};

// Integration module for basic testing
pub mod integration {
    //! Simple MCP integration utilities for testing and basic usage
    
    use crate::error::Result;
    
    /// Simple MCP integration for basic testing
    #[derive(Debug, Clone)]
    pub struct SimpleMCPIntegration {
        initialized: bool,
    }
    
    impl SimpleMCPIntegration {
        /// Create a new simple MCP integration
        pub fn new() -> Self {
            Self { initialized: false }
        }
        
        /// Initialize the integration
        pub async fn initialize(&mut self) -> Result<()> {
            self.initialized = true;
            Ok(())
        }
        
        /// Check if initialized
        pub fn is_initialized(&self) -> bool {
            self.initialized
        }
    }
    
    impl Default for SimpleMCPIntegration {
        fn default() -> Self {
            Self::new()
        }
    }
}

// TODO: Re-enable these modules as they're cleaned up and made MCP-core specific:
// pub mod session;     // Only if core MCP session management remains
// pub mod transport;   // Only if core MCP transport remains  
// pub mod tool;        // Only if core MCP tool integration remains
// pub mod types;       // Core MCP types only 