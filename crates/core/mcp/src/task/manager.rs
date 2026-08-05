// SPDX-License-Identifier: AGPL-3.0-or-later
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
///
/// Tasks are stored as `Arc<Task>` for cheap sharing across API boundaries.
/// Mutations use `Arc::make_mut` (copy-on-write) — zero-cost when uniquely owned.
#[derive(Debug)]
pub struct TaskManager {
    /// Map of task IDs to tasks (`Arc<Task>` for zero-copy reads)
    tasks: RwLock<HashMap<Arc<str>, Arc<Task>>>,

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
    pub async fn create_task(&self, mut task: Task) -> Result<Arc<Task>> {
        if task.id.is_empty() {
            task.id = Arc::from(Uuid::new_v4().to_string());
        }

        let mut tasks = self.tasks.write().await;

        if tasks.contains_key(task.id.as_ref()) {
            return Err(Error::AlreadyExists(format!(
                "Task with ID {} already exists",
                task.id.as_ref()
            )));
        }

        if let Some(context_id) = &task.context_id {
            let mut context_tasks = self.context_tasks.write().await;
            context_tasks
                .entry(context_id.clone())
                .or_default()
                .insert(Arc::clone(&task.id));
        }

        let id_key = Arc::clone(&task.id);
        let task = Arc::new(task);
        tasks.insert(id_key, Arc::clone(&task));

        Ok(task)
    }

    /// Get a task by ID.
    pub async fn get_task(&self, id: &str) -> Result<Arc<Task>> {
        let tasks = self.tasks.read().await;

        tasks
            .get(id)
            .map(Arc::clone)
            .ok_or_else(|| Error::NotFound(format!("Task with ID {id} not found")))
    }

    /// Update an existing task.
    ///
    /// This updates an existing task in the task manager. It preserves certain
    /// immutable fields like creation time and handles context and agent changes.
    pub async fn update_task(&self, updated_task: Task) -> Result<Arc<Task>> {
        let mut tasks = self.tasks.write().await;

        let existing = tasks.get(updated_task.id.as_ref()).ok_or_else(|| {
            Error::NotFound(format!(
                "Task with ID {} not found",
                updated_task.id.as_ref()
            ))
        })?;

        let saved_created_at = existing.created_at;
        let old_context = existing.context_id.clone();
        let old_agent = existing.agent_id.clone();

        let mut merged_task = updated_task;
        merged_task.created_at = saved_created_at;

        if old_context != merged_task.context_id {
            let mut context_tasks = self.context_tasks.write().await;

            if let Some(old_context_id) = &old_context
                && let Some(tasks_set) = context_tasks.get_mut(old_context_id)
            {
                tasks_set.remove(merged_task.id.as_ref());
            }

            if let Some(new_context_id) = &merged_task.context_id {
                context_tasks
                    .entry(new_context_id.clone())
                    .or_default()
                    .insert(Arc::clone(&merged_task.id));
            }
        }

        if old_agent != merged_task.agent_id {
            let mut agent_tasks = self.agent_tasks.write().await;

            if let Some(old_agent_id) = &old_agent
                && let Some(tasks_set) = agent_tasks.get_mut(old_agent_id)
            {
                tasks_set.remove(merged_task.id.as_ref());
            }

            if let Some(new_agent_id) = &merged_task.agent_id {
                agent_tasks
                    .entry(new_agent_id.clone())
                    .or_default()
                    .insert(Arc::clone(&merged_task.id));
            }
        }

        let id_key = Arc::clone(&merged_task.id);
        let task = Arc::new(merged_task);
        tasks.insert(id_key, Arc::clone(&task));

        Ok(task)
    }

