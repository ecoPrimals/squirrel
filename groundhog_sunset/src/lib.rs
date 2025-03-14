#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![warn(clippy::undocumented_unsafe_blocks)]

//! Machine Context Protocol (MCP) implementation for the DataScienceBioLab project.
//! 
//! This crate provides the core functionality for component-based UI management,
//! event handling, and layout management.

/// Terminal user interface module providing components and utilities for building
/// interactive terminal applications.
pub mod ui;

/// Machine Context Protocol (MCP) module providing core functionality for
/// component management, event handling, and state management.
pub mod mcp;