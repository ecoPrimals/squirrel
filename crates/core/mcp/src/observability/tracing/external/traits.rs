//! External Tracing Traits
//!
//! This module defines the trait interfaces for external tracing
//! system integrations.

use async_trait::async_trait;
use crate::observability::ObservabilityResult;
use crate::observability::tracing::types::Span;

/// Trait for exporting spans to external systems
#[async_trait]
pub trait SpanExporter: Send + Sync {
    /// Export a batch of spans to an external system
    async fn export_spans(&self, spans: Vec<Span>) -> ObservabilityResult<()>;
    
    /// Shutdown the exporter
    async fn shutdown(&self) -> ObservabilityResult<()>;
} 