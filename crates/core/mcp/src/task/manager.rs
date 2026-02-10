// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Task manager for managing tasks and their lifecycle.
//!
//! This module provides a TaskManager that handles task creation,
//! retrieval, updates, and assignment. It maintains the state of
//! all tasks in the system.

use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::debug;

use crate::error::{Result, Error};
use crate::task::types::{Task, TaskStatus};

/// Manager for task creation, execution and monitoring.
///
/// The TaskManager is responsible for maintaining the state of all tasks
/// in the system, handling their creation, updating, and assignment to agents.
#[derive(Debug)]
pub struct TaskManager {
    /// Map of task IDs to tasks
    tasks: RwLock<HashMap<String, Task>>,
    
    /// Map of agent IDs to task IDs
    agent_tasks: RwLock<HashMap<String, HashSet<String>>>,
    
    /// Map of context IDs to task IDs
    context_tasks: RwLock<HashMap<String, HashSet<String>>>,
}

impl TaskManager {
    /// Create a new task manager.
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
            task.id = Uuid::new_v4().to_string();
        }
        
        // Update the maps
        let mut tasks = self.tasks.write().await;
        
        // Check if task with this ID already exists
        if tasks.contains_key(&task.id) {
            return Err(Error::AlreadyExists(format!("Task with ID {} already exists", task.id)));
        }
        
        // Update the context task mapping
        if let Some(context_id) = &task.context_id {
            let mut context_tasks = self.context_tasks.write().await;
            let tasks_set = context_tasks.entry(context_id.clone()).or_insert_with(HashSet::new);
            tasks_set.insert(task.id.clone());
        }
        
        // Store the task
        let task_clone = task.clone();
        tasks.insert(task.id.clone(), task);
        
        Ok(task_clone)
    }
    
    /// Get a task by ID.
    pub async fn get_task(&self, id: &str) -> Result<Task> {
        let tasks = self.tasks.read().await;
        
        tasks.get(id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Task with ID {} not found", id)))
    }
    
    /// Update an existing task.
    ///
    /// This updates an existing task in the task manager. It preserves certain
    /// immutable fields like creation time and handles context and agent changes.
    pub async fn update_task(&self, updated_task: Task) -> Result<Task> {
        let mut tasks = self.tasks.write().await;
        
        // Get the existing task
        let existing_task = tasks.get(&updated_task.id)
            .ok_or_else(|| Error::NotFound(format!("Task with ID {} not found", updated_task.id)))?;
        
        // Preserve creation time and other immutable fields
        let mut merged_task = updated_task.clone();
        merged_task.created_at = existing_task.created_at;
        
        // Handle context change
        if existing_task.context_id != merged_task.context_id {
            let mut context_tasks = self.context_tasks.write().await;
            
            // Remove from old context
            if let Some(old_context_id) = &existing_task.context_id {
                if let Some(tasks_set) = context_tasks.get_mut(old_context_id) {
                    tasks_set.remove(&merged_task.id);
                }
            }
            
            // Add to new context
            if let Some(new_context_id) = &merged_task.context_id {
                let tasks_set = context_tasks.entry(new_context_id.clone()).or_insert_with(HashSet::new);
                tasks_set.insert(merged_task.id.clone());
            }
        }
        
        // Handle agent change
        if existing_task.agent_id != merged_task.agent_id {
            let mut agent_tasks = self.agent_tasks.write().await;
            
            // Remove from old agent
            if let Some(old_agent_id) = &existing_task.agent_id {
                if let Some(tasks_set) = agent_tasks.get_mut(old_agent_id) {
                    tasks_set.remove(&merged_task.id);
                }
            }
            
            // Add to new agent
            if let Some(new_agent_id) = &merged_task.agent_id {
                let tasks_set = agent_tasks.entry(new_agent_id.clone()).or_insert_with(HashSet::new);
                tasks_set.insert(merged_task.id.clone());
            }
        }
        
        // Update the task
        tasks.insert(merged_task.id.clone(), merged_task.clone());
        
        Ok(merged_task)
    }
    
    /// Assign a task to an agent.
    pub async fn assign_task(&self, task_id: &str, agent_id: &str) -> Result<Task> {
        let mut tasks = self.tasks.write().await;
        
        // Get the task
        let mut task = tasks.get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task with ID {} not found", task_id)))?
            .clone();
        
        // Check if the task is in a valid state to be assigned
        if task.status_code != TaskStatus::Pending && task.status_code != TaskStatus::Waiting {
            return Err(Error::InvalidState(format!(
                "Task {} is in state {:?} and cannot be assigned",
                task_id, task.status_code
            )));
        }
        
        // Check if all prerequisites are met
        let prerequisites_met = self.check_prerequisites(&task).await?;
        if !prerequisites_met {
            return Err(Error::InvalidState(format!(
                "Prerequisites for task {} are not all met",
                task_id
            )));
        }
        
        // Update the task
        task.mark_running(agent_id);
        
        // Update the agent task mapping
        let mut agent_tasks = self.agent_tasks.write().await;
        let tasks_set = agent_tasks.entry(agent_id.to_string()).or_insert_with(HashSet::new);
        tasks_set.insert(task_id.to_string());
        
        // Update the task in the map
        tasks.insert(task_id.to_string(), task.clone());
        
        Ok(task)
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
        let mut task = tasks.get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task with ID {} not found", task_id)))?
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
        
        // Update the task in the map
        tasks.insert(task_id.to_string(), task.clone());
        
        Ok(task)
    }
    
    /// Mark a task as completed.
    pub async fn complete_task(
        &self,
        task_id: &str,
        output_data: Option<HashMap<String, String>>,
    ) -> Result<Task> {
        let mut tasks = self.tasks.write().await;
        
        // Get the task
        let mut task = tasks.get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task with ID {} not found", task_id)))?
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
        
        // Update the task in the map
        tasks.insert(task_id.to_string(), task.clone());
        
        // Check dependent tasks
        drop(tasks); // Release the lock before calling another method
        self.check_dependent_tasks(task_id).await?;
        
        // Re-fetch the task to return the latest version
        let tasks = self.tasks.read().await;
        tasks.get(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task with ID {} not found after completion", task_id)))
            .map(|task| task.clone())
    }
    
    /// Mark a task as failed.
    pub async fn fail_task(&self, task_id: &str, error_message: &str) -> Result<Task> {
        let mut tasks = self.tasks.write().await;
        
        // Get the task
        let mut task = tasks.get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task with ID {} not found", task_id)))?
            .clone();
        
        // Mark the task as failed
        task.mark_failed(error_message);
        
        // Update the task in the map
        tasks.insert(task_id.to_string(), task.clone());
        
        Ok(task)
    }
    
    /// Cancel a task.
    pub async fn cancel_task(&self, task_id: &str, reason: &str) -> Result<Task> {
        let mut tasks = self.tasks.write().await;
        
        // Get the task
        let mut task = tasks.get_mut(task_id)
            .ok_or_else(|| Error::NotFound(format!("Task with ID {} not found", task_id)))?
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
        
        // Update the task in the map
        tasks.insert(task_id.to_string(), task.clone());
        
        Ok(task)
    }
    
    /// Get all tasks assigned to a specific agent.
    pub async fn get_agent_tasks(&self, agent_id: &str) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().await;
        let agent_tasks = self.agent_tasks.read().await;
        
        if let Some(task_ids) = agent_tasks.get(agent_id) {
            let agent_tasks: Vec<Task> = task_ids
                .iter()
                .filter_map(|task_id| tasks.get(task_id).cloned())
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
        
        let task_ids = context_tasks.get(context_id).map(|set| set.clone()).unwrap_or_default();
        
        let result: Vec<Task> = task_ids.iter()
            .filter_map(|id| tasks.get(id).cloned())
            .collect();
            
        Ok(result)
    }
    
    /// Get tasks by status.
    pub async fn get_tasks_by_status(&self, status: TaskStatus) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().await;
        
        let result: Vec<Task> = tasks.values()
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
            if let Some(prereq_task) = tasks.get(prereq_id) {
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
        let pending_tasks: Vec<Task> = tasks_guard.values()
            .filter(|task| task.status_code == TaskStatus::Pending)
            .cloned()
            .collect();
        
        // Drop the lock before processing task prerequisites
        drop(tasks_guard);
        
        let mut assignable_tasks = Vec::new();
        
        // Check each task's prerequisites
        for task in &pending_tasks {
            // Check if all prerequisites are met
            let prerequisites_met = self.check_prerequisites(task).await?;
            
            if prerequisites_met {
                assignable_tasks.push(task.clone());
            }
        }
        
        debug!("Found {} assignable tasks out of {} pending tasks", 
               assignable_tasks.len(), pending_tasks.len());
        
        Ok(assignable_tasks)
    }
    
    /// Update the status of tasks that depend on a completed task.
    ///
    /// This is called internally when a task is completed to check if any
    /// dependent tasks can now be transitioned to the Pending state.
    async fn check_dependent_tasks(&self, completed_task_id: &str) -> Result<()> {
        let tasks = self.tasks.read().await;
        
        // Find tasks that have the completed task as a prerequisite
        let dependent_tasks: Vec<Task> = tasks.values()
            .filter(|task| {
                task.prerequisites.contains(&completed_task_id.to_string()) && 
                task.status_code == TaskStatus::Waiting
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
                tasks.insert(task.id.clone(), task);
            }
        }
        
        Ok(())
    }

    /// List all tasks for a specific agent (alias for get_agent_tasks for compatibility)
    pub async fn list_tasks(&self, agent_id: Option<&str>) -> Result<Vec<Task>> {
        match agent_id {
            Some(agent_id) => self.get_agent_tasks(agent_id).await,
            None => {
                // Return all tasks if no agent specified
                let tasks = self.tasks.read().await;
                Ok(tasks.values().cloned().collect())
            }
        }
    }

    /// Update task progress (alias for update_progress for compatibility)
    pub async fn update_task_progress(
        &self,
        task_id: &str,
        progress: f32,
        status_message: &str,
    ) -> Result<Task> {
        self.update_progress(task_id, progress, Some(status_message.to_string())).await
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
} 