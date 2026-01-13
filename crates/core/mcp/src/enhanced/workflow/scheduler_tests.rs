//! Comprehensive tests for Workflow Scheduler
//!
//! Tests cover cron parsing, schedule management, execution timing,
//! and various scheduling patterns.

use super::scheduler::*;
use std::collections::HashMap;
use std::time::Duration;

#[tokio::test]
async fn test_scheduler_creation() {
    let config = SchedulerConfig::default();
    let scheduler = WorkflowScheduler::new(config);
    
    assert_eq!(scheduler.active_count().await, 0);
    assert!(scheduler.config().enabled);
}

#[tokio::test]
async fn test_schedule_one_time_workflow() {
    let scheduler = WorkflowScheduler::new(SchedulerConfig::default());
    
    let execution_time = chrono::Utc::now() + chrono::Duration::hours(1);
    let schedule = ScheduledWorkflow {
        id: "test-schedule-1".to_string(),
        workflow_id: "workflow-1".to_string(),
        schedule_type: ScheduleType::OneTime(execution_time),
        parameters: HashMap::new(),
        next_execution: Some(execution_time),
        last_execution: None,
        execution_count: 0,
        max_executions: Some(1),
        enabled: true,
    };
    
    scheduler.schedule_workflow(schedule).await.unwrap();
    
    let schedules = scheduler.list_schedules().await.unwrap();
    assert_eq!(schedules.len(), 1);
    assert_eq!(schedules[0].id, "test-schedule-1");
}

#[tokio::test]
async fn test_schedule_interval_workflow() {
    let scheduler = WorkflowScheduler::new(SchedulerConfig::default());
    
    let schedule = ScheduledWorkflow {
        id: "test-schedule-2".to_string(),
        workflow_id: "workflow-2".to_string(),
        schedule_type: ScheduleType::Interval(Duration::from_secs(300)),
        parameters: HashMap::new(),
        next_execution: None,
        last_execution: None,
        execution_count: 0,
        max_executions: None,
        enabled: true,
    };
    
    scheduler.schedule_workflow(schedule).await.unwrap();
    
    let retrieved = scheduler.get_schedule("test-schedule-2").await.unwrap();
    assert!(retrieved.is_some());
    assert!(retrieved.unwrap().next_execution.is_some());
}

#[tokio::test]
async fn test_cancel_schedule() {
    let scheduler = WorkflowScheduler::new(SchedulerConfig::default());
    
    let execution_time = chrono::Utc::now() + chrono::Duration::hours(1);
    let schedule = ScheduledWorkflow {
        id: "test-schedule-3".to_string(),
        workflow_id: "workflow-3".to_string(),
        schedule_type: ScheduleType::OneTime(execution_time),
        parameters: HashMap::new(),
        next_execution: Some(execution_time),
        last_execution: None,
        execution_count: 0,
        max_executions: Some(1),
        enabled: true,
    };
    
    scheduler.schedule_workflow(schedule).await.unwrap();
    assert_eq!(scheduler.list_schedules().await.unwrap().len(), 1);
    
    scheduler.cancel_schedule("test-schedule-3").await.unwrap();
    assert_eq!(scheduler.list_schedules().await.unwrap().len(), 0);
}

#[tokio::test]
async fn test_update_schedule_enabled_status() {
    let scheduler = WorkflowScheduler::new(SchedulerConfig::default());
    
    let execution_time = chrono::Utc::now() + chrono::Duration::hours(1);
    let schedule = ScheduledWorkflow {
        id: "test-schedule-4".to_string(),
        workflow_id: "workflow-4".to_string(),
        schedule_type: ScheduleType::OneTime(execution_time),
        parameters: HashMap::new(),
        next_execution: Some(execution_time),
        last_execution: None,
        execution_count: 0,
        max_executions: Some(1),
        enabled: true,
    };
    
    scheduler.schedule_workflow(schedule).await.unwrap();
    
    // Disable the schedule
    scheduler.update_schedule("test-schedule-4", false).await.unwrap();
    
    let retrieved = scheduler.get_schedule("test-schedule-4").await.unwrap().unwrap();
    assert!(!retrieved.enabled);
    
    // Re-enable the schedule
    scheduler.update_schedule("test-schedule-4", true).await.unwrap();
    
    let retrieved = scheduler.get_schedule("test-schedule-4").await.unwrap().unwrap();
    assert!(retrieved.enabled);
}

#[tokio::test]
async fn test_cron_parsing_daily_midnight() {
    let scheduler = WorkflowScheduler::new(SchedulerConfig::default());
    
    // Daily at midnight: "0 0 * * *"
    let schedule = ScheduledWorkflow {
        id: "cron-test-1".to_string(),
        workflow_id: "workflow-cron-1".to_string(),
        schedule_type: ScheduleType::Cron("0 0 * * *".to_string()),
        parameters: HashMap::new(),
        next_execution: None,
        last_execution: None,
        execution_count: 0,
        max_executions: None,
        enabled: true,
    };
    
    scheduler.schedule_workflow(schedule).await.unwrap();
    
    let retrieved = scheduler.get_schedule("cron-test-1").await.unwrap().unwrap();
    assert!(retrieved.next_execution.is_some());
    
    let next_exec = retrieved.next_execution.unwrap();
    assert_eq!(next_exec.hour(), 0);
    assert_eq!(next_exec.minute(), 0);
}

