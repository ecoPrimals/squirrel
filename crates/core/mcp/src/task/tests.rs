// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use super::types::{AgentType, Task, TaskPriority, TaskStatus};

#[test]
fn task_creation() {
    let task = Task::new("test-task", "A test task");
    assert_eq!(task.name.as_ref(), "test-task");
    assert_eq!(task.description, "A test task");
    assert_eq!(task.status_code, TaskStatus::Pending);
    assert_eq!(task.priority_code, TaskPriority::Medium);
    assert_eq!(task.agent_type, AgentType::Unspecified);
    assert!(!task.id.is_empty());
}

#[test]
fn task_priority_ordering() {
    assert!(TaskPriority::Critical > TaskPriority::High);
    assert!(TaskPriority::High > TaskPriority::Medium);
    assert!(TaskPriority::Medium > TaskPriority::Low);
}

#[test]
fn task_status_transitions() {
    let mut task = Task::new("status-test", "Testing status");
    assert_eq!(task.status_code, TaskStatus::Pending);

    task.status_code = TaskStatus::Running;
    assert_eq!(task.status_code, TaskStatus::Running);

    task.status_code = TaskStatus::Completed;
    assert_eq!(task.status_code, TaskStatus::Completed);
}
