// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

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

/// Parse cron expression and calculate next execution time
///
/// Supports standard cron format: "minute hour day month weekday"
/// Examples:
/// - "0 0 * * *" - Daily at midnight
/// - "*/5 * * * *" - Every 5 minutes
/// - "0 9-17 * * 1-5" - Every hour from 9-5 on weekdays
///
/// Format: minute (0-59) hour (0-23) day (1-31) month (1-12) weekday (0-6, 0=Sunday)
/// Special characters: * (any), */n (every n), n-m (range), n,m (list)
fn parse_cron_expression(expr: &str) -> Result<chrono::DateTime<chrono::Utc>> {
    let parts: Vec<&str> = expr.trim().split_whitespace().collect();
    
    if parts.len() != 5 {
        return Err(MCPError::InvalidArgument(format!(
            "Invalid cron expression '{}': expected 5 fields (minute hour day month weekday), got {}",
            expr,
            parts.len()
        )));
    }
    
    let now = chrono::Utc::now();
    let mut next_time = now;
    
    // Parse each field
    let minute_spec = parts[0];
    let hour_spec = parts[1];
    let day_spec = parts[2];
    let month_spec = parts[3];
    let weekday_spec = parts[4];
    
    // Start from the next minute
    next_time = next_time + chrono::Duration::minutes(1);
    next_time = next_time
        .with_second(0)
        .and_then(|t| t.with_nanosecond(0))
        .unwrap_or(next_time);
    
    // Find next matching time (limited search to prevent infinite loops)
    for _ in 0..10000 {
        // Safety limit: search up to ~1 week ahead
        if matches_cron_expression(
            &next_time,
            minute_spec,
            hour_spec,
            day_spec,
            month_spec,
            weekday_spec,
        )? {
            return Ok(next_time);
        }
        
        // Advance by 1 minute
        next_time = next_time + chrono::Duration::minutes(1);
    }
    
    // If we can't find a match within a week, return an error
    Err(MCPError::InvalidArgument(format!(
        "Could not find next execution time for cron expression '{}' within search window",
        expr
    )))
}

/// Check if a datetime matches a cron expression
fn matches_cron_expression(
    dt: &chrono::DateTime<chrono::Utc>,
    minute_spec: &str,
    hour_spec: &str,
    day_spec: &str,
    month_spec: &str,
    weekday_spec: &str,
) -> Result<bool> {
    let minute = dt.minute() as i32;
    let hour = dt.hour() as i32;
    let day = dt.day() as i32;
    let month = dt.month() as i32;
    let weekday = dt.weekday().num_days_from_sunday() as i32; // 0=Sunday
    
    Ok(matches_cron_field(minute_spec, minute, 0, 59)?
        && matches_cron_field(hour_spec, hour, 0, 23)?
        && matches_cron_field(day_spec, day, 1, 31)?
        && matches_cron_field(month_spec, month, 1, 12)?
        && matches_cron_field(weekday_spec, weekday, 0, 6)?)
}

