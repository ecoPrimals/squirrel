// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! # Tracing Interfaces
//!
//! This module contains interfaces for distributed tracing that are used
//! by multiple Squirrel components, including MCP and dashboard.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;

/// Status of a trace operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceStatus {
    /// Operation is still in progress
    Running,
    /// Operation completed successfully
    Success,
    /// Operation completed with an error
    Error,
    /// Operation status is unknown
    Unknown,
}

/// A serializable trace event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Name of the event
    pub name: String,
    /// When the event occurred (as unix timestamp in nanos)
    pub timestamp: u64,
    /// Additional event attributes
    pub attributes: HashMap<String, String>,
}

/// A serializable trace span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSpan {
    /// Unique identifier for the span
    pub id: String,
    /// Trace ID this span belongs to
    pub trace_id: String,
    /// Parent span ID, if any
    pub parent_id: Option<String>,
    /// Name of the span
    pub name: String,
    /// When the span was started (as unix timestamp in nanos)
    pub start_time: u64,
    /// When the span was ended (as unix timestamp in nanos), if complete
    pub end_time: Option<u64>,
    /// Key-value attributes for additional context
    pub attributes: HashMap<String, String>,
    /// Events that occurred during the span
    pub events: Vec<TraceEvent>,
    /// Status of the span
    pub status: TraceStatus,
    /// Service or component that created the span
    pub service: String,
}

/// A collection of trace spans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceData {
    /// All spans in the trace
    pub spans: Vec<TraceSpan>,
    /// Service or component that collected the trace
    pub service: String,
    /// Environment (e.g., "production", "staging")
    pub environment: String,
    /// When the trace data was collected
    pub collected_at: u64,
}

/// Trait for consuming trace data
pub trait TraceDataConsumer: Send + Sync {
    /// Process incoming trace data
    fn consume_trace_data(
        &self,
        trace_data: TraceData,
    ) -> impl Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send;
}

/// Trait for providing trace data
pub trait TraceDataProvider: Send + Sync {
    /// Get trace data
    fn get_trace_data(
        &self,
    ) -> impl Future<Output = Result<Vec<TraceData>, Box<dyn std::error::Error + Send + Sync>>> + Send;

    /// Get trace data for a specific trace ID
    fn get_trace_by_id(
        &self,
        trace_id: &str,
    ) -> impl Future<Output = Result<Option<TraceData>, Box<dyn std::error::Error + Send + Sync>>> + Send;
}

/// Configuration for trace data handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceConfig {
    /// Service or component name
    pub service_name: String,
    /// Environment name
    pub environment: String,
    /// Whether to include standard attributes
    pub include_standard_attributes: bool,
    /// Maximum number of events per span
    pub max_events_per_span: usize,
    /// Maximum number of spans to include
    pub max_spans: usize,
}
