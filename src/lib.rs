//! Squirrel Universal AI Primal
//!
//! This library provides a universal AI primal implementation that is agnostic
//! to specific primal types and uses dynamic service discovery.

pub mod api;
pub mod ecosystem;
pub mod error;
pub mod monitoring;
pub mod primal_provider;
pub mod shutdown;
pub mod universal;
pub mod universal_api;
pub mod universal_primal_provider;

// Re-export commonly used types
pub use universal::*;
pub use universal_api::*;
pub use universal_primal_provider::*;
pub use primal_provider::*;
pub use error::*;

// Re-export core types from dependencies
pub use squirrel_core::*;
pub use squirrel_config::*; 