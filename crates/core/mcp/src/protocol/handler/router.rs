// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Router module - public API for protocol handler.
//!
//! Re-exports team workflow types, manager, and message router for backward compatibility.

pub use super::team_types::*;

#[cfg(test)]
mod tests {
    use super::super::workflow_manager::TeamWorkflowManager;
    use super::*;
    use crate::error::SecurityLevel;
    use crate::security::manager::SecurityManagerImpl;
    use chrono::Duration;
    use squirrel_mcp_config::SecurityConfig;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn setup_test_environment() -> (TeamWorkflowManager, String) {
        let config = SecurityConfig {
            enable_rbac: false,
            ..SecurityConfig::default()
        };
        let security = Arc::new(SecurityManagerImpl::new(config));
        let token = "test_token_123";
        (TeamWorkflowManager::new(security), token.to_string())
    }

    #[tokio::test]
    async fn test_workflow_lifecycle() {
        let (manager, token) = setup_test_environment();

        let workflow = TeamWorkflow {
            id: "test_workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "Test workflow description".to_string(),
            status: WorkflowStatus::Active,
            security_level: SecurityLevel::High,
            permissions: vec![],
            metadata: HashMap::new(),
        };
        manager
            .create_workflow(workflow.clone(), &token)
            .await
            .expect("should succeed");

        let retrieved = manager
            .get_workflow("test_workflow", &token)
            .await
            .expect("should succeed");
        assert_eq!(retrieved.id, workflow.id);

        manager
            .update_workflow_status("test_workflow", WorkflowStatus::Completed, &token)
            .await
            .expect("should succeed");
        let updated = manager
            .get_workflow("test_workflow", &token)
            .await
            .expect("should succeed");
        assert!(matches!(updated.status, WorkflowStatus::Completed));
    }

    #[tokio::test]
    async fn test_message_handling() {
        let (manager, token) = setup_test_environment();

        let message = TeamMessage {
            id: "test_message".to_string(),
            type_: TeamMessageType::Task,
            content: "Test message".to_string(),
            sender: "test_user".to_string(),
            timestamp: chrono::Utc::now(),
            security_level: SecurityLevel::High,
            metadata: HashMap::new(),
        };
        manager
            .send_message("test_workflow", message.clone(), &token)
            .await
            .expect("should succeed");

        let messages = manager
            .get_messages("test_workflow", &token)
            .await
            .expect("should succeed");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, message.id);
    }

    #[tokio::test]
    async fn test_review_lifecycle() {
        let (manager, token) = setup_test_environment();

        let review = ReviewRequest {
            id: "test_review".to_string(),
            title: "Test Review".to_string(),
            description: "Test review description".to_string(),
            reviewer: "test_reviewer".to_string(),
            severity: ReviewSeverity::Critical,
            doc_type: DocType::Code,
            priority: Priority::High,
            security_level: SecurityLevel::High,
        };
        manager
            .create_review(review.clone(), &token)
            .await
            .expect("should succeed");

        let retrieved = manager
            .get_review("test_review", &token)
            .await
            .expect("should succeed");
        assert_eq!(retrieved.id, review.id);
    }

    #[tokio::test]
    async fn test_workflow_state_transition() {
        let (manager, token) = setup_test_environment();

        let workflow = TeamWorkflow {
            id: "test_workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "Test workflow description".to_string(),
            status: WorkflowStatus::Active,
            security_level: SecurityLevel::High,
            permissions: vec![],
            metadata: HashMap::new(),
        };
        manager
            .create_workflow(workflow, &token)
            .await
            .expect("should succeed");

        manager
            .transition_workflow_state(
                "test_workflow",
                WorkflowStatus::Paused,
                "test_user",
                "Pausing for review",
                &token,
            )
            .await
            .expect("should succeed");

        let updated = manager
            .get_workflow("test_workflow", &token)
            .await
            .expect("should succeed");
        assert!(matches!(updated.status, WorkflowStatus::Paused));
    }

    #[tokio::test]
    async fn test_task_management() {
        let (manager, token) = setup_test_environment();

        let workflow = TeamWorkflow {
            id: "test_workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "Test workflow description".to_string(),
            status: WorkflowStatus::Active,
            security_level: SecurityLevel::High,
            permissions: vec![],
            metadata: HashMap::new(),
        };
        manager
            .create_workflow(workflow, &token)
            .await
            .expect("should succeed");

        let task = Task {
            id: "test_task".to_string(),
            title: "Test Task".to_string(),
            description: "Test task description".to_string(),
            assignee: "test_user".to_string(),
            status: TaskStatus::Todo,
            priority: Priority::High,
            due_date: Some(chrono::Utc::now() + Duration::days(1)),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        manager
            .create_task("test_workflow", task.clone(), &token)
            .await
            .expect("should succeed");

        manager
            .update_task_status("test_workflow", "test_task", TaskStatus::InProgress, &token)
            .await
            .expect("should succeed");

        let tasks = manager
            .get_workflow_tasks("test_workflow", &token)
            .await
            .expect("should succeed");
        assert_eq!(tasks.len(), 1);
        assert!(matches!(tasks[0].status, TaskStatus::InProgress));
    }
}
