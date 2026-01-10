//! Compute state management
//!
//! This module manages the state of compute resources, jobs, and nodes.

use std::collections::HashMap;

use super::job::{ComputeJob, QueuedJob};
use super::node::ComputeNode;
use super::resource::ResourceAllocation;

/// Compute state management
#[derive(Debug, Clone, Default)]
pub struct ComputeState {
    pub active_jobs: HashMap<String, ComputeJob>,
    pub resource_allocations: HashMap<String, ResourceAllocation>,
    pub compute_nodes: HashMap<String, ComputeNode>,
    pub job_queue: Vec<QueuedJob>,
    pub registered: bool,
}

impl ComputeState {
    /// Create a new compute state
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a compute node
    pub fn add_node(&mut self, node_id: String, node: ComputeNode) {
        self.compute_nodes.insert(node_id, node);
    }

    /// Remove a compute node
    pub fn remove_node(&mut self, node_id: &str) {
        self.compute_nodes.remove(node_id);
    }

    /// Add a job to the active jobs
    pub fn add_job(&mut self, job_id: String, job: ComputeJob) {
        self.active_jobs.insert(job_id, job);
    }

    /// Remove a job from active jobs
    pub fn remove_job(&mut self, job_id: &str) {
        self.active_jobs.remove(job_id);
    }

    /// Add a job to the queue
    pub fn enqueue_job(&mut self, job: QueuedJob) {
        self.job_queue.push(job);
    }

    /// Get the next job from the queue
    pub fn dequeue_job(&mut self) -> Option<QueuedJob> {
        if self.job_queue.is_empty() {
            None
        } else {
            Some(self.job_queue.remove(0))
        }
    }

    /// Add a resource allocation
    pub fn add_allocation(&mut self, allocation_id: String, allocation: ResourceAllocation) {
        self.resource_allocations.insert(allocation_id, allocation);
    }

    /// Remove a resource allocation
    pub fn remove_allocation(&mut self, allocation_id: &str) {
        self.resource_allocations.remove(allocation_id);
    }

    /// Get the number of active jobs
    #[must_use]
    pub fn active_job_count(&self) -> usize {
        self.active_jobs.len()
    }

    /// Get the number of queued jobs
    #[must_use]
    pub fn queued_job_count(&self) -> usize {
        self.job_queue.len()
    }

    /// Get the number of compute nodes
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.compute_nodes.len()
    }
}
