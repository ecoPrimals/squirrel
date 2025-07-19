//! # Squirrel Interfaces
//!
//! This crate contains shared interfaces that are used by multiple Squirrel components.
//! The primary purpose is to break circular dependencies between crates.

pub mod context;
pub mod plugins;
pub mod tracing;

/// Error types and utilities used across the codebase
pub mod error;
