// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tracing Types and Configuration
//!
//! This module contains the core types used by the distributed tracing system,
//! including spans, events, contexts, and configuration structures.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Status of a span execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpanStatus {
    /// Span is still in progress
    Running,
    /// Span completed successfully
    Success,
    /// Span completed with an error
    Error,
}

/// An event that occurs during a span
#[derive(Debug, Clone)]
pub struct SpanEvent {
    /// Name of the event
    pub(crate) name: String,
    /// When the event occurred
    pub(crate) timestamp: Instant,
    /// Additional event attributes
    pub(crate) attributes: HashMap<String, String>,
}

impl SpanEvent {
    /// Create a new span event
    pub fn new(name: impl Into<String>, attributes: HashMap<String, String>) -> Self {
        Self {
            name: name.into(),
            timestamp: Instant::now(),
            attributes,
        }
    }

    /// Get the name of the event
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the timestamp when the event occurred
    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }

    /// Get the event attributes
    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }
}

/// Span represents a single unit of work in a distributed trace
#[derive(Debug, Clone)]
pub struct Span {
    /// Unique identifier for the span
    pub(crate) id: String,
    /// Trace ID this span belongs to
    pub(crate) trace_id: String,
    /// Parent span ID, if any
    pub(crate) parent_id: Option<String>,
    /// Name of the span
    pub(crate) name: String,
    /// When the span was started
    pub(crate) start_time: Instant,
    /// When the span was ended, if complete
    pub(crate) end_time: Option<Instant>,
    /// Key-value attributes for additional context
    pub(crate) attributes: HashMap<String, String>,
    /// Events that occurred during the span
    pub(crate) events: Vec<SpanEvent>,
    /// Status of the span
    pub(crate) status: SpanStatus,
}

impl Span {
    /// Create a new span
    pub fn new(
        name: impl Into<String>,
        trace_id: impl Into<String>,
        parent_id: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            trace_id: trace_id.into(),
            parent_id,
            name: name.into(),
            start_time: Instant::now(),
            end_time: None,
            attributes: HashMap::new(),
            events: Vec::new(),
            status: SpanStatus::Running,
        }
    }

    /// End the span and mark it as successful
    pub fn end(&mut self) {
        self.end_time = Some(Instant::now());
        self.status = SpanStatus::Success;
    }

    /// End the span and mark it as error
    pub fn end_with_error(&mut self) {
        self.end_time = Some(Instant::now());
        self.status = SpanStatus::Error;
    }

    /// Add an attribute to the span
    pub fn add_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    /// Add an event to the span
    pub fn add_event(&mut self, name: impl Into<String>, attributes: HashMap<String, String>) {
        self.events.push(SpanEvent::new(name, attributes));
    }

    /// Get the span ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the trace ID
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    /// Get the parent span ID
    pub fn parent_id(&self) -> Option<&str> {
        self.parent_id.as_deref()
    }

    /// Get the span name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the span duration
    pub fn duration(&self) -> Option<Duration> {
        self.end_time.map(|end| end.duration_since(self.start_time))
    }

    /// Get the span status
    pub fn status(&self) -> SpanStatus {
        self.status
    }

    /// Check if the span is active
    pub fn is_active(&self) -> bool {
        self.status == SpanStatus::Running
    }

    /// Get all span attributes
    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }

    /// Get all span events
    pub fn events(&self) -> &[SpanEvent] {
        &self.events
    }
}

/// Context for propagating trace information between services
#[derive(Debug, Clone)]
pub struct SpanContext {
    /// Trace ID
    trace_id: String,
    /// Span ID
    span_id: String,
    /// Whether this context is sampled (recorded)
    sampled: bool,
    /// Additional context information
    baggage: HashMap<String, String>,
}

impl SpanContext {
    /// Create a new span context
    pub fn new(trace_id: impl Into<String>, span_id: impl Into<String>, sampled: bool) -> Self {
        Self {
            trace_id: trace_id.into(),
            span_id: span_id.into(),
            sampled,
            baggage: HashMap::new(),
        }
    }

    /// Get the trace ID
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    /// Get the span ID
    pub fn span_id(&self) -> &str {
        &self.span_id
    }

    /// Check if this context is sampled
    pub fn is_sampled(&self) -> bool {
        self.sampled
    }

    /// Add baggage item
    pub fn add_baggage(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.baggage.insert(key.into(), value.into());
    }

    /// Get baggage item
    pub fn get_baggage(&self, key: &str) -> Option<&str> {
        self.baggage.get(key).map(|s| s.as_str())
    }

    /// Get all baggage
    pub fn baggage(&self) -> &HashMap<String, String> {
        &self.baggage
    }
}

/// Configuration for the tracer
#[derive(Debug, Clone)]
pub struct TracerConfig {
    /// Whether tracing is enabled
    pub enabled: bool,
    /// Sampling rate (0.0-1.0)
    pub sampling_rate: f64,
    /// Maximum number of spans to keep in memory
    pub max_spans: usize,
}

impl Default for TracerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling_rate: 1.0,
            max_spans: 10000,
        }
    }
}

/// Snapshot of a span for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanSnapshot {
    /// Unique identifier for the span
    pub id: String,
    /// Trace ID this span belongs to
    pub trace_id: String,
    /// Parent span ID, if any
    pub parent_id: Option<String>,
    /// Name of the span
    pub name: String,
    /// When the span was started
    pub start_time: std::time::SystemTime,
    /// When the span was ended, if complete
    pub end_time: Option<std::time::SystemTime>,
    /// Key-value attributes for additional context
    pub attributes: HashMap<String, String>,
    /// Events that occurred during the span
    pub events: Vec<SpanEventSnapshot>,
    /// Status of the span
    pub status: SpanStatus,
    /// Duration of the span
    pub duration_ms: Option<u64>,
}

/// Snapshot of a span event for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEventSnapshot {
    /// Name of the event
    pub name: String,
    /// When the event occurred
    pub timestamp: std::time::SystemTime,
    /// Additional event attributes
    pub attributes: HashMap<String, String>,
} 