// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Conversion functions between Task and protobuf types.
//!
//! This module provides conversions between the internal Task struct and
//! the generated protobuf Task type for serialization and communication.

use chrono::{DateTime, TimeZone, Utc};
use prost_types::Timestamp;

use crate::generated::mcp_task::{
    Task as GenTask,
};
use crate::task::types::{Task, TaskStatus, TaskPriority, AgentType};

/// Convert DateTime<Utc> to prost_types::Timestamp
pub(crate) fn datetime_to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

/// Implement conversion from Task to generated Task
impl From<Task> for GenTask {
    fn from(task: Task) -> Self {
        // Convert created_at and updated_at to Timestamp
        let created_at = Some(datetime_to_timestamp(task.created_at));
        let updated_at = Some(datetime_to_timestamp(task.updated_at));
        
        // Convert optional completion time if present
        let completed_at = task.completed_at.map(datetime_to_timestamp);
        
        // Convert input and output data to JSON bytes if present
        let input_data = task.input_data.map_or(Vec::new(), |data| {
            serde_json::to_string(&data).unwrap_or_default().into_bytes()
        });
        
        let output_data = task.output_data.map_or(Vec::new(), |data| {
            serde_json::to_string(&data).unwrap_or_default().into_bytes()
        });

        // Convert metadata to JSON bytes if present
        let metadata = task.metadata.map_or(Vec::new(), |data| {
            serde_json::to_string(&data).unwrap_or_default().into_bytes()
        });

        // Convert status, priority, and agent_type to i32 codes
        let status = task.status_code as i32;
        let priority = task.priority_code as i32;
        let agent_type = task.agent_type as i32;
        
        GenTask {
            id: task.id,
            name: task.name,
            description: task.description,
            status,
            priority,
            agent_type,
            progress_percent: task.progress as i32,
            agent_id: task.agent_id.unwrap_or_default(),
            context_id: task.context_id.unwrap_or_default(),
            prerequisite_task_ids: task.prerequisites,
            dependent_task_ids: Vec::new(), // Default empty
            created_at,
            updated_at,
            completed_at,
            input_data,
            output_data,
            error_message: task.error_message.unwrap_or_default(),
            progress_message: task.status_message.unwrap_or_default(),
            metadata,
            started_at: None, // Required by the protobuf but not present in our Task struct
        }
    }
}

/// Convert prost_types::Timestamp to DateTime<Utc>
fn timestamp_to_datetime(ts: &Timestamp) -> DateTime<Utc> {
    Utc.timestamp_opt(ts.seconds, ts.nanos as u32)
        .single()
        .unwrap_or_else(|| Utc::now())
}

/// Implement conversion from generated Task to Task
impl From<GenTask> for Task {
    fn from(gen_task: GenTask) -> Self {
        // Convert Timestamp to DateTime<Utc>
        let created_at = gen_task.created_at
            .map(|ts| timestamp_to_datetime(&ts))
            .unwrap_or_else(Utc::now);
            
        let updated_at = gen_task.updated_at
            .map(|ts| timestamp_to_datetime(&ts))
            .unwrap_or_else(Utc::now);
            
        let completed_at = gen_task.completed_at
            .map(|ts| timestamp_to_datetime(&ts));
            
        // Parse JSON bytes to HashMap if present
        let input_data = if !gen_task.input_data.is_empty() {
            match String::from_utf8(gen_task.input_data.clone()) {
                Ok(json_str) => serde_json::from_str(&json_str).ok(),
                Err(_) => None
            }
        } else {
            None
        };
        
        let output_data = if !gen_task.output_data.is_empty() {
            match String::from_utf8(gen_task.output_data.clone()) {
                Ok(json_str) => serde_json::from_str(&json_str).ok(),
                Err(_) => None
            }
        } else {
            None
        };
        
        // Parse metadata JSON bytes to HashMap if present
        let metadata = if !gen_task.metadata.is_empty() {
            match String::from_utf8(gen_task.metadata.clone()) {
                Ok(json_str) => serde_json::from_str(&json_str).ok(),
                Err(_) => None
            }
        } else {
            None
        };
        
        // Convert status, priority and agent_type codes
        let status_code = TaskStatus::from(gen_task.status as i32);
        let priority_code = TaskPriority::from(gen_task.priority as i32);
        let agent_type = AgentType::from(gen_task.agent_type as i32);
            
        Task {
            id: gen_task.id,
            name: gen_task.name,
            description: gen_task.description,
            status_code,
            priority_code,
            agent_type,
            progress: gen_task.progress_percent as f32,
            agent_id: Some(gen_task.agent_id),
            context_id: Some(gen_task.context_id),
            parent_id: None, // Not in protobuf
            prerequisites: gen_task.prerequisite_task_ids,
            created_at,
            updated_at,
            completed_at,
            input_data,
            output_data,
            metadata,
            error_message: Some(gen_task.error_message),
            status_message: Some(gen_task.progress_message),
            watchable: false, // Not in protobuf
            retry_count: 0,   // Not in protobuf
            max_retries: 3,   // Not in protobuf
            deadline: None,   // No deadline in protobuf
        }
    }
} 