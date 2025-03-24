//! Context adapter for Squirrel
//!
//! This crate provides adapters for interfacing with the context subsystem,
//! including context management, persistence, and synchronization.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::needless_raw_string_hashes)]
#![cfg_attr(test, allow(clippy::unwrap_used))]

/// Context adapter implementation
pub mod adapter;

/// Re-export the main adapter module
pub use adapter::*;

/// Re-export common interfaces
pub use squirrel_interfaces::context::{
    ContextAdapterPlugin,
    AdapterMetadata,
    ContextPlugin,
    ContextTransformation,
};

/// Tests for the context adapter
#[cfg(test)]
pub mod tests; 