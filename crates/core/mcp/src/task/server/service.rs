// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Core Task Service Implementation
//!
//! This module contains the main TaskServiceImpl struct and its initialization methods.

use crate::task::TaskManager;
use crate::generated::mcp_task::task_service_server::TaskServiceServer;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Task server configuration
#[derive(Debug, Clone)]
pub struct TaskServerConfig {
    pub max_concurrent_tasks: usize,
    pub task_timeout_seconds: u64,
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
    pub fn new(task_manager: Arc<Mutex<TaskManager>>, config: TaskServerConfig) -> Self {
        Self {
            task_manager,
            config,
        }
    }

    /// Create a new TaskServiceServer with the provided TaskManager.
    pub fn create_server(task_manager: Arc<Mutex<TaskManager>>) -> TaskServiceServer<Self> {
        let service = Self::new(task_manager, TaskServerConfig::default());
        TaskServiceServer::new(service)
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