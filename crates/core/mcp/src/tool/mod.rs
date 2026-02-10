// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Tool management module for MCP
//!
//! This module provides the core tool management functionality for MCP.

// Core tool modules
pub mod cleanup;
pub mod executor;
pub mod lifecycle;
pub mod management;

// Re-export core types and traits
pub use management::types::{Tool, ToolState, ToolError, ToolInfo};
pub use management::CoreToolManager;

// Import trait from management module to avoid conflicts
use management::ToolManager as ToolManagerTrait;

// Re-export the trait with a different name to avoid conflicts
pub use management::ToolManager;

// Core tool management interface
pub use management::ToolManager as ToolManagerInterface;
