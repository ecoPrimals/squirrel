// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Task manager for managing tasks and their lifecycle.
//!
//! This module provides a `TaskManager` that handles task creation,
//! retrieval, updates, and assignment. It maintains the state of
//! all tasks in the system.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::task::types::{Task, TaskStatus};

/// Manager for task creation, execution and monitoring.
///
/// The `TaskManager` is responsible for maintaining the state of all tasks
/// in the system, handling their creation, updating, and assignment to agents.
#[derive(Debug)]
pub struct TaskManager {
    /// Map of task IDs to tasks (`Arc<str>` keys for zero-copy)
    tasks: RwLock<HashMap<Arc<str>, Task>>,

    /// Map of agent IDs to task IDs
    agent_tasks: RwLock<HashMap<String, HashSet<Arc<str>>>>,

    /// Map of context IDs to task IDs
    context_tasks: RwLock<HashMap<String, HashSet<Arc<str>>>>,
}

impl TaskManager {
    /// Create a new task manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tasks: RwLock::new(HashMap::new()),
            agent_tasks: RwLock::new(HashMap::new()),
            context_tasks: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new task.
    ///
    /// This creates a new task and adds it to the task manager. It also updates
    /// the context task mapping if a context ID is provided.
    pub async fn create_task(&self, mut task: Task) -> Result<Task> {
        if task.id.is_empty() {
            task.id = Arc::from(Uuid::new_v4().to_string());
        }

        // Update the maps
        let mut tasks = self.tasks.write().await;

        // Check if task with this ID already exists
        if tasks.contains_key(task.id.as_ref()) {
            return Err(Error::AlreadyExists(format!(
                "Task with ID {} already exists",
                task.id.as_ref()
            )));
        }

        // Update the context task mapping
        if let Some(context_id) = &task.context_id {
            let mut context_tasks = self.context_tasks.write().await;
            let tasks_set = context_tasks
                .entry(context_id.clone())
                .or_insert_with(HashSet::new);
            tasks_set.insert(Arc::clone(&task.id));
        }

        let id_key = Arc::clone(&task.id);
        tasks.insert(Arc::clone(&task.id), task);

        Ok(tasks
            .get(id_key.as_ref())
            .ok_or_else(|| Error::NotFound("Task missing immediately after insert".to_string()))?
            .clone())
    }

