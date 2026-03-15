// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Tool definitions and state management
//!
//! This module contains the core tool types including Tool, ToolInfo, and ToolState.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::capability::Capability;

/// Basic tool information for MCP core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool ID
    pub id: String,
    /// Tool name
    pub name: String,
    /// Tool version
    pub version: String,
    /// Tool description
    pub description: String,
    /// Tool capabilities
    pub capabilities: Vec<Capability>,
    /// Tool security level (0-10, 0 being lowest)
    pub security_level: u8,
}

/// Tool state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolState {
    /// Tool is registered but not active
    Registered,
    /// Tool is active and ready to execute
    Active,
    /// Tool is in the starting process
    Starting,
    /// Tool has started
    Started,
    /// Tool is in the stopping process
    Stopping,
    /// Tool has been stopped
    Stopped,
    /// Tool is in the pausing process
    Pausing,
    /// Tool is paused
    Paused,
    /// Tool is in the resuming process
    Resuming,
    /// Tool is being updated
    Updating,
    /// Tool is in error state
    Error,
    /// Tool is unregistered
    Unregistered,
    /// Tool is in recovery process
    Recovering,
    /// Tool is inactive (but still registered)
    Inactive,
    /// Tool is running (executing)
    Running,
    /// Tool is initializing
    Initializing,
    /// Tool is resetting
    Resetting,
}

impl fmt::Display for ToolState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registered => write!(f, "Registered"),
            Self::Active => write!(f, "Active"),
            Self::Starting => write!(f, "Starting"),
            Self::Started => write!(f, "Started"),
            Self::Stopping => write!(f, "Stopping"),
            Self::Stopped => write!(f, "Stopped"),
            Self::Pausing => write!(f, "Pausing"),
            Self::Paused => write!(f, "Paused"),
            Self::Resuming => write!(f, "Resuming"),
            Self::Updating => write!(f, "Updating"),
            Self::Error => write!(f, "Error"),
            Self::Unregistered => write!(f, "Unregistered"),
            Self::Recovering => write!(f, "Recovering"),
            Self::Inactive => write!(f, "Inactive"),
            Self::Running => write!(f, "Running"),
            Self::Initializing => write!(f, "Initializing"),
            Self::Resetting => write!(f, "Resetting"),
        }
    }
}

