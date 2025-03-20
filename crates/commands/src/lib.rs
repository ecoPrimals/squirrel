//! Command system for Squirrel
//!
//! This crate provides functionality for command registration, validation,
//! and execution within the Squirrel system.

use thiserror::Error;

/// Built-in commands
pub mod builtin;

/// Command execution hooks
pub mod hooks;

/// Command lifecycle management
pub mod lifecycle;

/// Resource management for commands
pub mod resources;

/// Command validation
pub mod validation;

/// Command registry
mod registry;
pub use registry::{Command, CommandRegistry, CommandResult};

/// Command errors
#[derive(Debug, Error)]
pub enum CommandError {
    /// Error during command registration
    #[error("Registration error: {0}")]
    RegistrationError(String),
    
    /// Error during command execution
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    /// Error during command validation
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Error during command resource management
    #[error("Resource error: {0}")]
    ResourceError(String),
}

/// Command factory for creating command registries
mod factory;
pub use factory::{CommandRegistryFactory, create_command_registry};

/// Re-export common types from the core crate
pub use squirrel_core::error::Result;

#[cfg(test)]
mod tests; 