    /// Assign a task to an agent.
    pub async fn assign_task(&self, task_id: &str, agent_id: &str) -> Result<Arc<Task>> {
        // Cheap Arc snapshot — release read lock before check_prerequisites (avoids deadlock).
        let snapshot = {
            let tasks = self.tasks.read().await;
            Arc::clone(
                tasks
                    .get(task_id)
                    .ok_or_else(|| Error::NotFound(format!("Task with ID {task_id} not found")))?,
            )
        };

        if snapshot.status_code != TaskStatus::Pending
            && snapshot.status_code != TaskStatus::Waiting
        {
            return Err(Error::InvalidState(format!(
                "Task {} is in state {:?} and cannot be assigned",
                task_id, snapshot.status_code
            )));
        }

        let prerequisites_met = self.check_prerequisites(&snapshot).await?;
        if !prerequisites_met {
            return Err(Error::InvalidState(format!(
                "Prerequisites for task {task_id} are not all met"
            )));
        }

        let mut tasks = self.tasks.write().await;
        let task_arc = tasks
            .get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task with ID {task_id} not found")))?;

        if task_arc.status_code != TaskStatus::Pending
            && task_arc.status_code != TaskStatus::Waiting
        {
            return Err(Error::InvalidState(format!(
                "Task {} is in state {:?} and cannot be assigned",
                task_id, task_arc.status_code
            )));
        }

        let task = Arc::make_mut(task_arc);
        task.mark_running(agent_id);
        let task_id_arc = Arc::clone(&task.id);

        self.agent_tasks
            .write()
            .await
            .entry(agent_id.to_string())
            .or_default()
            .insert(Arc::clone(&task_id_arc));

        Ok(Arc::clone(tasks.get(task_id).unwrap()))
    }

    /// Update the progress of a task.
    pub async fn update_progress(
        &self,
        task_id: &str,
        progress: f32,
        status_message: Option<String>,
    ) -> Result<Arc<Task>> {
        let mut tasks = self.tasks.write().await;

        let task_arc = tasks
            .get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task with ID {task_id} not found")))?;

        if task_arc.status_code != TaskStatus::Running {
            return Err(Error::InvalidState(format!(
                "Task {} is in state {:?} and progress cannot be updated",
                task_id, task_arc.status_code
            )));
        }

        Arc::make_mut(task_arc).update_progress(progress, status_message);

        Ok(Arc::clone(tasks.get(task_id).unwrap()))
    }

    /// Mark a task as completed.
    pub async fn complete_task(
        &self,
        task_id: &str,
        output_data: Option<HashMap<String, String>>,
    ) -> Result<Arc<Task>> {
        {
            let mut tasks = self.tasks.write().await;

            let task_arc = tasks
                .get_mut(task_id)
                .ok_or_else(|| Error::NotFound(format!("Task with ID {task_id} not found")))?;

            if task_arc.status_code != TaskStatus::Running {
                return Err(Error::InvalidState(format!(
                    "Task {} is in state {:?} and cannot be completed",
                    task_id, task_arc.status_code
                )));
            }

            Arc::make_mut(task_arc).mark_completed(output_data);
        }

        self.check_dependent_tasks(task_id).await?;

        let tasks = self.tasks.read().await;
        tasks
            .get(task_id)
            .map(Arc::clone)
            .ok_or_else(|| {
                Error::NotFound(format!("Task with ID {task_id} not found after completion"))
            })
    }

    /// Mark a task as failed.
    pub async fn fail_task(&self, task_id: &str, error_message: &str) -> Result<Arc<Task>> {
        let mut tasks = self.tasks.write().await;

        let task_arc = tasks
            .get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task with ID {task_id} not found")))?;

        Arc::make_mut(task_arc).mark_failed(error_message);

        Ok(Arc::clone(tasks.get(task_id).unwrap()))
    }

    /// Cancel a task.
    pub async fn cancel_task(&self, task_id: &str, reason: &str) -> Result<Arc<Task>> {
        let mut tasks = self.tasks.write().await;

        let task_arc = tasks
            .get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task with ID {task_id} not found")))?;

        if task_arc.is_finished() {
            return Err(Error::InvalidState(format!(
                "Task {} is already in terminal state {:?} and cannot be cancelled",
                task_id, task_arc.status_code
            )));
        }

        Arc::make_mut(task_arc).mark_cancelled(reason);

        Ok(Arc::clone(tasks.get(task_id).unwrap()))
    }

