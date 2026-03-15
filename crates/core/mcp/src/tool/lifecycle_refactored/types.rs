// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Core Types for Tool Lifecycle Management
//!
//! This module contains all the fundamental types, enums, and data structures
//! used throughout the tool lifecycle management system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::tool::ToolState;

/// Type alias for tool state history entries
pub type StateHistoryEntry = (ToolState, DateTime<Utc>);

/// Type alias for tool state history map
pub type StateHistoryMap = HashMap<String, Vec<StateHistoryEntry>>;

/// Lifecycle event types for tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecycleEvent {
    /// Tool is registered
    Register,
    /// Tool is unregistered
    Unregister,
    /// Tool is activated
    Activate,
    /// Tool is deactivated
    Deactivate,
    /// Tool encounters an error
    Error,
    /// Tool is about to start
    PreStart,
    /// Tool has started
    PostStart,
    /// Tool is about to stop
    PreStop,
    /// Tool has stopped
    PostStop,
    /// Tool is paused
    Pause,
    /// Tool is resumed
    Resume,
    /// Tool is updated
    Update,
    /// Tool is cleaned up
    Cleanup,
    /// Tool is initialized
    Initialize,
    /// Tool is about to execute
    PreExecute,
    /// Tool has executed
    PostExecute,
    /// Tool is reset
    Reset,
}

impl fmt::Display for LifecycleEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Register => write!(f, "register"),
            Self::Unregister => write!(f, "unregister"),
            Self::Activate => write!(f, "activate"),
            Self::Deactivate => write!(f, "deactivate"),
            Self::Error => write!(f, "error"),
            Self::PreStart => write!(f, "pre_start"),
            Self::PostStart => write!(f, "post_start"),
            Self::PreStop => write!(f, "pre_stop"),
            Self::PostStop => write!(f, "post_stop"),
            Self::Pause => write!(f, "pause"),
            Self::Resume => write!(f, "resume"),
            Self::Update => write!(f, "update"),
            Self::Cleanup => write!(f, "cleanup"),
            Self::Initialize => write!(f, "initialize"),
            Self::PreExecute => write!(f, "pre_execute"),
            Self::PostExecute => write!(f, "post_execute"),
            Self::Reset => write!(f, "reset"),
        }
    }
}

/// Tool lifecycle event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleRecord {
    /// Tool ID
    pub tool_id: String,
    /// Event type
    pub event: LifecycleEvent,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Tool state before the event
    pub previous_state: Option<ToolState>,
    /// Tool state after the event
    pub new_state: Option<ToolState>,
    /// Error message if applicable
    pub error_message: Option<String>,
    /// Duration of the operation in milliseconds (if applicable)
    pub duration_ms: Option<u64>,
}

/// Tool recovery strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStrategy {
    /// Maximum number of recovery attempts
    pub max_attempts: u32,
    /// Backoff strategy for retry attempts
    pub backoff_strategy: BackoffStrategy,
    /// Recovery actions to attempt
    pub recovery_actions: Vec<RecoveryAction>,
}

/// Backoff strategy for recovery attempts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// Fixed delay between attempts (milliseconds)
    Fixed(u64),
    /// Exponential backoff (base milliseconds)
    Exponential(u64),
    /// Linear backoff (increase milliseconds)
    Linear(u64),
}

/// Recovery action for tool errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryAction {
    /// Reset the tool
    Reset,
    /// Restart the tool
    Restart,
    /// Recreate the tool
    Recreate,
    /// Custom action name (to be handled by the recovery hook)
    Custom(String),
}

/// Record of a recovery attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAttempt {
    /// Tool ID
    pub tool_id: String,
    /// Recovery action attempted
    pub action: RecoveryAction,
    /// Timestamp of the attempt
    pub timestamp: DateTime<Utc>,
    /// Whether the recovery was successful
    pub successful: bool,
    /// Error message if recovery failed
    pub error_message: Option<String>,
    /// Attempt number
    pub attempt_number: u32,
} 