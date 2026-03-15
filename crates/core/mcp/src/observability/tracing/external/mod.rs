// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! External Tracing Integrations
//!
//! This module provides integration with external tracing systems like
//! OpenTelemetry, Jaeger, and Zipkin for distributed tracing capabilities.

mod config;
mod exporters;
mod traits;
mod tracer;
mod converters;

// Re-export public items
pub use config::ExternalTracingConfig;
pub use exporters::{OpenTelemetryExporter, JaegerExporter, ZipkinExporter};
pub use traits::SpanExporter;
pub use tracer::ExternalTracer;
pub use converters::{convert_to_otlp_format, convert_to_zipkin_format}; 