// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Team workflow management implementation
//!
//! This module implements the TeamWorkflowManager which handles:
//! - Workflow creation and lifecycle management
//! - Team message broadcasting
//! - Review request management
//! - Task tracking and status updates
//! - Workflow metrics and filtering

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument};

use crate::error::SecurityLevel;
use crate::error::{MCPError, tool::ToolError};
use crate::security::manager::SecurityManagerImpl;

use super::team_types::{
    ReviewRequest, StateTransition, Task, TaskStatus, TeamMessage, TeamMessageType, TeamWorkflow,
    WorkflowFilter, WorkflowMetrics, WorkflowStatus,
};
use super::workflow_types::{
    compute_workflow_metrics, message_read, message_write, review_read, review_write, task_read,
    task_write, workflow_matches_filter, workflow_read, workflow_write,
};

/// Manager for team workflows and collaboration
pub struct TeamWorkflowManager {
    workflows: Arc<RwLock<HashMap<String, TeamWorkflow>>>,
    messages: Arc<RwLock<HashMap<String, Vec<TeamMessage>>>>,
    reviews: Arc<RwLock<HashMap<String, ReviewRequest>>>,
    tasks: Arc<RwLock<HashMap<String, Vec<Task>>>>,
    security: Arc<SecurityManagerImpl>,
}

impl TeamWorkflowManager {
    /// Creates a manager backed by the given security implementation for permission checks.
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

    /// Stores a new workflow after verifying write permission on the auth token.
    #[instrument(skip(self))]
    pub async fn create_workflow(
        &self,
        workflow: TeamWorkflow,
        token: &str,
    ) -> Result<(), MCPError> {
        self.security
            .check_permission(token, &workflow_write())
            .await?;

        let mut workflows = self.workflows.write().await;
        workflows.insert(workflow.id.clone(), workflow);
        Ok(())
    }

    /// Returns the workflow by id after read permission is granted.
    #[instrument(skip(self))]
    pub async fn get_workflow(&self, id: &str, token: &str) -> Result<TeamWorkflow, MCPError> {
        self.security
            .check_permission(token, &workflow_read())
            .await?;

        let workflows = self.workflows.read().await;
        workflows
            .get(id)
            .cloned()
            .ok_or_else(|| MCPError::Tool(ToolError::NotFound(id.to_string())))
    }

    /// Sets the workflow status after write permission is granted.
    #[instrument(skip(self))]
    pub async fn update_workflow_status(
        &self,
        id: &str,
        status: WorkflowStatus,
        token: &str,
    ) -> Result<(), MCPError> {
        self.security
            .check_permission(token, &workflow_write())
            .await?;

        let mut workflows = self.workflows.write().await;
        if let Some(workflow) = workflows.get_mut(id) {
            workflow.status = status;
            Ok(())
        } else {
            Err(MCPError::Tool(ToolError::NotFound(id.to_string())))
        }
    }

    /// Appends a message to the workflow log after message write permission is granted.
    #[instrument(skip(self))]
    pub async fn send_message(
        &self,
        workflow_id: &str,
        message: TeamMessage,
        token: &str,
    ) -> Result<(), MCPError> {
        self.security
            .check_permission(token, &message_write())
            .await?;

        let mut messages = self.messages.write().await;
        messages
            .entry(workflow_id.to_string())
            .or_insert_with(Vec::new)
            .push(message);
        Ok(())
    }

    /// Returns all messages for a workflow after read permission is granted.
    #[instrument(skip(self))]
    pub async fn get_messages(
        &self,
        workflow_id: &str,
        token: &str,
    ) -> Result<Vec<TeamMessage>, MCPError> {
        self.security
            .check_permission(token, &message_read())
            .await?;

        let messages = self.messages.read().await;
        Ok(messages.get(workflow_id).cloned().unwrap_or_default())
    }

    /// Registers a review request after review write permission is granted.
    #[instrument(skip(self))]
    pub async fn create_review(&self, review: ReviewRequest, token: &str) -> Result<(), MCPError> {
        self.security
            .check_permission(token, &review_write())
            .await?;

        let mut reviews = self.reviews.write().await;
        reviews.insert(review.id.clone(), review);
        Ok(())
    }

    /// Returns a review by id after read permission is granted.
    #[instrument(skip(self))]
    pub async fn get_review(&self, id: &str, token: &str) -> Result<ReviewRequest, MCPError> {
        self.security
            .check_permission(token, &review_read())
            .await?;

        let reviews = self.reviews.read().await;
        reviews
            .get(id)
            .cloned()
            .ok_or_else(|| MCPError::Tool(ToolError::NotFound(id.to_string())))
    }

