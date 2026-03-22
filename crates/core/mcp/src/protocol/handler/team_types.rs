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

/// Team message types for collaboration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TeamMessageType {
    /// Code review feedback with location and severity.
    CodeReview {
        /// Path to the reviewed file.
        file_path: String,
        /// Line numbers under review.
        line_numbers: Vec<u32>,
        /// Review severity level.
        severity: ReviewSeverity,
        /// Reviewer comment text.
        comment: String,
        /// Rule identifiers that were violated.
        rule_violations: Vec<String>,
    },
    /// Documentation change proposal or update.
    DocumentationUpdate {
        /// Component or area being documented.
        component: String,
        /// Kind of documentation affected.
        doc_type: DocType,
        /// Proposed or updated documentation body.
        content: String,
        /// Scheduling priority for the update.
        priority: Priority,
    },
    /// Process or pipeline failure notification.
    ProcessError {
        /// Failing component name.
        component: String,
        /// Category of process failure.
        error_type: ProcessErrorType,
        /// Human-readable error details.
        details: String,
        /// Rules implicated by the failure.
        affected_rules: Vec<String>,
    },
    /// Build or CI status for a branch.
    BuildStatus {
        /// Branch name.
        branch: String,
        /// Outcome of the build.
        status: BuildStatus,
        /// Build timing and quality metrics.
        metrics: BuildMetrics,
        /// Non-fatal warnings emitted by the build.
        warnings: Vec<String>,
    },
    /// Task-related team message.
    Task,
    /// Review-related team message.
    Review,
    /// General comment.
    Comment,
    /// Status update.
    Status,
    /// Alert or escalation.
    Alert,
}

/// Severity levels for code reviews.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewSeverity {
    /// Informational note.
    Info,
    /// Warning that should be addressed.
    Warning,
    /// Error-level issue.
    Error,
    /// Blocking or security-critical issue.
    Critical,
}

/// Documentation categories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocType {
    /// API reference documentation.
    API,
    /// System architecture documentation.
    Architecture,
    /// Rules and policy documentation.
    Rules,
    /// Process documentation.
    Process,
    /// Source code documentation.
    Code,
    /// Design documentation.
    Design,
    /// General prose documentation.
    Documentation,
    /// Test documentation.
    Test,
    /// Uncategorized documentation.
    Other,
}

/// Priority levels for tasks and messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    /// Low urgency.
    Low,
    /// Normal urgency.
    Medium,
    /// High urgency.
    High,
    /// Time-sensitive urgency.
    Urgent,
    /// Highest urgency.
    Critical,
}

/// Categories of process failures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessErrorType {
    /// Build failure.
    Build,
    /// Test failure.
    Test,
    /// Lint failure.
    Lint,
    /// Security scan failure.
    Security,
    /// Performance regression.
    Performance,
}

/// High-level build outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildStatus {
    /// Build completed successfully.
    Success,
    /// Build succeeded with warnings.
    Warning,
    /// Build failed.
    Error,
    /// Build still running.
    InProgress,
}

/// Build timing and quality metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetrics {
    /// Build duration in seconds.
    pub duration: f64,
    /// Test coverage ratio (0–1 or percent per caller convention).
    pub test_coverage: f64,
    /// Count of warnings.
    pub warnings_count: u32,
    /// Count of errors.
    pub errors_count: u32,
}

/// Team workflow definition and access control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamWorkflow {
    /// Workflow identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Longer description of the workflow purpose.
    pub description: String,
    /// Current lifecycle state.
    pub status: WorkflowStatus,
    /// Minimum security tier for participants.
    pub security_level: SecurityLevel,
    /// RBAC permissions required to interact with the workflow.
    pub permissions: Vec<Permission>,
    /// Arbitrary metadata (e.g. assignee, last transition).
    pub metadata: HashMap<String, String>,
}

/// Workflow lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Workflow is running.
    Active,
    /// Workflow is temporarily paused.
    Paused,
    /// Workflow finished successfully.
    Completed,
    /// Workflow ended in failure.
    Failed,
}

/// Team channel message envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMessage {
    /// Message identifier.
    pub id: String,
    /// Discriminated message payload type.
    pub type_: TeamMessageType,
    /// Primary text body.
    pub content: String,
    /// Sender identifier.
    pub sender: String,
    /// UTC timestamp when the message was sent.
    pub timestamp: DateTime<Utc>,
    /// Security tier applied to this message.
    pub security_level: SecurityLevel,
    /// Additional structured metadata.
    pub metadata: HashMap<String, String>,
}

/// Request for a structured review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRequest {
    /// Review request id.
    pub id: String,
    /// Short title.
    pub title: String,
    /// Detailed description of what to review.
    pub description: String,
    /// Assigned reviewer identity.
    pub reviewer: String,
    /// Expected severity of findings.
    pub severity: ReviewSeverity,
    /// Documentation area under review.
    pub doc_type: DocType,
    /// Scheduling priority.
    pub priority: Priority,
    /// Minimum security clearance for the review.
    pub security_level: SecurityLevel,
}

/// Aggregated metrics for a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetrics {
    /// Recorded state transitions.
    pub state_transitions: Vec<StateTransition>,
    /// Total messages observed for the workflow.
    pub total_messages: u32,
    /// Number of review-related messages.
    pub review_count: u32,
    /// Average time to complete reviews (seconds).
    pub average_review_time: f64,
    /// Fraction of work completed (0–100).
    pub completion_rate: f64,
}

/// Single workflow state transition audit entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Previous workflow status.
    pub from_state: WorkflowStatus,
    /// New workflow status.
    pub to_state: WorkflowStatus,
    /// When the transition occurred.
    pub timestamp: DateTime<Utc>,
    /// Actor that initiated the transition.
    pub initiator: String,
    /// Human-readable reason for the change.
    pub reason: String,
}

/// Work item tracked within a team workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task identifier.
    pub id: String,
    /// Short title.
    pub title: String,
    /// Detailed description.
    pub description: String,
    /// Owner or assignee.
    pub assignee: String,
    /// Current task state.
    pub status: TaskStatus,
    /// Scheduling priority.
    pub priority: Priority,
    /// Optional due date.
    pub due_date: Option<DateTime<Utc>>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
    /// Arbitrary task metadata.
    pub metadata: HashMap<String, String>,
}

/// Task lifecycle states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Not yet started.
    Todo,
    /// Actively in progress.
    InProgress,
    /// Blocked on an external dependency.
    Blocked,
    /// Awaiting review.
    UnderReview,
    /// Finished successfully.
    Completed,
    /// Cancelled without completion.
    Cancelled,
}

/// Filter criteria when listing or searching workflows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowFilter {
    /// Match workflows in this status, if set.
    pub status: Option<WorkflowStatus>,
    /// Match workflows at this security level, if set.
    pub security_level: Option<SecurityLevel>,
    /// Match workflows last updated in this UTC range, if set.
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// Match workflows assigned to this user, if set.
    pub assignee: Option<String>,
    /// Match workflows with this priority, if set.
    pub priority: Option<Priority>,
}
