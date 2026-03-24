// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Distributed Tracing
//! 
//! This module provides distributed tracing capabilities for the MCP,
//! enabling request tracking across system components and services.
//!
//! ## Architecture
//!
//! The tracing system is organized into focused modules:
//! - `types`: Core tracing types and configuration structures
//! - `active_span`: ActiveSpan wrapper for lifecycle management
//! - `tracer`: Main Tracer implementation for span management
//! - `external`: External system integration (OpenTelemetry, Jaeger, Zipkin)
//!
//! ## Key Components
//!
//! - **Span**: A unit of work or operation, with start and end times
//! - **Trace**: A collection of spans forming a request flow
//! - **SpanContext**: Context that propagates between services
//! - **Tracer**: Creates and manages spans and traces
//! - **ActiveSpan**: Wrapper that manages span lifecycle
//! - **SpanExporter**: Interface for external system integration
//!
//! ## Usage
//!
//! ```rust
//! use std::sync::Arc;
//! use crate::observability::tracing::{Tracer, TracerConfig};
//!
//! // Create and configure a tracer
//! let tracer = Arc::new(Tracer::new());
//! let config = TracerConfig {
//!     enabled: true,
//!     sampling_rate: 1.0,
//!     max_spans: 10000,
//! };
//! tracer.set_config(config)?;
//!
//! // Start a span
//! let span = tracer.start_span("operation_name")?;
//!
//! // Add attributes and events
//! {
//!     let mut span_guard = span.lock().expect("example");
//!     span_guard.add_attribute("key", "value");
//!     span_guard.add_event("event_name", HashMap::new());
//! }
//!
//! // End the span
//! {
//!     let span_guard = span.lock().expect("example");
//!     span_guard.end();
//! }
//! ```
//!
//! ## External Integration
//!
//! The tracing system supports integration with external tracing platforms:
//!
//! ```rust
//! use crate::observability::tracing::external::{
//!     ExternalTracingConfig, OpenTelemetryExporter, ExternalTracer
//! };
//!
//! // Configure external tracing
//! let config = ExternalTracingConfig {
//!     endpoint_url: "http://jaeger:14268/api/traces".to_string(),
//!     service_name: "my-service".to_string(),
//!     environment: "production".to_string(),
//!     ..Default::default()
//! };
//!
//! // Create exporter and tracer
//! let exporter = OpenTelemetryExporter::new(config);
//! let mut external_tracer = ExternalTracer::new(exporter);
//! external_tracer.initialize().await?;
//!
//! // Use the external tracer
//! let span = external_tracer.start_span("external_operation")?;
//! // ... work ...
//! external_tracer.export_completed_spans().await?;
//! ```

// Core modules
pub mod types;
pub mod active_span;
pub mod tracer;

// External integration
pub mod external;

// Re-export core types
pub use types::{
    Span, SpanEvent, SpanStatus, SpanContext, TracerConfig, SpanSnapshot
};

// Re-export active span functionality
pub use active_span::{ActiveSpan, set_current_span, current_span};

// Re-export tracer
pub use tracer::Tracer;

// Re-export external integration
pub use external::{
    ExternalTracingConfig, SpanExporter, 
    OpenTelemetryExporter, JaegerExporter, ZipkinExporter,
    ExternalTracer, convert_to_otlp_format, convert_to_zipkin_format
}; 