    /// Moves a workflow to a new status and records transition metadata.
    #[instrument(skip(self))]
    pub async fn transition_workflow_state(
        &self,
        id: &str,
        new_status: WorkflowStatus,
        initiator: &str,
        reason: &str,
        token: &str,
    ) -> Result<(), MCPError> {
        self.security
            .check_permission(token, &workflow_write())
            .await?;

        let mut workflows = self.workflows.write().await;
        if let Some(workflow) = workflows.get_mut(id) {
            let old_status = workflow.status;
            workflow.status = new_status;

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
                workflow
                    .metadata
                    .insert("last_transition".to_string(), transition_json);
            } else {
                tracing::error!(
                    "Failed to serialize workflow transition for workflow '{}'",
                    id
                );
            }
            workflow
                .metadata
                .insert("last_updated".to_string(), Utc::now().to_string());

            info!(
                workflow_id = id,
                from_state = ?old_status,
                to_state = ?new_status,
                initiator = initiator,
                "Workflow state transition completed"
            );

            Ok(())
        } else {
            Err(MCPError::Tool(ToolError::NotFound(id.to_string())))
        }
    }

    /// Computes message and completion metrics for a workflow after read access.
    #[instrument(skip(self))]
    pub async fn get_workflow_metrics(
        &self,
        id: &str,
        token: &str,
    ) -> Result<WorkflowMetrics, MCPError> {
        self.security
            .check_permission(token, &workflow_read())
            .await?;

        let workflows = self.workflows.read().await;
        let messages = self.messages.read().await;
        let reviews = self.reviews.read().await;

        let workflow = workflows
            .get(id)
            .ok_or_else(|| MCPError::Tool(ToolError::NotFound(id.to_string())))?;

        let workflow_messages = messages.get(id).cloned().unwrap_or_default();

        Ok(compute_workflow_metrics(
            workflow,
            &workflow_messages,
            &reviews,
        ))
    }

    /// Duplicates a message into each listed workflow after write permission is granted.
    #[instrument(skip(self))]
    pub async fn broadcast_team_message(
        &self,
        message: TeamMessage,
        workflows: Vec<String>,
        token: &str,
    ) -> Result<(), MCPError> {
        self.security
            .check_permission(token, &message_write())
            .await?;

        let mut messages = self.messages.write().await;
        for workflow_id in &workflows {
            messages
                .entry(workflow_id.clone())
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

    /// Adds a task to a workflow and emits a corresponding task message.
    #[instrument(skip(self))]
    pub async fn create_task(
        &self,
        workflow_id: &str,
        task: Task,
        token: &str,
    ) -> Result<(), MCPError> {
        self.security.check_permission(token, &task_write()).await?;

        let mut tasks = self.tasks.write().await;
        tasks
            .entry(workflow_id.to_string())
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

    /// Updates a task status and posts a status message to the workflow.
    #[instrument(skip(self))]
    pub async fn update_task_status(
        &self,
        workflow_id: &str,
        task_id: &str,
        new_status: TaskStatus,
        token: &str,
    ) -> Result<(), MCPError> {
        self.security.check_permission(token, &task_write()).await?;

        let mut tasks = self.tasks.write().await;
        if let Some(workflow_tasks) = tasks.get_mut(workflow_id) {
            if let Some(task) = workflow_tasks.iter_mut().find(|t| t.id == task_id) {
                let content = format!("Task {task_id} status updated to {new_status:?}");
                task.status = new_status;
                task.updated_at = Utc::now();

                // Create a status update message
                let status_message = TeamMessage {
                    id: format!("task_update_{}", Utc::now().timestamp()),
                    type_: TeamMessageType::Status,
                    content,
                    sender: task.assignee.clone(),
                    timestamp: Utc::now(),
                    security_level: SecurityLevel::High,
                    metadata: {
                        let mut map = HashMap::new();
                        map.insert("task_id".to_string(), task_id.to_string());
                        map.insert("status".to_string(), format!("{:?}", task.status));
                        map
                    },
                };

                self.send_message(workflow_id, status_message, token)
                    .await?;
                Ok(())
            } else {
                Err(MCPError::Tool(ToolError::NotFound(task_id.to_string())))
            }
        } else {
            Err(MCPError::Tool(ToolError::NotFound(workflow_id.to_string())))
        }
    }

    /// Returns workflows matching the filter after read permission is granted.
    #[instrument(skip(self))]
    pub async fn filter_workflows(
        &self,
        filter: &WorkflowFilter,
        token: &str,
    ) -> Result<Vec<TeamWorkflow>, MCPError> {
        self.security
            .check_permission(token, &workflow_read())
            .await?;

        let workflows = self.workflows.read().await;
        let filtered: Vec<TeamWorkflow> = workflows
            .values()
            .filter(|workflow| workflow_matches_filter(workflow, filter))
            .cloned()
            .collect();

        Ok(filtered)
    }

    /// Returns all tasks recorded for a workflow after task read permission is granted.
    #[instrument(skip(self))]
    pub async fn get_workflow_tasks(
        &self,
        workflow_id: &str,
        token: &str,
    ) -> Result<Vec<Task>, MCPError> {
        self.security.check_permission(token, &task_read()).await?;

        let tasks = self.tasks.read().await;
        Ok(tasks.get(workflow_id).cloned().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests;
