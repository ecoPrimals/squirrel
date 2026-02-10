// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Tool executor trait
//!
//! This module defines the ToolExecutor trait for executing tool capabilities.

use std::fmt;

use super::error::ToolError;
use super::execution::{ToolContext, ToolExecutionResult};

/// Trait for tool executors
pub trait ToolExecutor: fmt::Debug + Send + Sync {
    /// Executes a capability with the given context
    fn execute(&self, context: ToolContext) -> impl std::future::Future<Output = Result<ToolExecutionResult, ToolError>> + Send;

    /// Gets the tool ID this executor is associated with
    fn get_tool_id(&self) -> String;

    /// Gets the capabilities this executor can handle
    fn get_capabilities(&self) -> Vec<String>;

    /// Starts the executor
    fn start(&self) -> impl std::future::Future<Output = Result<(), ToolError>> + Send {
        async {
            // Default implementation does nothing
            Ok(())
        }
    }

    /// Stops the executor
    fn stop(&self) -> impl std::future::Future<Output = Result<(), ToolError>> + Send {
        async {
            // Default implementation does nothing
            Ok(())
        }
    }

    /// Pauses the executor
    fn pause(&self) -> impl std::future::Future<Output = Result<(), ToolError>> + Send {
        async {
            // Default implementation does nothing
            Ok(())
        }
    }

    /// Resumes the executor
    fn resume(&self) -> impl std::future::Future<Output = Result<(), ToolError>> + Send {
        async {
            // Default implementation does nothing
            Ok(())
        }
    }
}

