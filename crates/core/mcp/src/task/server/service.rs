// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Core Task Service Implementation
//!
//! This module contains the main `TaskServiceImpl` struct and its initialization methods.

use crate::task::manager::TaskManager;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Task server configuration
#[derive(Debug, Clone)]
pub struct TaskServerConfig {
    /// Maximum number of tasks that can run concurrently
    pub max_concurrent_tasks: usize,
    /// Timeout for task execution in seconds
    pub task_timeout_seconds: u64,
    /// Whether to collect and expose metrics
    pub enable_metrics: bool,
}

impl Default for TaskServerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 100,
            task_timeout_seconds: 300,
            enable_metrics: true,
        }
    }
}

/// Task service implementation
#[derive(Debug)]
pub struct TaskServiceImpl {
    pub(super) task_manager: Arc<Mutex<TaskManager>>,
    pub(super) config: TaskServerConfig,
}

impl TaskServiceImpl {
    /// Creates a new task service with the given task manager and configuration.
    pub const fn new(task_manager: Arc<Mutex<TaskManager>>, config: TaskServerConfig) -> Self {
        Self {
            task_manager,
            config,
        }
    }

    /// Create a new `TaskServiceImpl` with the provided `TaskManager`.
    /// Use `handle_json_rpc_request` to process JSON-RPC requests.
    pub fn create_server(task_manager: Arc<Mutex<TaskManager>>) -> Self {
        Self::new(task_manager, TaskServerConfig::default())
    }
}

impl Clone for TaskServiceImpl {
    fn clone(&self) -> Self {
        Self {
            task_manager: Arc::clone(&self.task_manager),
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::manager::TaskManager;

    #[test]
    fn task_server_config_default_clone_debug() {
        let a = TaskServerConfig::default();
        let b = a.clone();
        assert_eq!(a.max_concurrent_tasks, b.max_concurrent_tasks);
        assert_eq!(a.task_timeout_seconds, b.task_timeout_seconds);
        assert_eq!(a.enable_metrics, b.enable_metrics);
        let s = format!("{a:?}");
        assert!(s.contains("TaskServerConfig"));
    }

    #[test]
    fn task_service_impl_new_and_create_server_and_clone_share_manager() {
        let tm = Arc::new(Mutex::new(TaskManager::new()));
        let cfg = TaskServerConfig {
            max_concurrent_tasks: 7,
            task_timeout_seconds: 42,
            enable_metrics: false,
        };
        let svc = TaskServiceImpl::new(Arc::clone(&tm), cfg);
        let svc2 = TaskServiceImpl::create_server(Arc::clone(&tm));
        let cloned = svc.clone();
        assert!(Arc::ptr_eq(&svc.task_manager, &cloned.task_manager));
        assert!(Arc::ptr_eq(&svc.task_manager, &svc2.task_manager));
        assert_eq!(svc.config.max_concurrent_tasks, 7);
        let dbg = format!("{svc:?}");
        assert!(dbg.contains("TaskServiceImpl"));
    }
}