/// Check if a value matches a cron field specification
///
/// Supports:
/// - * (any value)
/// - n (specific value)
/// - n,m,o (list of values)
/// - n-m (range)
/// - */n (every n)
/// - n-m/s (range with step)
fn matches_cron_field(spec: &str, value: i32, min: i32, max: i32) -> Result<bool> {
    // Wildcard matches everything
    if spec == "*" {
        return Ok(true);
    }
    
    // Handle step values (*/n or n-m/s)
    if spec.contains('/') {
        let parts: Vec<&str> = spec.split('/').collect();
        if parts.len() != 2 {
            return Err(MCPError::InvalidArgument(format!(
                "Invalid cron field '{}': step syntax requires exactly one '/'",
                spec
            )));
        }
        
        let step: i32 = parts[1].parse().map_err(|_| {
            MCPError::InvalidArgument(format!(
                "Invalid cron step value '{}': must be a number",
                parts[1]
            ))
        })?;
        
        if step <= 0 {
            return Err(MCPError::InvalidArgument(format!(
                "Invalid cron step value '{}': must be positive",
                step
            )));
        }
        
        // Handle */n (every n)
        if parts[0] == "*" {
            return Ok(value % step == 0);
        }
        
        // Handle n-m/s (range with step)
        if parts[0].contains('-') {
            let range_parts: Vec<&str> = parts[0].split('-').collect();
            if range_parts.len() != 2 {
                return Err(MCPError::InvalidArgument(format!(
                    "Invalid cron range '{}': expected 'start-end'",
                    parts[0]
                )));
            }
            
            let range_start: i32 = range_parts[0].parse().map_err(|_| {
                MCPError::InvalidArgument(format!(
                    "Invalid cron range start '{}': must be a number",
                    range_parts[0]
                ))
            })?;
            
            let range_end: i32 = range_parts[1].parse().map_err(|_| {
                MCPError::InvalidArgument(format!(
                    "Invalid cron range end '{}': must be a number",
                    range_parts[1]
                ))
            })?;
            
            if value < range_start || value > range_end {
                return Ok(false);
            }
            
            return Ok((value - range_start) % step == 0);
        }
        
        return Err(MCPError::InvalidArgument(format!(
            "Invalid cron step specification '{}'",
            spec
        )));
    }
    
    // Handle comma-separated lists (n,m,o)
    if spec.contains(',') {
        for part in spec.split(',') {
            if matches_cron_field(part.trim(), value, min, max)? {
                return Ok(true);
            }
        }
        return Ok(false);
    }
    
    // Handle ranges (n-m)
    if spec.contains('-') {
        let parts: Vec<&str> = spec.split('-').collect();
        if parts.len() != 2 {
            return Err(MCPError::InvalidArgument(format!(
                "Invalid cron range '{}': expected 'start-end'",
                spec
            )));
        }
        
        let range_start: i32 = parts[0].parse().map_err(|_| {
            MCPError::InvalidArgument(format!(
                "Invalid cron range start '{}': must be a number",
                parts[0]
            ))
        })?;
        
        let range_end: i32 = parts[1].parse().map_err(|_| {
            MCPError::InvalidArgument(format!(
                "Invalid cron range end '{}': must be a number",
                parts[1]
            ))
        })?;
        
        return Ok(value >= range_start && value <= range_end);
    }
    
    // Handle specific value (n)
    let specific_value: i32 = spec.parse().map_err(|_| {
        MCPError::InvalidArgument(format!(
            "Invalid cron field value '{}': must be a number, range, list, or wildcard",
            spec
        ))
    })?;
    
    if specific_value < min || specific_value > max {
        return Err(MCPError::InvalidArgument(format!(
            "Cron field value {} out of range ({}-{})",
            specific_value, min, max
        )));
    }
    
    Ok(value == specific_value)
}

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
                let duration_chrono = chrono::Duration::from_std(*duration)
                    .map_err(|e| crate::error::types::MCPError::InvalidArgument(format!(
                        "Invalid duration for interval schedule: {}", e
                    )))?;
                Ok(Some(chrono::Utc::now() + duration_chrono))
            }
            ScheduleType::Cron(expr) => {
                // Parse cron expression and calculate next execution time
                // Supports standard cron format: "minute hour day month weekday"
                // Examples: "0 0 * * *" (daily at midnight), "*/5 * * * *" (every 5 minutes)
                match parse_cron_expression(expr) {
                    Ok(next_time) => Ok(Some(next_time)),
                    Err(e) => {
                        tracing::warn!("Failed to parse cron expression '{}': {}. Using fallback (next minute)", expr, e);
                        // Fallback to next minute if parsing fails
                        Ok(Some(chrono::Utc::now() + chrono::Duration::minutes(1)))
                    }
                }
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

