// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Task Server Implementation
//!
//! This module provides a comprehensive gRPC server implementation for the TaskService.
//! The original 1,402-line server.rs file has been refactored into focused modules:
//!
//! - `types`: Trait definitions and type aliases
//! - `watchers`: Task watcher management functionality  
//! - `service`: Core TaskServiceImpl struct and initialization
//! - `handlers`: gRPC service method implementations
//! - `commands`: Production command registry and execution logic
//! - `mock`: Mock implementations for testing only
//!
//! ## Production Usage
//!
//! The module exports `ProductionCommandRegistry` for production use. This provides:
//! - Thread-safe command execution with proper error handling
//! - Command statistics and performance tracking
//! - Production-safe mutex handling
//! - Built-in commands like `help`
//!
//! Use `get_command_registry()` to access the global production registry instance.

// Import all refactored modules
pub mod commands;
pub mod handlers;
pub mod mock;
pub mod service;
pub mod types;
pub mod watchers;

// Re-export public types and components for backward compatibility
pub use commands::{
    LocalCommandRegistry, ProductionCommandRegistry, get_command_registry,
    json_params_to_string_vec,
};
pub use service::TaskServiceImpl;
pub use types::{SimpleCommand, TaskUpdateSender};
pub use watchers::TaskWatcherManager;

// Mock implementations for testing only - use ProductionCommandRegistry in production
#[cfg(test)]
pub use mock::{MockCommand, MockCommandRegistry};

// Re-export commonly used items from the crate
pub use crate::task::manager::TaskManager;
pub use crate::task::types::{AgentType, Task, TaskPriority, TaskStatus};