#[tokio::test]
async fn test_cron_parsing_every_5_minutes() {
    let scheduler = WorkflowScheduler::new(SchedulerConfig::default());
    
    // Every 5 minutes: "*/5 * * * *"
    let schedule = ScheduledWorkflow {
        id: "cron-test-2".to_string(),
        workflow_id: "workflow-cron-2".to_string(),
        schedule_type: ScheduleType::Cron("*/5 * * * *".to_string()),
        parameters: HashMap::new(),
        next_execution: None,
        last_execution: None,
        execution_count: 0,
        max_executions: None,
        enabled: true,
    };
    
    scheduler.schedule_workflow(schedule).await.unwrap();
    
    let retrieved = scheduler.get_schedule("cron-test-2").await.unwrap().unwrap();
    assert!(retrieved.next_execution.is_some());
    
    let next_exec = retrieved.next_execution.unwrap();
    assert_eq!(next_exec.minute() % 5, 0);
}

#[tokio::test]
async fn test_cron_parsing_weekday_business_hours() {
    let scheduler = WorkflowScheduler::new(SchedulerConfig::default());
    
    // Every hour from 9-5 on weekdays: "0 9-17 * * 1-5"
    let schedule = ScheduledWorkflow {
        id: "cron-test-3".to_string(),
        workflow_id: "workflow-cron-3".to_string(),
        schedule_type: ScheduleType::Cron("0 9-17 * * 1-5".to_string()),
        parameters: HashMap::new(),
        next_execution: None,
        last_execution: None,
        execution_count: 0,
        max_executions: None,
        enabled: true,
    };
    
    scheduler.schedule_workflow(schedule).await.unwrap();
    
    let retrieved = scheduler.get_schedule("cron-test-3").await.unwrap().unwrap();
    assert!(retrieved.next_execution.is_some());
    
    let next_exec = retrieved.next_execution.unwrap();
    assert_eq!(next_exec.minute(), 0);
    assert!(next_exec.hour() >= 9 && next_exec.hour() <= 17);
    let weekday = next_exec.weekday().num_days_from_sunday();
    assert!(weekday >= 1 && weekday <= 5); // Monday-Friday
}

#[tokio::test]
async fn test_cron_parsing_specific_times() {
    let scheduler = WorkflowScheduler::new(SchedulerConfig::default());
    
    // At 8:30 AM and 5:30 PM: "30 8,17 * * *"
    let schedule = ScheduledWorkflow {
        id: "cron-test-4".to_string(),
        workflow_id: "workflow-cron-4".to_string(),
        schedule_type: ScheduleType::Cron("30 8,17 * * *".to_string()),
        parameters: HashMap::new(),
        next_execution: None,
        last_execution: None,
        execution_count: 0,
        max_executions: None,
        enabled: true,
    };
    
    scheduler.schedule_workflow(schedule).await.unwrap();
    
    let retrieved = scheduler.get_schedule("cron-test-4").await.unwrap().unwrap();
    assert!(retrieved.next_execution.is_some());
    
    let next_exec = retrieved.next_execution.unwrap();
    assert_eq!(next_exec.minute(), 30);
    assert!(next_exec.hour() == 8 || next_exec.hour() == 17);
}

#[tokio::test]
async fn test_cron_parsing_invalid_expression() {
    let scheduler = WorkflowScheduler::new(SchedulerConfig::default());
    
    // Invalid: too few fields
    let schedule = ScheduledWorkflow {
        id: "cron-test-invalid-1".to_string(),
        workflow_id: "workflow-cron-invalid-1".to_string(),
        schedule_type: ScheduleType::Cron("0 0".to_string()),
        parameters: HashMap::new(),
        next_execution: None,
        last_execution: None,
        execution_count: 0,
        max_executions: None,
        enabled: true,
    };
    
    // Should still schedule but with fallback behavior
    let result = scheduler.schedule_workflow(schedule).await;
    assert!(result.is_ok());
    
    let retrieved = scheduler.get_schedule("cron-test-invalid-1").await.unwrap().unwrap();
    // Fallback should schedule for next minute
    assert!(retrieved.next_execution.is_some());
}

