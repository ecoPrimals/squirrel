// SPDX-License-Identifier: AGPL-3.0-only
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
use crate::security::rbac::Permission;

use super::team_types::{
    ReviewRequest, StateTransition, Task, TaskStatus, TeamMessage, TeamMessageType, TeamWorkflow,
    WorkflowFilter, WorkflowMetrics, WorkflowStatus,
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

    /// Returns the workflow by id after read permission is granted.
    #[instrument(skip(self))]
    pub async fn get_workflow(&self, id: &str, token: &str) -> Result<TeamWorkflow, MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "workflow".to_string(),
            action: "Read".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

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
        // Validate permissions
        let permission = Permission {
            resource: "message".to_string(),
            action: "Write".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

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
        // Validate permissions
        let permission = Permission {
            resource: "message".to_string(),
            action: "Read".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

        let messages = self.messages.read().await;
        Ok(messages.get(workflow_id).cloned().unwrap_or_default())
    }

    /// Registers a review request after review write permission is granted.
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

    /// Returns a review by id after read permission is granted.
    #[instrument(skip(self))]
    pub async fn get_review(&self, id: &str, token: &str) -> Result<ReviewRequest, MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "review".to_string(),
            action: "Read".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

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
        // Validate permissions
        let permission = Permission {
            resource: "workflow".to_string(),
            action: "Write".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

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
        // Validate permissions
        let permission = Permission {
            resource: "workflow".to_string(),
            action: "Read".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

        let workflows = self.workflows.read().await;
        let messages = self.messages.read().await;
        let reviews = self.reviews.read().await;

        let workflow = workflows
            .get(id)
            .ok_or_else(|| MCPError::Tool(ToolError::NotFound(id.to_string())))?;

        // Calculate metrics
        let workflow_messages = messages.get(id).cloned().unwrap_or_default();
        let workflow_reviews: Vec<_> = reviews
            .values()
            .filter(|r| workflow_messages.iter().any(|m| m.content.contains(&r.id)))
            .collect();

        let total_messages = workflow_messages.len() as u32;
        let review_count = workflow_reviews.len() as u32;

        // Calculate average review time
        let average_review_time = if workflow_reviews.is_empty() {
            0.0
        } else {
            let total_time: f64 = workflow_messages
                .iter()
                .filter_map(|m| match &m.type_ {
                    TeamMessageType::Review => {
                        Some((Utc::now() - m.timestamp).num_seconds() as f64)
                    }
                    _ => None,
                })
                .sum();
            total_time / f64::from(review_count)
        };

        // Calculate completion rate
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

        // Parse state transitions from metadata
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

        Ok(WorkflowMetrics {
            state_transitions,
            total_messages,
            review_count,
            average_review_time,
            completion_rate,
        })
    }

    /// Duplicates a message into each listed workflow after write permission is granted.
    #[instrument(skip(self))]
    pub async fn broadcast_team_message(
        &self,
        message: TeamMessage,
        workflows: Vec<String>,
        token: &str,
    ) -> Result<(), MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "message".to_string(),
            action: "Write".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

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
        // Validate permissions
        let permission = Permission {
            resource: "task".to_string(),
            action: "Write".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

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
        // Validate permissions
        let permission = Permission {
            resource: "task".to_string(),
            action: "Write".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

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
        // Validate permissions
        let permission = Permission {
            resource: "workflow".to_string(),
            action: "Read".to_string(),
        };
        self.security.check_permission(token, &permission).await?;

        let workflows = self.workflows.read().await;
        let filtered: Vec<TeamWorkflow> = workflows
            .values()
            .filter(|workflow| {
                // Apply filters
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
            })
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

#[cfg(test)]
mod tests {
    use super::TeamWorkflowManager;
    use crate::error::SecurityLevel;
    use crate::protocol::handler::team_types::{
        DocType, Priority, ReviewRequest, ReviewSeverity, StateTransition, Task, TaskStatus,
        TeamMessage, TeamMessageType, TeamWorkflow, WorkflowFilter, WorkflowStatus,
    };
    use crate::security::manager::SecurityManagerImpl;
    use squirrel_mcp_config::SecurityConfig;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn test_security() -> Arc<SecurityManagerImpl> {
        let cfg = SecurityConfig {
            enable_rbac: false,
            ..Default::default()
        };
        Arc::new(SecurityManagerImpl::new(cfg))
    }

    fn sample_workflow(id: &str) -> TeamWorkflow {
        TeamWorkflow {
            id: id.to_string(),
            name: "n".into(),
            description: "d".into(),
            status: WorkflowStatus::Active,
            security_level: SecurityLevel::Medium,
            permissions: vec![],
            metadata: HashMap::from([("assignee".into(), "alice".into())]),
        }
    }

    #[tokio::test]
    async fn workflow_lifecycle_and_tasks() {
        let mgr = TeamWorkflowManager::new(test_security());
        let token = "tok";
        let wf = sample_workflow("wf1");
        mgr.create_workflow(wf, token).await.expect("create");

        let got = mgr.get_workflow("wf1", token).await.expect("get");
        assert_eq!(got.id, "wf1");

        mgr.update_workflow_status("wf1", WorkflowStatus::Paused, token)
            .await
            .expect("status");
        assert_eq!(
            mgr.get_workflow("wf1", token).await.unwrap().status,
            WorkflowStatus::Paused
        );

        mgr.transition_workflow_state("wf1", WorkflowStatus::Active, "u1", "resume", token)
            .await
            .expect("transition");

        let msg = TeamMessage {
            id: "m1".into(),
            type_: TeamMessageType::Comment,
            content: "hello".into(),
            sender: "alice".into(),
            timestamp: chrono::Utc::now(),
            security_level: SecurityLevel::Low,
            metadata: HashMap::new(),
        };
        mgr.send_message("wf1", msg, token).await.expect("send");
        let messages = mgr.get_messages("wf1", token).await.expect("msgs");
        assert_eq!(messages.len(), 1);

        let review = ReviewRequest {
            id: "r1".into(),
            title: "t".into(),
            description: "d".into(),
            reviewer: "bob".into(),
            severity: ReviewSeverity::Info,
            doc_type: DocType::API,
            priority: Priority::Low,
            security_level: SecurityLevel::Low,
        };
        mgr.create_review(review, token).await.expect("review");
        let r = mgr.get_review("r1", token).await.expect("get rev");
        assert_eq!(r.title, "t");

        let task = Task {
            id: "t1".into(),
            title: "task".into(),
            description: String::new(),
            assignee: "alice".into(),
            status: TaskStatus::Todo,
            priority: Priority::Medium,
            due_date: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        mgr.create_task("wf1", task, token).await.expect("task");
        let tasks = mgr.get_workflow_tasks("wf1", token).await.expect("tasks");
        assert_eq!(tasks.len(), 1);

        mgr.update_task_status("wf1", "t1", TaskStatus::InProgress, token)
            .await
            .expect("upd task");

        assert!(mgr.get_workflow("missing", token).await.is_err());
        assert!(
            mgr.update_workflow_status("missing", WorkflowStatus::Completed, token)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn filter_workflows_and_metrics() {
        let mgr = TeamWorkflowManager::new(test_security());
        let token = "tok";
        let mut wf = sample_workflow("wf2");
        wf.metadata
            .insert("last_updated".into(), chrono::Utc::now().to_rfc3339());
        wf.metadata
            .insert("priority".into(), format!("{:?}", Priority::High));
        mgr.create_workflow(wf, token).await.unwrap();

        let filter = WorkflowFilter {
            status: Some(WorkflowStatus::Active),
            security_level: Some(SecurityLevel::Medium),
            date_range: None,
            assignee: Some("alice".into()),
            priority: Some(Priority::High),
        };
        let list = mgr.filter_workflows(&filter, token).await.unwrap();
        assert_eq!(list.len(), 1);

        let metrics = mgr.get_workflow_metrics("wf2", token).await.unwrap();
        assert_eq!(metrics.completion_rate, 0.0);

        mgr.update_workflow_status("wf2", WorkflowStatus::Completed, token)
            .await
            .unwrap();
        let m2 = mgr.get_workflow_metrics("wf2", token).await.unwrap();
        assert_eq!(m2.completion_rate, 100.0);

        mgr.update_workflow_status("wf2", WorkflowStatus::Failed, token)
            .await
            .unwrap();
        let m3 = mgr.get_workflow_metrics("wf2", token).await.unwrap();
        assert_eq!(m3.completion_rate, 0.0);
    }

    #[tokio::test]
    async fn broadcast_and_task_errors() {
        let mgr = TeamWorkflowManager::new(test_security());
        let token = "tok";
        mgr.create_workflow(sample_workflow("a"), token)
            .await
            .unwrap();
        mgr.create_workflow(sample_workflow("b"), token)
            .await
            .unwrap();

        let msg = TeamMessage {
            id: "bm".into(),
            type_: TeamMessageType::Status,
            content: "x".into(),
            sender: "s".into(),
            timestamp: chrono::Utc::now(),
            security_level: SecurityLevel::Low,
            metadata: HashMap::new(),
        };
        mgr.broadcast_team_message(msg, vec!["a".into(), "b".into()], token)
            .await
            .unwrap();

        let task = Task {
            id: "tx".into(),
            title: "t".into(),
            description: String::new(),
            assignee: "a".into(),
            status: TaskStatus::Todo,
            priority: Priority::Low,
            due_date: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        mgr.create_task("a", task, token).await.unwrap();
        assert!(
            mgr.update_task_status("a", "nope", TaskStatus::Completed, token)
                .await
                .is_err()
        );
        assert!(
            mgr.update_task_status("missing", "tx", TaskStatus::Completed, token)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn transition_workflow_missing_and_metrics_branches() {
        let mgr = TeamWorkflowManager::new(test_security());
        let token = "tok";
        assert!(
            mgr.transition_workflow_state("nope", WorkflowStatus::Active, "u", "r", token,)
                .await
                .is_err()
        );

        let mut wf = sample_workflow("wf-m");
        wf.metadata
            .insert("last_updated".into(), chrono::Utc::now().to_rfc3339());
        let tr = StateTransition {
            from_state: WorkflowStatus::Active,
            to_state: WorkflowStatus::Paused,
            timestamp: chrono::Utc::now(),
            initiator: "u".into(),
            reason: "seed".into(),
        };
        wf.metadata.insert(
            "transition_seed".into(),
            serde_json::to_string(&tr).unwrap(),
        );
        mgr.create_workflow(wf, token).await.unwrap();

        let task = Task {
            id: "t-m1".into(),
            title: "t1".into(),
            description: String::new(),
            assignee: "a".into(),
            status: TaskStatus::Todo,
            priority: Priority::Low,
            due_date: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        mgr.create_task("wf-m", task, token).await.unwrap();
        mgr.update_task_status("wf-m", "t-m1", TaskStatus::Completed, token)
            .await
            .unwrap();

        let review = ReviewRequest {
            id: "rev-m1".into(),
            title: "rev".into(),
            description: String::new(),
            reviewer: "r".into(),
            severity: ReviewSeverity::Info,
            doc_type: DocType::API,
            priority: Priority::Low,
            security_level: SecurityLevel::Low,
        };
        mgr.create_review(review, token).await.unwrap();
        let msg = TeamMessage {
            id: "mrev".into(),
            type_: TeamMessageType::Review,
            content: "rev-m1 review".into(),
            sender: "s".into(),
            timestamp: chrono::Utc::now(),
            security_level: SecurityLevel::Low,
            metadata: HashMap::new(),
        };
        mgr.send_message("wf-m", msg, token).await.unwrap();

        let metrics = mgr.get_workflow_metrics("wf-m", token).await.unwrap();
        assert_eq!(metrics.state_transitions.len(), 1);
        assert!(metrics.review_count >= 1);
        assert!(metrics.completion_rate >= 0.0);
        assert!(metrics.average_review_time >= 0.0);
    }
}
