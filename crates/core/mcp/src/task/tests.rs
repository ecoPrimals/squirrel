// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

use super::types::{AgentType, Task, TaskPriority, TaskStatus};

#[test]
fn task_creation() {
    let task = Task::new("test-task", "A test task");
    assert_eq!(task.name, "test-task");
    assert_eq!(task.description, "A test task");
    assert_eq!(task.status, TaskStatus::Pending);
    assert_eq!(task.priority_code, TaskPriority::Medium);
    assert_eq!(task.agent_type, AgentType::Generic);
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
    assert_eq!(task.status, TaskStatus::Pending);

    task.status = TaskStatus::InProgress;
    assert_eq!(task.status, TaskStatus::InProgress);

    task.status = TaskStatus::Completed;
    assert_eq!(task.status, TaskStatus::Completed);
}