#[tokio::test]
async fn test_cron_parsing_every_minute() {
    let scheduler = WorkflowScheduler::new(SchedulerConfig::default());
    
    // Every minute: "* * * * *"
    let schedule = ScheduledWorkflow {
        id: "cron-test-5".to_string(),
        workflow_id: "workflow-cron-5".to_string(),
        schedule_type: ScheduleType::Cron("* * * * *".to_string()),
        parameters: HashMap::new(),
        next_execution: None,
        last_execution: None,
        execution_count: 0,
        max_executions: None,
        enabled: true,
    };
    
    scheduler.schedule_workflow(schedule).await.unwrap();
    
    let retrieved = scheduler.get_schedule("cron-test-5").await.unwrap().unwrap();
    assert!(retrieved.next_execution.is_some());
    
    // Should be scheduled for the next minute
    let next_exec = retrieved.next_execution.unwrap();
    let now = chrono::Utc::now();
    let diff = next_exec.signed_duration_since(now);
    assert!(diff.num_seconds() >= 0 && diff.num_seconds() <= 120);
}

#[tokio::test]
async fn test_event_driven_schedule() {
    let scheduler = WorkflowScheduler::new(SchedulerConfig::default());
    
    let schedule = ScheduledWorkflow {
        id: "event-test-1".to_string(),
        workflow_id: "workflow-event-1".to_string(),
        schedule_type: ScheduleType::EventDriven("data.updated".to_string()),
        parameters: HashMap::new(),
        next_execution: None, // Event-driven schedules don't have time-based execution
        last_execution: None,
        execution_count: 0,
        max_executions: None,
        enabled: true,
    };
    
    scheduler.schedule_workflow(schedule).await.unwrap();
    
    let retrieved = scheduler.get_schedule("event-test-1").await.unwrap().unwrap();
    assert!(retrieved.next_execution.is_none());
}

#[tokio::test]
async fn test_multiple_schedules() {
    let scheduler = WorkflowScheduler::new(SchedulerConfig::default());
    
    // Schedule multiple workflows
    for i in 0..5 {
        let schedule = ScheduledWorkflow {
            id: format!("multi-test-{}", i),
            workflow_id: format!("workflow-{}", i),
            schedule_type: ScheduleType::Interval(Duration::from_secs(60 * (i + 1) as u64)),
            parameters: HashMap::new(),
            next_execution: None,
            last_execution: None,
            execution_count: 0,
            max_executions: None,
            enabled: true,
        };
        
        scheduler.schedule_workflow(schedule).await.unwrap();
    }
    
    let schedules = scheduler.list_schedules().await.unwrap();
    assert_eq!(schedules.len(), 5);
}

#[tokio::test]
async fn test_scheduler_config() {
    let config = SchedulerConfig {
        enabled: false,
        check_interval: Duration::from_secs(5),
        max_concurrent: 50,
        timezone: "America/New_York".to_string(),
    };
    
    let scheduler = WorkflowScheduler::new(config);
    
    assert!(!scheduler.config().enabled);
    assert_eq!(scheduler.config().check_interval, Duration::from_secs(5));
    assert_eq!(scheduler.config().max_concurrent, 50);
    assert_eq!(scheduler.config().timezone, "America/New_York");
}

#[tokio::test]
async fn test_cron_parsing_monthly_first_day() {
    let scheduler = WorkflowScheduler::new(SchedulerConfig::default());
    
    // First day of month at noon: "0 12 1 * *"
    let schedule = ScheduledWorkflow {
        id: "cron-test-6".to_string(),
        workflow_id: "workflow-cron-6".to_string(),
        schedule_type: ScheduleType::Cron("0 12 1 * *".to_string()),
        parameters: HashMap::new(),
        next_execution: None,
        last_execution: None,
        execution_count: 0,
        max_executions: None,
        enabled: true,
    };
    
    scheduler.schedule_workflow(schedule).await.unwrap();
    
    let retrieved = scheduler.get_schedule("cron-test-6").await.unwrap().unwrap();
    assert!(retrieved.next_execution.is_some());
    
    let next_exec = retrieved.next_execution.unwrap();
    assert_eq!(next_exec.day(), 1);
    assert_eq!(next_exec.hour(), 12);
    assert_eq!(next_exec.minute(), 0);
}

#[tokio::test]
async fn test_schedule_with_parameters() {
    let scheduler = WorkflowScheduler::new(SchedulerConfig::default());
    
    let mut parameters = HashMap::new();
    parameters.insert("user_id".to_string(), serde_json::json!("user123"));
    parameters.insert("action".to_string(), serde_json::json!("backup"));
    
    let schedule = ScheduledWorkflow {
        id: "param-test-1".to_string(),
        workflow_id: "workflow-param-1".to_string(),
        schedule_type: ScheduleType::Interval(Duration::from_secs(3600)),
        parameters: parameters.clone(),
        next_execution: None,
        last_execution: None,
        execution_count: 0,
        max_executions: None,
        enabled: true,
    };
    
    scheduler.schedule_workflow(schedule).await.unwrap();
    
    let retrieved = scheduler.get_schedule("param-test-1").await.unwrap().unwrap();
    assert_eq!(retrieved.parameters.len(), 2);
    assert_eq!(retrieved.parameters.get("user_id").unwrap(), &serde_json::json!("user123"));
    assert_eq!(retrieved.parameters.get("action").unwrap(), &serde_json::json!("backup"));
}
