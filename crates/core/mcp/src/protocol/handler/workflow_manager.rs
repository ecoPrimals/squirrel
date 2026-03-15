// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Team workflow management implementation
//!
//! This module implements the TeamWorkflowManager which handles:
//! - Workflow creation and lifecycle management
//! - Team message broadcasting
//! - Review request management
//! - Task tracking and status updates
//! - Workflow metrics and filtering

use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use tokio::sync::RwLock;
use tracing::{info, instrument};

use crate::mcp::error::{MCPError, ErrorContext, ErrorSeverity};
use crate::mcp::SecurityLevel;
use crate::security::manager::SecurityManagerImpl;
use crate::security::rbac::Permission;

use super::team_types::*;

/// Manager for team workflows and collaboration
pub struct TeamWorkflowManager {
    workflows: Arc<RwLock<HashMap<String, TeamWorkflow>>>,
    messages: Arc<RwLock<HashMap<String, Vec<TeamMessage>>>>,
    reviews: Arc<RwLock<HashMap<String, ReviewRequest>>>,
    tasks: Arc<RwLock<HashMap<String, Vec<Task>>>>,
    security: Arc<SecurityManagerImpl>,
}

impl TeamWorkflowManager {
    #[instrument(skip(security))]
    pub fn new(security: Arc<SecurityManagerImpl>) -> Self {
        Self {
            workflows: Arc::new(RwLock::new(HashMap::new())),
            messages: Arc::new(RwLock::new(HashMap::new())),
            reviews: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            security,
        }
    }

    #[instrument(skip(self))]
    pub async fn create_workflow(&self, workflow: TeamWorkflow, token: &str) -> Result<(), MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "workflow".to_string(),
            action: "Write".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

        let mut workflows = self.workflows.write().await;
        workflows.insert(workflow.id.clone(), workflow);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_workflow(&self, id: &str, token: &str) -> Result<TeamWorkflow, MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "workflow".to_string(),
            action: "Read".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