    /// Get a task by ID.
    pub async fn get_task(&self, id: &str) -> Result<Task> {
        let tasks = self.tasks.read().await;

        tasks
            .get(id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Task with ID {id} not found")))
    }

    /// Update an existing task.
    ///
    /// This updates an existing task in the task manager. It preserves certain
    /// immutable fields like creation time and handles context and agent changes.
    pub async fn update_task(&self, updated_task: Task) -> Result<Task> {
        let mut tasks = self.tasks.write().await;

        // Get the existing task
        let existing_task = tasks.get(updated_task.id.as_ref()).ok_or_else(|| {
            Error::NotFound(format!(
                "Task with ID {} not found",
                updated_task.id.as_ref()
            ))
        })?;

        // Preserve creation time and other immutable fields
        let mut merged_task = updated_task.clone();
        merged_task.created_at = existing_task.created_at;

        // Handle context change
        if existing_task.context_id != merged_task.context_id {
            let mut context_tasks = self.context_tasks.write().await;

            // Remove from old context
            if let Some(old_context_id) = &existing_task.context_id
                && let Some(tasks_set) = context_tasks.get_mut(old_context_id)
            {
                tasks_set.remove(merged_task.id.as_ref());
            }

            // Add to new context
            if let Some(new_context_id) = &merged_task.context_id {
                let tasks_set = context_tasks
                    .entry(new_context_id.clone())
                    .or_insert_with(HashSet::new);
                tasks_set.insert(Arc::clone(&merged_task.id));
            }
        }

        // Handle agent change
        if existing_task.agent_id != merged_task.agent_id {
            let mut agent_tasks = self.agent_tasks.write().await;

            // Remove from old agent
            if let Some(old_agent_id) = &existing_task.agent_id
                && let Some(tasks_set) = agent_tasks.get_mut(old_agent_id)
            {
                tasks_set.remove(merged_task.id.as_ref());
            }

            // Add to new agent
            if let Some(new_agent_id) = &merged_task.agent_id {
                let tasks_set = agent_tasks
                    .entry(new_agent_id.clone())
                    .or_insert_with(HashSet::new);
                tasks_set.insert(Arc::clone(&merged_task.id));
            }
        }

        // Update the task (move merged_task into map; return a single clone for the API)
        tasks.insert(Arc::clone(&merged_task.id), merged_task);

        Ok(tasks
            .get(updated_task.id.as_ref())
            .ok_or_else(|| Error::NotFound("Task missing immediately after update".to_string()))?
            .clone())
    }

    /// Assign a task to an agent.
    pub async fn assign_task(&self, task_id: &str, agent_id: &str) -> Result<Task> {
        // Release any `tasks` lock before awaiting `check_prerequisites`, which takes a read lock
        // on the same map (would deadlock if we held the write lock across the await).
        let task_snapshot = {
            let tasks = self.tasks.read().await;
            tasks
                .get(task_id)
                .ok_or_else(|| Error::NotFound(format!("Task with ID {task_id} not found")))?
                .clone()
        };

        if task_snapshot.status_code != TaskStatus::Pending
            && task_snapshot.status_code != TaskStatus::Waiting
        {
            return Err(Error::InvalidState(format!(
                "Task {} is in state {:?} and cannot be assigned",
                task_id, task_snapshot.status_code
            )));
        }

        let prerequisites_met = self.check_prerequisites(&task_snapshot).await?;
        if !prerequisites_met {
            return Err(Error::InvalidState(format!(
                "Prerequisites for task {task_id} are not all met"
            )));
        }

        let mut tasks = self.tasks.write().await;
        let mut task = tasks
            .get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task with ID {task_id} not found")))?
            .clone();

        if task.status_code != TaskStatus::Pending && task.status_code != TaskStatus::Waiting {
            return Err(Error::InvalidState(format!(
                "Task {} is in state {:?} and cannot be assigned",
                task_id, task.status_code
            )));
        }

        task.mark_running(agent_id);

        let mut agent_tasks = self.agent_tasks.write().await;
        let tasks_set = agent_tasks
            .entry(agent_id.to_string())
            .or_insert_with(HashSet::new);
        tasks_set.insert(Arc::clone(&task.id));

        tasks.insert(Arc::clone(&task.id), task);

        Ok(tasks
            .get(task_id)
            .ok_or_else(|| Error::NotFound("Task missing immediately after assign".to_string()))?
            .clone())
    }

    /// Update the progress of a task.
    pub async fn update_progress(
        &self,
        task_id: &str,
        progress: f32,
        status_message: Option<String>,
    ) -> Result<Task> {
        let mut tasks = self.tasks.write().await;

        // Get the task
        let mut task = tasks
            .get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task with ID {task_id} not found")))?
            .clone();

        // Check if the task is in a valid state to update progress
        if task.status_code != TaskStatus::Running {
            return Err(Error::InvalidState(format!(
                "Task {} is in state {:?} and progress cannot be updated",
                task_id, task.status_code
            )));
        }

        // Update the progress
        task.update_progress(progress, status_message);

        let id_key = Arc::clone(&task.id);
        tasks.insert(Arc::clone(&task.id), task);

        Ok(tasks
            .get(id_key.as_ref())
            .ok_or_else(|| Error::NotFound("Task missing after progress update".to_string()))?
            .clone())
    }

    /// Mark a task as completed.
    pub async fn complete_task(
        &self,
        task_id: &str,
        output_data: Option<HashMap<String, String>>,
    ) -> Result<Task> {
        let mut tasks = self.tasks.write().await;

        // Get the task
        let mut task = tasks
            .get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task with ID {task_id} not found")))?
            .clone();

        // Check if the task is in a valid state to be completed
        if task.status_code != TaskStatus::Running {
            return Err(Error::InvalidState(format!(
                "Task {} is in state {:?} and cannot be completed",
                task_id, task.status_code
            )));
        }

        // Mark the task as completed
        task.mark_completed(output_data);

        tasks.insert(Arc::clone(&task.id), task);

        // Check dependent tasks
        drop(tasks); // Release the lock before calling another method
        self.check_dependent_tasks(task_id).await?;

        // Re-fetch the task to return the latest version
        let tasks = self.tasks.read().await;
        tasks
            .get(task_id)
            .ok_or_else(|| {
                Error::NotFound(format!("Task with ID {task_id} not found after completion"))
            })
            .cloned()
    }

    /// Mark a task as failed.
    pub async fn fail_task(&self, task_id: &str, error_message: &str) -> Result<Task> {
        let mut tasks = self.tasks.write().await;

        // Get the task
        let mut task = tasks
            .get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task with ID {task_id} not found")))?
            .clone();

        // Mark the task as failed
        task.mark_failed(error_message);

        let id_key = Arc::clone(&task.id);
        tasks.insert(Arc::clone(&task.id), task);

        Ok(tasks
            .get(id_key.as_ref())
            .ok_or_else(|| Error::NotFound("Task missing after fail".to_string()))?
            .clone())
    }

    /// Cancel a task.
    pub async fn cancel_task(&self, task_id: &str, reason: &str) -> Result<Task> {
        let mut tasks = self.tasks.write().await;

        // Get the task
        let mut task = tasks
            .get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task with ID {task_id} not found")))?
            .clone();

        // Check if the task is in a valid state to be cancelled
        if task.is_finished() {
            return Err(Error::InvalidState(format!(
                "Task {} is already in terminal state {:?} and cannot be cancelled",
                task_id, task.status_code
            )));
        }

        // Mark the task as cancelled
        task.mark_cancelled(reason);

        let id_key = Arc::clone(&task.id);
        tasks.insert(Arc::clone(&task.id), task);

        Ok(tasks
            .get(id_key.as_ref())
            .ok_or_else(|| Error::NotFound("Task missing after cancel".to_string()))?
            .clone())
    }

    /// Get all tasks assigned to a specific agent.
    pub async fn get_agent_tasks(&self, agent_id: &str) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().await;
        let agent_tasks = self.agent_tasks.read().await;

        if let Some(task_ids) = agent_tasks.get(agent_id) {
            let agent_tasks: Vec<Task> = task_ids
                .iter()
                .filter_map(|task_id| tasks.get(task_id.as_ref()).cloned())
                .collect();
            Ok(agent_tasks)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get all tasks associated with a specific context.
    pub async fn get_context_tasks(&self, context_id: &str) -> Result<Vec<Task>> {
        let context_tasks = self.context_tasks.read().await;
        let tasks = self.tasks.read().await;

        let task_ids = context_tasks.get(context_id).cloned().unwrap_or_default();

        let result: Vec<Task> = task_ids
            .iter()
            .filter_map(|id| tasks.get(id.as_ref()).cloned())
            .collect();

        Ok(result)
    }

    /// Get tasks by status.
    pub async fn get_tasks_by_status(&self, status: TaskStatus) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().await;

        let result: Vec<Task> = tasks
            .values()
            .filter(|task| task.status_code == status)
            .cloned()
            .collect();

        Ok(result)
    }

    /// Check if all prerequisites for a task are met.
    pub async fn check_prerequisites(&self, task: &Task) -> Result<bool> {
        if task.prerequisites.is_empty() {
            return Ok(true);
        }

        let tasks = self.tasks.read().await;

        for prereq_id in &task.prerequisites {
            if let Some(prereq_task) = tasks.get(prereq_id.as_str()) {
                if !prereq_task.is_completed() {
                    return Ok(false);
                }
            } else {
                // Prerequisite task not found
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Find tasks that are ready to be assigned to agents.
    ///
    /// This method finds tasks that are in the Pending state and have all their
    /// prerequisites met. These tasks are candidates for assignment to agents.
    pub async fn find_assignable_tasks(&self) -> Result<Vec<Task>> {
        let tasks_guard = self.tasks.read().await;

        // First collect all pending tasks
        let pending_tasks: Vec<Task> = tasks_guard
            .values()
            .filter(|task| task.status_code == TaskStatus::Pending)
            .cloned()
            .collect();

        // Drop the lock before processing task prerequisites
        drop(tasks_guard);

        let mut assignable_tasks = Vec::new();
        let pending_len = pending_tasks.len();

        // Check each task's prerequisites
        for task in pending_tasks {
            let prerequisites_met = self.check_prerequisites(&task).await?;

            if prerequisites_met {
                assignable_tasks.push(task);
            }
        }

        debug!(
            "Found {} assignable tasks out of {} pending tasks",
            assignable_tasks.len(),
            pending_len
        );

        Ok(assignable_tasks)
    }

    /// Update the status of tasks that depend on a completed task.
    ///
    /// This is called internally when a task is completed to check if any
    /// dependent tasks can now be transitioned to the Pending state.
    async fn check_dependent_tasks(&self, completed_task_id: &str) -> Result<()> {
        let tasks = self.tasks.read().await;

        // Find tasks that have the completed task as a prerequisite
        let dependent_tasks: Vec<Task> = tasks
            .values()
            .filter(|task| {
                task.prerequisites.contains(&completed_task_id.to_string())
                    && task.status_code == TaskStatus::Waiting
            })
            .cloned()
            .collect();

        drop(tasks); // Release the lock

        // Update each dependent task if all prerequisites are now met
        for mut task in dependent_tasks {
            let prerequisites_met = self.check_prerequisites(&task).await?;

            if prerequisites_met {
                // Update to Pending state
                task.status_code = TaskStatus::Pending;
                task.updated_at = chrono::Utc::now();

                let mut tasks = self.tasks.write().await;
                tasks.insert(Arc::clone(&task.id), task);
            }
        }

        Ok(())
    }

    /// List all tasks for a specific agent (alias for `get_agent_tasks` for compatibility)
    pub async fn list_tasks(&self, agent_id: Option<&str>) -> Result<Vec<Task>> {
        if let Some(agent_id) = agent_id {
            self.get_agent_tasks(agent_id).await
        } else {
            // Return all tasks if no agent specified
            let tasks = self.tasks.read().await;
            Ok(tasks.values().cloned().collect())
        }
    }

    /// Update task progress (alias for `update_progress` for compatibility)
    pub async fn update_task_progress(
        &self,
        task_id: &str,
        progress: f32,
        status_message: &str,
    ) -> Result<Task> {
        self.update_progress(task_id, progress, Some(status_message.to_string()))
            .await
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::types::{Task, TaskStatus};

    #[tokio::test]
    async fn create_get_update_list_and_duplicate_id_error() {
        let mgr = TaskManager::new();
        let t = Task::new("job", "do work");
        let created = mgr.create_task(t.clone()).await.unwrap();
        assert_eq!(created.name.as_ref(), "job");
        let got = mgr.get_task(created.id.as_ref()).await.unwrap();
        assert_eq!(got.id, created.id);

        let mut upd = got.clone();
        upd.description = "updated".into();
        mgr.update_task(upd).await.unwrap();

        let all = mgr.list_tasks(None).await.unwrap();
        assert_eq!(all.len(), 1);

        let mut dup = Task::new("x", "y");
        dup.id = Arc::clone(&created.id);
        let err = mgr.create_task(dup).await.unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[tokio::test]
    async fn assign_progress_complete_lifecycle() {
        let mgr = TaskManager::new();
        let t = Task::new("run", "go");
        let created = mgr.create_task(t).await.unwrap();
        let id = created.id.as_ref().to_string();
        mgr.assign_task(&id, "agent-a").await.unwrap();
        mgr.update_task_progress(&id, 33.0, "third").await.unwrap();
        mgr.complete_task(&id, None).await.unwrap();
        let done = mgr.get_task(&id).await.unwrap();
        assert_eq!(done.status_code, TaskStatus::Completed);
    }

    #[tokio::test]
    async fn cancel_pending_task_and_get_not_found() {
        let mgr = TaskManager::new();
        let t = Task::new("c", "cancel me");
        let created = mgr.create_task(t).await.unwrap();
        let id = created.id.as_ref();
        mgr.cancel_task(id, "because").await.unwrap();
        let cancelled = mgr.get_task(id).await.unwrap();
        assert_eq!(cancelled.status_code, TaskStatus::Cancelled);

        let err = mgr.get_task("missing-id").await.unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[tokio::test]
    async fn assign_invalid_state_and_progress_wrong_state_return_errors() {
        let mgr = TaskManager::new();
        let t = Task::new("e", "err");
        let created = mgr.create_task(t).await.unwrap();
        let id = created.id.as_ref().to_string();
        mgr.assign_task(&id, "a1").await.unwrap();
        let e2 = mgr.assign_task(&id, "a2").await.unwrap_err();
        assert!(e2.to_string().contains("cannot be assigned"));

        let pending = Task::new("p", "p");
        let p = mgr.create_task(pending).await.unwrap();
        let pid = p.id.as_ref().to_string();
        let pe = mgr.update_task_progress(&pid, 1.0, "x").await.unwrap_err();
        assert!(pe.to_string().contains("progress"));
    }

    #[tokio::test]
    async fn context_and_agent_indexes_update_on_create_and_task_moves() {
        let mgr = TaskManager::new();
        let t = Task::new("c", "with ctx").with_context("ctx-a");
        let created = mgr.create_task(t).await.unwrap();
        let ctx_tasks = mgr.get_context_tasks("ctx-a").await.unwrap();
        assert_eq!(ctx_tasks.len(), 1);

        let mut moved = mgr.get_task(created.id.as_ref()).await.unwrap();
        moved.context_id = Some("ctx-b".into());
        mgr.update_task(moved).await.unwrap();
        assert!(mgr.get_context_tasks("ctx-a").await.unwrap().is_empty());
        assert_eq!(mgr.get_context_tasks("ctx-b").await.unwrap().len(), 1);

        let t2 = Task::new("agented", "x");
        let c2 = mgr.create_task(t2).await.unwrap();
        mgr.assign_task(c2.id.as_ref(), "agent-1").await.unwrap();
        assert_eq!(mgr.get_agent_tasks("agent-1").await.unwrap().len(), 1);

        let mut reassigned = mgr.get_task(c2.id.as_ref()).await.unwrap();
        assert!(
            mgr.assign_task(reassigned.id.as_ref(), "agent-2")
                .await
                .is_err()
        );
        reassigned.status_code = TaskStatus::Pending;
        reassigned.agent_id = None;
        mgr.update_task(reassigned).await.unwrap();
        mgr.assign_task(c2.id.as_ref(), "agent-2").await.unwrap();
        assert!(mgr.get_agent_tasks("agent-1").await.unwrap().is_empty());
        assert_eq!(mgr.get_agent_tasks("agent-2").await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn prerequisites_block_assign_until_complete_then_dependent_unblocks() {
        let mgr = TaskManager::new();
        let pre = Task::new("pre", "first");
        let pre_created = mgr.create_task(pre).await.unwrap();
        let pre_id = pre_created.id.as_ref().to_string();

        let mut dep = Task::new("dep", "after pre");
        dep.prerequisites = vec![pre_id.clone()];
        dep.status_code = TaskStatus::Waiting;
        let dep_created = mgr.create_task(dep).await.unwrap();

        let err = mgr
            .assign_task(dep_created.id.as_ref(), "a")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("Prerequisites"));

        mgr.assign_task(&pre_id, "a").await.unwrap();
        mgr.complete_task(&pre_id, None).await.unwrap();

        let dep_after = mgr.get_task(dep_created.id.as_ref()).await.unwrap();
        assert_eq!(dep_after.status_code, TaskStatus::Pending);
        assert!(mgr.check_prerequisites(&dep_after).await.unwrap());
        assert!(
            mgr.find_assignable_tasks()
                .await
                .unwrap()
                .iter()
                .any(|t| t.id.as_ref() == dep_created.id.as_ref())
        );
    }

    #[tokio::test]
    async fn get_tasks_by_status_and_fail_task() {
        let mgr = TaskManager::new();
        let t = Task::new("f", "fail me");
        let c = mgr.create_task(t).await.unwrap();
        let id = c.id.as_ref().to_string();
        mgr.fail_task(&id, "boom").await.unwrap();
        let failed = mgr.get_tasks_by_status(TaskStatus::Failed).await.unwrap();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].id.as_ref(), id);
    }

    #[tokio::test]
    async fn complete_and_cancel_invalid_states() {
        let mgr = TaskManager::new();
        let t = Task::new("p", "pending");
        let c = mgr.create_task(t).await.unwrap();
        let id = c.id.as_ref().to_string();

        let ce = mgr.complete_task(&id, None).await.unwrap_err();
        assert!(ce.to_string().contains("cannot be completed"));

        mgr.assign_task(&id, "ag").await.unwrap();
        mgr.cancel_task(&id, "stop").await.unwrap();

        let t2 = Task::new("p2", "run to done");
        let c2 = mgr.create_task(t2).await.unwrap();
        let id2 = c2.id.as_ref().to_string();
        mgr.assign_task(&id2, "ag2").await.unwrap();
        mgr.complete_task(&id2, None).await.unwrap();
        let ce3 = mgr.cancel_task(&id2, "late").await.unwrap_err();
        assert!(ce3.to_string().contains("terminal"));
    }

    #[tokio::test]
    async fn check_prerequisites_missing_prereq_task_returns_false() {
        let mgr = TaskManager::new();
        let mut t = Task::new("x", "y");
        t.prerequisites = vec!["nonexistent".into()];
        assert!(!mgr.check_prerequisites(&t).await.unwrap());
    }

    #[tokio::test]
    async fn list_tasks_with_agent_and_without() {
        let mgr = TaskManager::new();
        let a = mgr.create_task(Task::new("a", "")).await.unwrap();
        mgr.assign_task(a.id.as_ref(), "z").await.unwrap();
        let list_z = mgr.list_tasks(Some("z")).await.unwrap();
        assert_eq!(list_z.len(), 1);
        let all = mgr.list_tasks(None).await.unwrap();
        assert_eq!(all.len(), 1);
    }
}
