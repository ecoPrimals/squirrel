//! Workflow Scheduler
//!
//! Manages workflow scheduling, cron jobs, and time-based execution.
//! Supports one-time, recurring, and event-driven scheduling.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, instrument};
use serde::{Serialize, Deserialize};

use crate::error::{Result, types::MCPError};
use super::types::*;

/// Workflow scheduler
///
/// Manages workflow scheduling, cron jobs, and time-based execution.
/// Supports one-time, recurring, and event-driven scheduling.
#[derive(Debug)]
pub struct WorkflowScheduler {
    /// Scheduled workflows
    scheduled_workflows: Arc<RwLock<HashMap<String, ScheduledWorkflow>>>,
    
    /// Scheduler configuration
    config: SchedulerConfig,
    
    /// Active schedules
    active_schedules: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

/// Scheduler configuration
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Enable scheduler
    pub enabled: bool,
    
    /// Check interval for scheduled workflows
    pub check_interval: Duration,
    
    /// Maximum concurrent scheduled workflows
    pub max_concurrent: usize,
    
    /// Timezone for scheduling
    pub timezone: String,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: Duration::from_secs(10),
            max_concurrent: 100,
            timezone: "UTC".to_string(),
        }
    }
}

/// Scheduled workflow
#[derive(Debug, Clone)]
pub struct ScheduledWorkflow {
    /// Schedule ID
    pub id: String,
    
    /// Workflow ID to execute
    pub workflow_id: String,
    
    /// Schedule type
    pub schedule_type: ScheduleType,
    
    /// Parameters for workflow execution
    pub parameters: HashMap<String, serde_json::Value>,
    
    /// Next execution time
    pub next_execution: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Last execution time
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Execution count
    pub execution_count: u64,
    
    /// Maximum executions (None = unlimited)
    pub max_executions: Option<u64>,
    
    /// Schedule enabled
    pub enabled: bool,
}

/// Schedule type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleType {
    /// One-time execution at specific time
    OneTime(chrono::DateTime<chrono::Utc>),
    
    /// Recurring with cron expression
    Cron(String),
    
    /// Recurring with interval
    Interval(Duration),
    
    /// Event-driven (triggered by external event)
    EventDriven(String),
}

impl WorkflowScheduler {
    /// Create a new scheduler
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            scheduled_workflows: Arc::new(RwLock::new(HashMap::new())),
            config,
            active_schedules: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Schedule a workflow
    #[instrument(skip(self))]
    pub async fn schedule_workflow(
        &self,
        workflow_id: String,
        schedule_type: ScheduleType,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        let schedule_id = uuid::Uuid::new_v4().to_string();
        
        let next_execution = self.calculate_next_execution(&schedule_type)?;
        
        let scheduled = ScheduledWorkflow {
            id: schedule_id.clone(),
            workflow_id,
            schedule_type,
            parameters,
            next_execution,
            last_execution: None,
            execution_count: 0,
            max_executions: None,
            enabled: true,
        };
        
        let mut schedules = self.scheduled_workflows.write().await;
        schedules.insert(schedule_id.clone(), scheduled);
        
        info!("Scheduled workflow: {}", schedule_id);
        Ok(schedule_id)
    }
    
    /// Calculate next execution time based on schedule type
    fn calculate_next_execution(
        &self,
        schedule_type: &ScheduleType,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>> {
        match schedule_type {
            ScheduleType::OneTime(time) => Ok(Some(*time)),
            ScheduleType::Interval(duration) => {
                Ok(Some(chrono::Utc::now() + chrono::Duration::from_std(*duration).unwrap()))
            }
            ScheduleType::Cron(_expr) => {
                // TODO: Implement cron parsing with cron library
                // For now, schedule for next minute
                Ok(Some(chrono::Utc::now() + chrono::Duration::minutes(1)))
            }
            ScheduleType::EventDriven(_) => Ok(None), // No time-based execution
        }
    }
    
    /// Cancel a scheduled workflow
    #[instrument(skip(self))]
    pub async fn cancel_schedule(&self, schedule_id: &str) -> Result<()> {
        let mut schedules = self.scheduled_workflows.write().await;
        schedules.remove(schedule_id);
        
        // Cancel active task if running
        let mut active = self.active_schedules.write().await;
        if let Some(handle) = active.remove(schedule_id) {
            handle.abort();
        }
        
        info!("Cancelled schedule: {}", schedule_id);
        Ok(())
    }
    
    /// List all scheduled workflows
    pub async fn list_schedules(&self) -> Result<Vec<ScheduledWorkflow>> {
        let schedules = self.scheduled_workflows.read().await;
        Ok(schedules.values().cloned().collect())
    }
    
    /// Get schedule by ID
    pub async fn get_schedule(&self, schedule_id: &str) -> Result<Option<ScheduledWorkflow>> {
        let schedules = self.scheduled_workflows.read().await;
        Ok(schedules.get(schedule_id).cloned())
    }
    
    /// Update schedule enabled status
    #[instrument(skip(self))]
    pub async fn update_schedule(&self, schedule_id: &str, enabled: bool) -> Result<()> {
        let mut schedules = self.scheduled_workflows.write().await;
        if let Some(schedule) = schedules.get_mut(schedule_id) {
            schedule.enabled = enabled;
            info!("Updated schedule {} enabled: {}", schedule_id, enabled);
            Ok(())
        } else {
            Err(MCPError::InvalidArgument(format!(
                "Schedule not found: {}",
                schedule_id
            ))
            .into())
        }
    }
    
    /// Check and execute due schedules
    ///
    /// This is typically called by a background task at regular intervals
    pub async fn check_schedules(&self) -> Vec<ScheduledWorkflow> {
        let schedules = self.scheduled_workflows.read().await;
        let now = chrono::Utc::now();
        
        schedules
            .values()
            .filter(|s| {
                s.enabled
                    && s.next_execution
                        .map(|next| next <= now)
                        .unwrap_or(false)
            })
            .cloned()
            .collect()
    }
    
    /// Update schedule after execution
    #[instrument(skip(self))]
    pub async fn record_execution(&self, schedule_id: &str) -> Result<()> {
        let mut schedules = self.scheduled_workflows.write().await;
        if let Some(schedule) = schedules.get_mut(schedule_id) {
            schedule.last_execution = Some(chrono::Utc::now());
            schedule.execution_count += 1;
            
            // Calculate next execution
            schedule.next_execution = self.calculate_next_execution(&schedule.schedule_type)?;
            
            // Disable if max executions reached
            if let Some(max) = schedule.max_executions {
                if schedule.execution_count >= max {
                    schedule.enabled = false;
                    info!(
                        "Schedule {} disabled after reaching max executions: {}",
                        schedule_id, max
                    );
                }
            }
            
            Ok(())
        } else {
            Err(MCPError::InvalidArgument(format!(
                "Schedule not found: {}",
                schedule_id
            ))
            .into())
        }
    }
    
    /// Get scheduler configuration
    pub fn config(&self) -> &SchedulerConfig {
        &self.config
    }
    
    /// Get active schedule count
    pub async fn active_count(&self) -> usize {
        let active = self.active_schedules.read().await;
        active.len()
    }
}