        let workflows = self.workflows.read().await;
        workflows.get(id).cloned().ok_or_else(|| MCPError::Tool {
            kind: crate::mcp::error::ToolErrorKind::NotFound,
            context: ErrorContext::new("get_workflow", "team_workflow")
                .with_severity(ErrorSeverity::Medium),
            tool_id: id.to_string(),
        })
    }

    #[instrument(skip(self))]
    pub async fn update_workflow_status(&self, id: &str, status: WorkflowStatus, token: &str) -> Result<(), MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "workflow".to_string(),
            action: "Write".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

        let mut workflows = self.workflows.write().await;
        if let Some(workflow) = workflows.get_mut(id) {
            workflow.status = status;
            Ok(())
        } else {
            Err(MCPError::Tool {
                kind: crate::mcp::error::ToolErrorKind::NotFound,
                context: ErrorContext::new("update_workflow_status", "team_workflow")
                    .with_severity(ErrorSeverity::Medium),
                tool_id: id.to_string(),
            })
        }
    }

    #[instrument(skip(self))]
    pub async fn send_message(&self, workflow_id: &str, message: TeamMessage, token: &str) -> Result<(), MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "message".to_string(),
            action: "Write".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

        let mut messages = self.messages.write().await;
        messages.entry(workflow_id.to_string())
            .or_insert_with(Vec::new)
            .push(message);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_messages(&self, workflow_id: &str, token: &str) -> Result<Vec<TeamMessage>, MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "message".to_string(),
            action: "Read".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

        let messages = self.messages.read().await;
        Ok(messages.get(workflow_id).cloned().unwrap_or_default())
    }

    #[instrument(skip(self))]
    pub async fn create_review(&self, review: ReviewRequest, token: &str) -> Result<(), MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "review".to_string(),
            action: "Write".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

        let mut reviews = self.reviews.write().await;
        reviews.insert(review.id.clone(), review);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_review(&self, id: &str, token: &str) -> Result<ReviewRequest, MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "review".to_string(),
            action: "Read".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

        let reviews = self.reviews.read().await;
        reviews.get(id).cloned().ok_or_else(|| MCPError::Tool {
            kind: crate::mcp::error::ToolErrorKind::NotFound,
            context: ErrorContext::new("get_review", "team_workflow")
                .with_severity(ErrorSeverity::Medium),
            tool_id: id.to_string(),
        })
    }

    #[instrument(skip(self))]
    pub async fn transition_workflow_state(
        &self,
        id: &str,
        new_status: WorkflowStatus,
        initiator: &str,
        reason: &str,
        token: &str
    ) -> Result<(), MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "workflow".to_string(),
            action: "Write".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

        let mut workflows = self.workflows.write().await;
        if let Some(workflow) = workflows.get_mut(id) {
            let old_status = workflow.status.clone();
            workflow.status = new_status.clone();

            // Record state transition
            let transition = StateTransition {
                from_state: old_status,
                to_state: new_status,
                timestamp: Utc::now(),
                initiator: initiator.to_string(),
                reason: reason.to_string(),
            };

            // Update workflow metadata
            if let Ok(transition_json) = serde_json::to_string(&transition) {
                workflow.metadata.insert("last_transition".to_string(), transition_json);
            } else {
                tracing::error!("Failed to serialize workflow transition for workflow '{}'", id);
            }
            workflow.metadata.insert("last_updated".to_string(), Utc::now().to_string());

            info!(
                workflow_id = id,
                from_state = ?old_status,
                to_state = ?new_status,
                initiator = initiator,
                "Workflow state transition completed"
            );

            Ok(())
        } else {
            Err(MCPError::Tool {
                kind: crate::mcp::error::ToolErrorKind::NotFound,
                context: ErrorContext::new("transition_workflow_state", "team_workflow")
                    .with_severity(ErrorSeverity::Medium),
                tool_id: id.to_string(),
            })
        }
    }

    #[instrument(skip(self))]
    pub async fn get_workflow_metrics(&self, id: &str, token: &str) -> Result<WorkflowMetrics, MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "workflow".to_string(),
            action: "Read".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

        let workflows = self.workflows.read().await;
        let messages = self.messages.read().await;
        let reviews = self.reviews.read().await;

        let workflow = workflows.get(id).ok_or_else(|| MCPError::Tool {
            kind: crate::mcp::error::ToolErrorKind::NotFound,
            context: ErrorContext::new("get_workflow_metrics", "team_workflow")
                .with_severity(ErrorSeverity::Medium),
            tool_id: id.to_string(),
        })?;

        // Calculate metrics
        let workflow_messages = messages.get(id).cloned().unwrap_or_default();
        let workflow_reviews: Vec<_> = reviews.values()
            .filter(|r| workflow_messages.iter().any(|m| m.content.contains(&r.id)))
            .collect();

        let total_messages = workflow_messages.len() as u32;
        let review_count = workflow_reviews.len() as u32;

        // Calculate average review time
        let average_review_time = if !workflow_reviews.is_empty() {
            let total_time: f64 = workflow_messages.iter()
                .filter_map(|m| match &m.type_ {
                    TeamMessageType::Review => {
                        Some((Utc::now() - m.timestamp).num_seconds() as f64)
                    }
                    _ => None,
                })
                .sum();
            total_time / review_count as f64
        } else {
            0.0
        };

        // Calculate completion rate
        let completion_rate = match workflow.status {
            WorkflowStatus::Completed => 100.0,
            WorkflowStatus::Failed => 0.0,
            _ => {
                let total_tasks = workflow_messages.iter()
                    .filter(|m| matches!(m.type_, TeamMessageType::Task))
                    .count();
                let completed_tasks = workflow_messages.iter()
                    .filter(|m| {
                        matches!(m.type_, TeamMessageType::Task) &&
                        m.metadata.get("status").map_or(false, |s| s == "completed")
                    })
                    .count();
                if total_tasks > 0 {
                    (completed_tasks as f64 / total_tasks as f64) * 100.0
                } else {
                    0.0
                }
            }
        };

        // Parse state transitions from metadata
        let state_transitions = workflow.metadata.iter()
            .filter_map(|(key, value)| {
                if key.starts_with("transition_") {
                    serde_json::from_str::<StateTransition>(value).ok()
                } else {
                    None
                }
            })
            .collect();

        Ok(WorkflowMetrics {
            state_transitions,
            total_messages,
            review_count,
            average_review_time,
            completion_rate,
        })
    }

    #[instrument(skip(self))]
    pub async fn broadcast_team_message(
        &self,
        message: TeamMessage,
        workflows: Vec<String>,
        token: &str
    ) -> Result<(), MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "message".to_string(),
            action: "Write".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

        let mut messages = self.messages.write().await;
        for workflow_id in &workflows {
            messages.entry(workflow_id.clone())
                .or_insert_with(Vec::new)
                .push(message.clone());
        }

        info!(
            message_id = ?message.id,
            workflow_count = workflows.len(),
            "Team message broadcasted"
        );

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn create_task(&self, workflow_id: &str, task: Task, token: &str) -> Result<(), MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "task".to_string(),
            action: "Write".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

        let mut tasks = self.tasks.write().await;
        tasks.entry(workflow_id.to_string())
            .or_insert_with(Vec::new)
            .push(task.clone());

        // Create a task message
        let task_message = TeamMessage {
            id: format!("task_msg_{}", task.id),
            type_: TeamMessageType::Task,
            content: format!("Task created: {}", task.title),
            sender: task.assignee.clone(),
            timestamp: Utc::now(),
            security_level: SecurityLevel::High,
            metadata: {
                let mut map = HashMap::new();
                map.insert("task_id".to_string(), task.id);
                map.insert("status".to_string(), format!("{:?}", task.status));
                map
            },
        };

        self.send_message(workflow_id, task_message, token).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn update_task_status(
        &self,
        workflow_id: &str,
        task_id: &str,
        new_status: TaskStatus,
        token: &str
    ) -> Result<(), MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "task".to_string(),
            action: "Write".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

        let mut tasks = self.tasks.write().await;
        if let Some(workflow_tasks) = tasks.get_mut(workflow_id) {
            if let Some(task) = workflow_tasks.iter_mut().find(|t| t.id == task_id) {
                task.status = new_status.clone();
                task.updated_at = Utc::now();

                // Create a status update message
                let status_message = TeamMessage {
                    id: format!("task_update_{}", Utc::now().timestamp()),
                    type_: TeamMessageType::Status,
                    content: format!("Task {} status updated to {:?}", task_id, new_status),
                    sender: task.assignee.clone(),
                    timestamp: Utc::now(),
                    security_level: SecurityLevel::High,
                    metadata: {
                        let mut map = HashMap::new();
                        map.insert("task_id".to_string(), task_id.to_string());
                        map.insert("status".to_string(), format!("{:?}", new_status));
                        map
                    },
                };

                self.send_message(workflow_id, status_message, token).await?;
                Ok(())
            } else {
                Err(MCPError::Tool {
                    kind: crate::mcp::error::ToolErrorKind::NotFound,
                    context: ErrorContext::new("update_task_status", "team_workflow")
                        .with_severity(ErrorSeverity::Medium),
                    tool_id: task_id.to_string(),
                })
            }
        } else {
            Err(MCPError::Tool {
                kind: crate::mcp::error::ToolErrorKind::NotFound,
                context: ErrorContext::new("update_task_status", "team_workflow")
                    .with_severity(ErrorSeverity::Medium),
                tool_id: workflow_id.to_string(),
            })
        }
    }

    #[instrument(skip(self))]
    pub async fn filter_workflows(&self, filter: &WorkflowFilter, token: &str) -> Result<Vec<TeamWorkflow>, MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "workflow".to_string(),
            action: "Read".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

        let workflows = self.workflows.read().await;
        let filtered: Vec<TeamWorkflow> = workflows.values()
            .filter(|workflow| {
                // Apply filters
                let status_match = filter.status.as_ref()
                    .map_or(true, |s| &workflow.status == s);
                
                let security_match = filter.security_level.as_ref()
                    .map_or(true, |s| &workflow.security_level == s);
                
                let date_match = filter.date_range.as_ref()
                    .map_or(true, |(start, end)| {
                        workflow.metadata.get("last_updated")
                            .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
                            .map_or(false, |date| date >= *start && date <= *end)
                    });

                let assignee_match = filter.assignee.as_ref()
                    .map_or(true, |a| {
                        workflow.metadata.get("assignee")
                            .map_or(false, |assignee| assignee == a)
                    });

                let priority_match = filter.priority.as_ref()
                    .map_or(true, |p| {
                        workflow.metadata.get("priority")
                            .map_or(false, |priority| priority == &format!("{:?}", p))
                    });

                status_match && security_match && date_match && assignee_match && priority_match
            })
            .cloned()
            .collect();

        Ok(filtered)
    }

    #[instrument(skip(self))]
    pub async fn get_workflow_tasks(&self, workflow_id: &str, token: &str) -> Result<Vec<Task>, MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "task".to_string(),
            action: "Read".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

        let tasks = self.tasks.read().await;
        Ok(tasks.get(workflow_id).cloned().unwrap_or_default())
    }
}

