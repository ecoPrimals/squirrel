#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![warn(clippy::undocumented_unsafe_blocks)]

//! Core functionality for the Squirrel project.
//! 
//! This crate provides the core infrastructure for:
//! - Context management and state handling
//! - Command execution and lifecycle management
//! - Resource management and validation
//! - Event handling and synchronization

pub mod context;
pub mod commands;

pub use context::*;
pub use commands::*; 