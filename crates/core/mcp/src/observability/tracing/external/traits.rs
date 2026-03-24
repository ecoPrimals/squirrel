// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! External Tracing Traits
//!
//! This module defines the trait interfaces for external tracing
//! system integrations.

use std::future::Future;
use crate::observability::ObservabilityResult;
use crate::observability::tracing::types::Span;

/// Trait for exporting spans to external systems
pub trait SpanExporter: Send + Sync {
    /// Export a batch of spans to an external system
    fn export_spans(&self, spans: Vec<Span>) -> impl Future<Output = ObservabilityResult<()>> + Send;
    
    /// Shutdown the exporter
    fn shutdown(&self) -> impl Future<Output = ObservabilityResult<()>> + Send;
} 