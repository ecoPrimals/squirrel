// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Workflow RBAC [`Permission`] builders and pure helpers for workflow metrics and filtering.

use std::collections::HashMap;

use chrono::Utc;

use super::team_types::{
    ReviewRequest, StateTransition, TeamMessage, TeamMessageType, TeamWorkflow, WorkflowFilter,
    WorkflowMetrics, WorkflowStatus,
};
use crate::security::rbac::Permission;

/// RBAC permission for reading workflow state.
#[must_use]
pub fn workflow_read() -> Permission {
    Permission {
        resource: "workflow".to_string(),
        action: "Read".to_string(),
    }
}

/// RBAC permission for creating or mutating workflows.
#[must_use]
pub fn workflow_write() -> Permission {
    Permission {
        resource: "workflow".to_string(),
        action: "Write".to_string(),
    }
}

/// RBAC permission for reading team messages.
#[must_use]
pub fn message_read() -> Permission {
    Permission {
        resource: "message".to_string(),
        action: "Read".to_string(),
    }
}

/// RBAC permission for posting team messages.
#[must_use]
pub fn message_write() -> Permission {
    Permission {
        resource: "message".to_string(),
        action: "Write".to_string(),
    }
}

/// RBAC permission for reading review requests.
#[must_use]
pub fn review_read() -> Permission {
    Permission {
        resource: "review".to_string(),
        action: "Read".to_string(),
    }
}

/// RBAC permission for creating or updating review requests.
#[must_use]
pub fn review_write() -> Permission {
    Permission {
        resource: "review".to_string(),
        action: "Write".to_string(),
    }
}

/// RBAC permission for reading workflow tasks.
#[must_use]
pub fn task_read() -> Permission {
    Permission {
        resource: "task".to_string(),
        action: "Read".to_string(),
    }
}

/// RBAC permission for creating or updating workflow tasks.
#[must_use]
pub fn task_write() -> Permission {
    Permission {
        resource: "task".to_string(),
        action: "Write".to_string(),
    }
}

/// Returns true when `workflow` satisfies all set fields on `filter`.
#[must_use]
pub fn workflow_matches_filter(workflow: &TeamWorkflow, filter: &WorkflowFilter) -> bool {
    let status_match = filter.status.as_ref().is_none_or(|s| &workflow.status == s);

    let security_match = filter
        .security_level
        .as_ref()
        .is_none_or(|s| &workflow.security_level == s);

    let date_match = filter.date_range.as_ref().is_none_or(|(start, end)| {
        workflow
            .metadata
            .get("last_updated")
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
            .is_some_and(|date| date >= *start && date <= *end)
    });

    let assignee_match = filter
        .assignee
        .as_ref()
        .is_none_or(|a| workflow.metadata.get("assignee") == Some(a));

    let priority_match = filter.priority.as_ref().is_none_or(|p| {
        workflow
            .metadata
            .get("priority")
            .is_some_and(|priority| priority == &format!("{p:?}"))
    });

    status_match && security_match && date_match && assignee_match && priority_match
}

/// Aggregates message, review, and completion metrics for a single workflow snapshot.
#[must_use]
#[expect(
    clippy::implicit_hasher,
    reason = "Public API uses default HashMap; callers pass standard maps"
)]
pub fn compute_workflow_metrics(
    workflow: &TeamWorkflow,
    workflow_messages: &[TeamMessage],
    reviews: &HashMap<String, ReviewRequest>,
) -> WorkflowMetrics {
    let workflow_reviews: Vec<_> = reviews
        .values()
        .filter(|r| workflow_messages.iter().any(|m| m.content.contains(&r.id)))
        .collect();

    let total_messages = workflow_messages.len() as u32;
    let review_count = workflow_reviews.len() as u32;

    let average_review_time = if workflow_reviews.is_empty() {
        0.0
    } else {
        let total_time: f64 = workflow_messages
            .iter()
            .filter_map(|m| match &m.type_ {
                TeamMessageType::Review => Some((Utc::now() - m.timestamp).num_seconds() as f64),
                _ => None,
            })
            .sum();
        total_time / f64::from(review_count)
    };

    let completion_rate = match workflow.status {
        WorkflowStatus::Completed => 100.0,
        WorkflowStatus::Failed => 0.0,
        _ => {
            let total_tasks = workflow_messages
                .iter()
                .filter(|m| matches!(m.type_, TeamMessageType::Task))
                .count();
            let completed_tasks = workflow_messages
                .iter()
                .filter(|m| {
                    matches!(m.type_, TeamMessageType::Task)
                        && m.metadata.get("status").is_some_and(|s| s == "completed")
                })
                .count();
            if total_tasks > 0 {
                (completed_tasks as f64 / total_tasks as f64) * 100.0
            } else {
                0.0
            }
        }
    };

    let state_transitions = workflow
        .metadata
        .iter()
        .filter_map(|(key, value)| {
            if key.starts_with("transition_") {
                serde_json::from_str::<StateTransition>(value).ok()
            } else {
                None
            }
        })
        .collect();

    WorkflowMetrics {
        state_transitions,
        total_messages,
        review_count,
        average_review_time,
        completion_rate,
    }
}