    /// Get all tasks assigned to a specific agent.
    pub async fn get_agent_tasks(&self, agent_id: &str) -> Result<Vec<Arc<Task>>> {
        let tasks = self.tasks.read().await;
        let agent_tasks = self.agent_tasks.read().await;

        if let Some(task_ids) = agent_tasks.get(agent_id) {
            let agent_tasks: Vec<Arc<Task>> = task_ids
                .iter()
                .filter_map(|task_id| tasks.get(task_id.as_ref()).map(Arc::clone))
                .collect();
            Ok(agent_tasks)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get all tasks associated with a specific context.
    pub async fn get_context_tasks(&self, context_id: &str) -> Result<Vec<Arc<Task>>> {
        let context_tasks = self.context_tasks.read().await;
        let tasks = self.tasks.read().await;

        let Some(task_ids) = context_tasks.get(context_id) else {
            return Ok(Vec::new());
        };

        let result: Vec<Arc<Task>> = task_ids
            .iter()
            .filter_map(|id| tasks.get(id.as_ref()).map(Arc::clone))
            .collect();

        Ok(result)
    }

    /// Get tasks by status.
    pub async fn get_tasks_by_status(&self, status: TaskStatus) -> Result<Vec<Arc<Task>>> {
        let tasks = self.tasks.read().await;

        let result: Vec<Arc<Task>> = tasks
            .values()
            .filter(|task| task.status_code == status)
            .map(Arc::clone)
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
    pub async fn find_assignable_tasks(&self) -> Result<Vec<Arc<Task>>> {
        let pending_tasks: Vec<Arc<Task>> = {
            let tasks_guard = self.tasks.read().await;
            tasks_guard
                .values()
                .filter(|task| task.status_code == TaskStatus::Pending)
                .map(Arc::clone)
                .collect()
        };

        let mut assignable_tasks = Vec::new();
        let pending_len = pending_tasks.len();

        for task in pending_tasks {
            if self.check_prerequisites(&task).await? {
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
        let dependent_ids: Vec<Arc<str>> = {
            let tasks = self.tasks.read().await;
            tasks
                .iter()
                .filter(|(_, task)| {
                    task.prerequisites.contains(&completed_task_id.to_string())
                        && task.status_code == TaskStatus::Waiting
                })
                .map(|(id, _)| Arc::clone(id))
                .collect()
        };

        for dep_id in dependent_ids {
            let snapshot = {
                let tasks = self.tasks.read().await;
                tasks.get(dep_id.as_ref()).map(Arc::clone)
            };
            let Some(snapshot) = snapshot else {
                continue;
            };

            if self.check_prerequisites(&snapshot).await? {
                let mut tasks = self.tasks.write().await;
                if let Some(task_arc) = tasks.get_mut(dep_id.as_ref()) {
                    let task = Arc::make_mut(task_arc);
                    task.status_code = TaskStatus::Pending;
                    task.updated_at = chrono::Utc::now();
                }
            }
        }

        Ok(())
    }

    /// List all tasks for a specific agent (alias for `get_agent_tasks` for compatibility)
    pub async fn list_tasks(&self, agent_id: Option<&str>) -> Result<Vec<Arc<Task>>> {
        if let Some(agent_id) = agent_id {
            self.get_agent_tasks(agent_id).await
        } else {
            let tasks = self.tasks.read().await;
            Ok(tasks.values().map(Arc::clone).collect())
        }
    }

    /// Update task progress (alias for `update_progress` for compatibility)
    pub async fn update_task_progress(
        &self,
        task_id: &str,
        progress: f32,
        status_message: &str,
    ) -> Result<Arc<Task>> {
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
        let created = mgr.create_task(t.clone()).await.expect("should succeed");
        assert_eq!(created.name.as_ref(), "job");
        let got = mgr
            .get_task(created.id.as_ref())
            .await
            .expect("should succeed");
        assert_eq!(got.id, created.id);

        let mut upd = Task::clone(&got);
        upd.description = "updated".into();
        mgr.update_task(upd).await.expect("should succeed");

        let all = mgr.list_tasks(None).await.expect("should succeed");
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
        let created = mgr.create_task(t).await.expect("should succeed");
        let id = created.id.as_ref().to_string();
        mgr.assign_task(&id, "agent-a")
            .await
            .expect("should succeed");
        mgr.update_task_progress(&id, 33.0, "third")
            .await
            .expect("should succeed");
        mgr.complete_task(&id, None).await.expect("should succeed");
        let done = mgr.get_task(&id).await.expect("should succeed");
        assert_eq!(done.status_code, TaskStatus::Completed);
    }

    #[tokio::test]
    async fn cancel_pending_task_and_get_not_found() {
        let mgr = TaskManager::new();
        let t = Task::new("c", "cancel me");
        let created = mgr.create_task(t).await.expect("should succeed");
        let id = created.id.as_ref();
        mgr.cancel_task(id, "because")
            .await
            .expect("should succeed");
        let cancelled = mgr.get_task(id).await.expect("should succeed");
        assert_eq!(cancelled.status_code, TaskStatus::Cancelled);

        let err = mgr.get_task("missing-id").await.unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[tokio::test]
    async fn assign_invalid_state_and_progress_wrong_state_return_errors() {
        let mgr = TaskManager::new();
        let t = Task::new("e", "err");
        let created = mgr.create_task(t).await.expect("should succeed");
        let id = created.id.as_ref().to_string();
        mgr.assign_task(&id, "a1").await.expect("should succeed");
        let e2 = mgr.assign_task(&id, "a2").await.unwrap_err();
        assert!(e2.to_string().contains("cannot be assigned"));

        let pending = Task::new("p", "p");
        let p = mgr.create_task(pending).await.expect("should succeed");
        let pid = p.id.as_ref().to_string();
        let pe = mgr.update_task_progress(&pid, 1.0, "x").await.unwrap_err();
        assert!(pe.to_string().contains("progress"));
    }

    #[tokio::test]
    async fn context_and_agent_indexes_update_on_create_and_task_moves() {
        let mgr = TaskManager::new();
        let t = Task::new("c", "with ctx").with_context("ctx-a");
        let created = mgr.create_task(t).await.expect("should succeed");
        let ctx_tasks = mgr
            .get_context_tasks("ctx-a")
            .await
            .expect("should succeed");
        assert_eq!(ctx_tasks.len(), 1);

        let mut moved = Task::clone(
            &mgr.get_task(created.id.as_ref())
                .await
                .expect("should succeed"),
        );
        moved.context_id = Some("ctx-b".into());
        mgr.update_task(moved).await.expect("should succeed");
        assert!(
            mgr.get_context_tasks("ctx-a")
                .await
                .expect("should succeed")
                .is_empty()
        );
        assert_eq!(
            mgr.get_context_tasks("ctx-b")
                .await
                .expect("should succeed")
                .len(),
            1
        );

        let t2 = Task::new("agented", "x");
        let c2 = mgr.create_task(t2).await.expect("should succeed");
        mgr.assign_task(c2.id.as_ref(), "agent-1")
            .await
            .expect("should succeed");
        assert_eq!(
            mgr.get_agent_tasks("agent-1")
                .await
                .expect("should succeed")
                .len(),
            1
        );

        let reassigned_arc = mgr.get_task(c2.id.as_ref()).await.expect("should succeed");
        assert!(
            mgr.assign_task(reassigned_arc.id.as_ref(), "agent-2")
                .await
                .is_err()
        );
        let mut reassigned = Task::clone(&reassigned_arc);
        reassigned.status_code = TaskStatus::Pending;
        reassigned.agent_id = None;
        mgr.update_task(reassigned).await.expect("should succeed");
        mgr.assign_task(c2.id.as_ref(), "agent-2")
            .await
            .expect("should succeed");
        assert!(
            mgr.get_agent_tasks("agent-1")
                .await
                .expect("should succeed")
                .is_empty()
        );
        assert_eq!(
            mgr.get_agent_tasks("agent-2")
                .await
                .expect("should succeed")
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn prerequisites_block_assign_until_complete_then_dependent_unblocks() {
        let mgr = TaskManager::new();
        let pre = Task::new("pre", "first");
        let pre_created = mgr.create_task(pre).await.expect("should succeed");
        let pre_id = pre_created.id.as_ref().to_string();

        let mut dep = Task::new("dep", "after pre");
        dep.prerequisites = vec![pre_id.clone()];
        dep.status_code = TaskStatus::Waiting;
        let dep_created = mgr.create_task(dep).await.expect("should succeed");

        let err = mgr
            .assign_task(dep_created.id.as_ref(), "a")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("Prerequisites"));

        mgr.assign_task(&pre_id, "a").await.expect("should succeed");
        mgr.complete_task(&pre_id, None)
            .await
            .expect("should succeed");

        let dep_after = mgr
            .get_task(dep_created.id.as_ref())
            .await
            .expect("should succeed");
        assert_eq!(dep_after.status_code, TaskStatus::Pending);
        assert!(
            mgr.check_prerequisites(&dep_after)
                .await
                .expect("should succeed")
        );
        assert!(
            mgr.find_assignable_tasks()
                .await
                .expect("should succeed")
                .iter()
                .any(|t| t.id.as_ref() == dep_created.id.as_ref())
        );
    }

    #[tokio::test]
    async fn get_tasks_by_status_and_fail_task() {
        let mgr = TaskManager::new();
        let t = Task::new("f", "fail me");
        let c = mgr.create_task(t).await.expect("should succeed");
        let id = c.id.as_ref().to_string();
        mgr.fail_task(&id, "boom").await.expect("should succeed");
        let failed = mgr
            .get_tasks_by_status(TaskStatus::Failed)
            .await
            .expect("should succeed");
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].id.as_ref(), id);
    }

    #[tokio::test]
    async fn complete_and_cancel_invalid_states() {
        let mgr = TaskManager::new();
        let t = Task::new("p", "pending");
        let c = mgr.create_task(t).await.expect("should succeed");
        let id = c.id.as_ref().to_string();

        let ce = mgr.complete_task(&id, None).await.unwrap_err();
        assert!(ce.to_string().contains("cannot be completed"));

        mgr.assign_task(&id, "ag").await.expect("should succeed");
        mgr.cancel_task(&id, "stop").await.expect("should succeed");

        let t2 = Task::new("p2", "run to done");
        let c2 = mgr.create_task(t2).await.expect("should succeed");
        let id2 = c2.id.as_ref().to_string();
        mgr.assign_task(&id2, "ag2").await.expect("should succeed");
        mgr.complete_task(&id2, None).await.expect("should succeed");
        let ce3 = mgr.cancel_task(&id2, "late").await.unwrap_err();
        assert!(ce3.to_string().contains("terminal"));
    }

    #[tokio::test]
    async fn check_prerequisites_missing_prereq_task_returns_false() {
        let mgr = TaskManager::new();
        let mut t = Task::new("x", "y");
        t.prerequisites = vec!["nonexistent".into()];
        assert!(!mgr.check_prerequisites(&t).await.expect("should succeed"));
    }

    #[tokio::test]
    async fn list_tasks_with_agent_and_without() {
        let mgr = TaskManager::new();
        let a = mgr
            .create_task(Task::new("a", ""))
            .await
            .expect("should succeed");
        mgr.assign_task(a.id.as_ref(), "z")
            .await
            .expect("should succeed");
        let list_z = mgr.list_tasks(Some("z")).await.expect("should succeed");
        assert_eq!(list_z.len(), 1);
        let all = mgr.list_tasks(None).await.expect("should succeed");
        assert_eq!(all.len(), 1);
    }
}
