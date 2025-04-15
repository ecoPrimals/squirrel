//! # Squirrel Interfaces
//! 
//! This crate contains shared interfaces that are used by multiple Squirrel components.
//! The primary purpose is to break circular dependencies between crates.

pub mod plugins; pub mod context; pub mod tracing;
