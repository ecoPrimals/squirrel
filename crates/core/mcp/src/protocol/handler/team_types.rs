// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Team collaboration type definitions
//!
//! This module contains all type definitions for team workflows, messages,
//! reviews, and tasks used in the MCP protocol handler.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::SecurityLevel;
use crate::security::rbac::Permission;

/// Team message types for collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TeamMessageType {
    CodeReview {
        file_path: String,
        line_numbers: Vec<u32>,
        severity: ReviewSeverity,
        comment: String,
        rule_violations: Vec<String>,
    },
    DocumentationUpdate {
        component: String,
        doc_type: DocType,
        content: String,
        priority: Priority,
    },
    ProcessError {
        component: String,
        error_type: ProcessErrorType,
        details: String,
        affected_rules: Vec<String>,
    },
    BuildStatus {
        branch: String,
        status: BuildStatus,
        metrics: BuildMetrics,
        warnings: Vec<String>,
    },
    Task,
    Review,
    Comment,
    Status,
    Alert,
}

/// Severity levels for code reviews
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Documentation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocType {
    API,
    Architecture,
    Rules,
    Process,
    Code,
    Design,
    Documentation,
    Test,
    Other,
}

/// Priority levels for tasks and messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Urgent,
    Critical,
}

/// Process error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessErrorType {
    Build,
    Test,
    Lint,
    Security,
    Performance,
}

/// Build status states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildStatus {
    Success,
    Warning,
    Error,
    InProgress,
}

/// Build metrics information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetrics {
    pub duration: f64,
    pub test_coverage: f64,
    pub warnings_count: u32,
    pub errors_count: u32,
}

/// Team workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamWorkflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: WorkflowStatus,
    pub security_level: SecurityLevel,
    pub permissions: Vec<Permission>,
    pub metadata: HashMap<String, String>,
}

/// Workflow status states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Active,
    Paused,
    Completed,
    Failed,
}

/// Team message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMessage {
    pub id: String,
    pub type_: TeamMessageType,
    pub content: String,
    pub sender: String,
    pub timestamp: DateTime<Utc>,
    pub security_level: SecurityLevel,
    pub metadata: HashMap<String, String>,
}

/// Review request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRequest {
    pub id: String,
    pub title: String,
    pub description: String,
    pub reviewer: String,
    pub severity: ReviewSeverity,
    pub doc_type: DocType,
    pub priority: Priority,
    pub security_level: SecurityLevel,
}

/// Workflow metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetrics {
    pub state_transitions: Vec<StateTransition>,
    pub total_messages: u32,
    pub review_count: u32,
    pub average_review_time: f64,
    pub completion_rate: f64,
}

/// State transition tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from_state: WorkflowStatus,
    pub to_state: WorkflowStatus,
    pub timestamp: DateTime<Utc>,
    pub initiator: String,
    pub reason: String,
}

/// Task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub assignee: String,
    pub status: TaskStatus,
    pub priority: Priority,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Task status states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Blocked,
    UnderReview,
    Completed,
    Cancelled,
}

/// Workflow filter criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowFilter {
    pub status: Option<WorkflowStatus>,
    pub security_level: Option<SecurityLevel>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub assignee: Option<String>,
    pub priority: Option<Priority>,
}
