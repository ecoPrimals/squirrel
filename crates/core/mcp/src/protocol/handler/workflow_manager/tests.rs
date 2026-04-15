// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

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
        mgr.get_workflow("wf1", token)
            .await
            .expect("should succeed")
            .status,
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
    mgr.create_workflow(wf, token)
        .await
        .expect("should succeed");

    let filter = WorkflowFilter {
        status: Some(WorkflowStatus::Active),
        security_level: Some(SecurityLevel::Medium),
        date_range: None,
        assignee: Some("alice".into()),
        priority: Some(Priority::High),
    };
    let list = mgr
        .filter_workflows(&filter, token)
        .await
        .expect("should succeed");
    assert_eq!(list.len(), 1);

    let metrics = mgr
        .get_workflow_metrics("wf2", token)
        .await
        .expect("should succeed");
    assert_eq!(metrics.completion_rate, 0.0);

    mgr.update_workflow_status("wf2", WorkflowStatus::Completed, token)
        .await
        .expect("should succeed");
    let m2 = mgr
        .get_workflow_metrics("wf2", token)
        .await
        .expect("should succeed");
    assert_eq!(m2.completion_rate, 100.0);

    mgr.update_workflow_status("wf2", WorkflowStatus::Failed, token)
        .await
        .expect("should succeed");
    let m3 = mgr
        .get_workflow_metrics("wf2", token)
        .await
        .expect("should succeed");
    assert_eq!(m3.completion_rate, 0.0);
}

#[tokio::test]
async fn broadcast_and_task_errors() {
    let mgr = TeamWorkflowManager::new(test_security());
    let token = "tok";
    mgr.create_workflow(sample_workflow("a"), token)
        .await
        .expect("should succeed");
    mgr.create_workflow(sample_workflow("b"), token)
        .await
        .expect("should succeed");

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
        .expect("should succeed");

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
    mgr.create_task("a", task, token)
        .await
        .expect("should succeed");
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
        serde_json::to_string(&tr).expect("should succeed"),
    );
    mgr.create_workflow(wf, token)
        .await
        .expect("should succeed");

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
    mgr.create_task("wf-m", task, token)
        .await
        .expect("should succeed");
    mgr.update_task_status("wf-m", "t-m1", TaskStatus::Completed, token)
        .await
        .expect("should succeed");

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
    mgr.create_review(review, token)
        .await
        .expect("should succeed");
    let msg = TeamMessage {
        id: "mrev".into(),
        type_: TeamMessageType::Review,
        content: "rev-m1 review".into(),
        sender: "s".into(),
        timestamp: chrono::Utc::now(),
        security_level: SecurityLevel::Low,
        metadata: HashMap::new(),
    };
    mgr.send_message("wf-m", msg, token)
        .await
        .expect("should succeed");

    let metrics = mgr
        .get_workflow_metrics("wf-m", token)
        .await
        .expect("should succeed");
    assert_eq!(metrics.state_transitions.len(), 1);
    assert!(metrics.review_count >= 1);
    assert!(metrics.completion_rate >= 0.0);
    assert!(metrics.average_review_time >= 0.0);
}
