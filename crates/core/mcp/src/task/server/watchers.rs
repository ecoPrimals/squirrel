// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Task Watcher Management
//!
//! This module handles the management of task watchers, including registration,
//! broadcasting updates, and cleanup of closed channels.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::types::TaskUpdateSender;
use crate::task::types::Task;

/// Manager for task watchers
#[derive(Debug)]
pub struct TaskWatcherManager {
    /// Channels for tasks being watched by clients
    watchers: Arc<RwLock<HashMap<String, Vec<TaskUpdateSender>>>>,
}

impl TaskWatcherManager {
    /// Create a new `TaskWatcherManager`
    #[must_use]
    pub fn new() -> Self {
        Self {
            watchers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Broadcasts a task update to all registered watchers
    pub async fn broadcast_task_update(&self, task: Task) {
        let watchers = self.watchers.read().await;

        // Send to all watchers for this task
        if let Some(task_watchers) = watchers.get(task.id.as_ref()) {
            for sender in task_watchers {
                // Send a simple string update instead of protobuf
                let _ = sender
                    .send(format!("Task {} updated", task.id.as_ref()))
                    .await;
            }
        }
    }

    /// Check if a task update is significant to send to watchers
    #[must_use]
    pub fn is_significant_update(
        &self,
        old_task: &Task,
        new_task: &Task,
        only_watchable: bool,
    ) -> bool {
        // Always significant if states are different
        if old_task.status_code != new_task.status_code {
            return true;
        }

        // If only_watchable is true, check if the task is watchable
        if only_watchable && new_task.is_finished() {
            // Terminal state - check if it was updated recently (within last 60 seconds)
            let one_minute_ago = Utc::now() - chrono::Duration::minutes(1);
            return new_task.updated_at > one_minute_ago;
        }

        // Progress changes over 5% are significant
        if (new_task.progress - old_task.progress).abs() >= 5.0 {
            return true;
        }

        // Status message changes are significant
        if new_task.status_message != old_task.status_message {
            return true;
        }

        false
    }

    /// Register a watcher for a task
    pub async fn register_watcher(&self, task_id: &str, sender: TaskUpdateSender) {
        let mut watchers = self.watchers.write().await;
        let channels = watchers.entry(task_id.to_string()).or_insert_with(Vec::new);
        channels.push(sender);
    }

    /// Clean up closed channels for a task
    pub async fn clean_watchers(&self, task_id: &str) {
        let mut watchers = self.watchers.write().await;
        if let Some(channels) = watchers.get_mut(task_id) {
            // Keep only channels that are still open
            channels.retain(|sender| !sender.is_closed());

            // Remove the entry if no channels remain
            if channels.is_empty() {
                watchers.remove(task_id);
            }
        }
    }

    /// Unregister a watcher for a task
    pub async fn unregister_watcher(&self, task_id: &str, sender: &TaskUpdateSender) {
        let mut watchers = self.watchers.write().await;
        if let Some(channels) = watchers.get_mut(task_id) {
            // Remove the specific sender
            channels.retain(|s| !s.same_channel(sender));

            // Remove the entry if no channels remain
            if channels.is_empty() {
                watchers.remove(task_id);
            }
        }
    }
}

impl Default for TaskWatcherManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::types::{Task, TaskStatus};

    #[test]
    fn task_watcher_manager_new_default_debug() {
        let m = TaskWatcherManager::new();
        let d = TaskWatcherManager::default();
        let s = format!("{m:?}");
        assert!(s.contains("TaskWatcherManager"));
        let s2 = format!("{d:?}");
        assert!(s2.contains("TaskWatcherManager"));
    }

    #[test]
    fn is_significant_update_status_change_is_true() {
        let mgr = TaskWatcherManager::new();
        let a = Task::new("a", "d");
        let mut b = a.clone();
        b.status_code = TaskStatus::Running;
        assert!(mgr.is_significant_update(&a, &b, false));
    }

    #[test]
    fn is_significant_update_large_progress_delta_is_true() {
        let mgr = TaskWatcherManager::new();
        let a = Task::new("a", "d");
        let mut b = a.clone();
        b.progress = 10.0;
        assert!(mgr.is_significant_update(&a, &b, false));
    }

    #[test]
    fn is_significant_update_status_message_change_is_true() {
        let mgr = TaskWatcherManager::new();
        let a = Task::new("a", "d");
        let mut b = a.clone();
        b.status_message = Some("msg".into());
        assert!(mgr.is_significant_update(&a, &b, false));
    }

    #[test]
    fn is_significant_update_no_change_is_false() {
        let mgr = TaskWatcherManager::new();
        let a = Task::new("a", "d");
        let b = a.clone();
        assert!(!mgr.is_significant_update(&a, &b, false));
    }

    #[tokio::test]
    async fn register_broadcast_clean_and_unregister_watcher() {
        let mgr = TaskWatcherManager::new();
        let task = Task::new("n", "d");
        let id = task.id.as_ref().to_string();
        let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(4);
        let tx_for_unregister = tx.clone();
        mgr.register_watcher(&id, tx).await;
        mgr.broadcast_task_update(task.clone()).await;
        assert_eq!(rx.recv().await, Some(format!("Task {id} updated")));

        mgr.unregister_watcher(&id, &tx_for_unregister).await;
        let mut t2 = Task::new("n2", "d");
        t2.id = task.id.clone();
        mgr.broadcast_task_update(t2).await;
        drop(tx_for_unregister);
        assert!(rx.recv().await.is_none());

        drop(rx);
        mgr.clean_watchers(&id).await;
    }
}
