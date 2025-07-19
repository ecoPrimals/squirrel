//! External Tracer
//!
//! This module provides the ExternalTracer that combines internal span
//! management with external system integration.

use std::sync::{Arc, Mutex};

use crate::observability::ObservabilityResult;
use crate::observability::tracing::{Tracer, ActiveSpan};
use super::traits::SpanExporter;

/// Enhanced tracer with external system integration
pub struct ExternalTracer<E: SpanExporter> {
    /// Inner tracer for span management
    inner: Tracer,
    
    /// Exporter for sending spans to external systems
    pub exporter: E,
    
    /// Handle to the background flush task
    _flush_task: Option<tokio::task::JoinHandle<()>>,
    
    /// Whether this tracer has been initialized
    initialized: std::sync::atomic::AtomicBool,
}

impl<E: SpanExporter> ExternalTracer<E> {
    /// Create a new external tracer
    pub fn new(exporter: E) -> Self {
        Self {
            inner: Tracer::new(),
            exporter,
            _flush_task: None,
            initialized: std::sync::atomic::AtomicBool::new(false),
        }
    }
    
    /// Initialize the tracer
    pub async fn initialize(&mut self) -> ObservabilityResult<()> {
        // Initialize inner tracer
        self.inner.initialize()?;
        
        // Set initialized flag
        self.initialized.store(true, std::sync::atomic::Ordering::SeqCst);
        
        Ok(())
    }
    
    /// Start a new span
    pub fn start_span(&self, name: impl Into<String>) -> ObservabilityResult<Arc<Mutex<ActiveSpan>>> {
        self.inner.start_span(name)
    }
    
    /// Start a new span with parent
    pub fn start_span_with_parent(
        &self,
        name: impl Into<String>,
        parent_span: Option<Arc<Mutex<ActiveSpan>>>,
    ) -> ObservabilityResult<Arc<Mutex<ActiveSpan>>> {
        self.inner.start_span_with_parent(name, parent_span)
    }
    
    /// Get the current span
    pub fn current_span(&self) -> ObservabilityResult<Option<Arc<Mutex<ActiveSpan>>>> {
        self.inner.current_span()
    }
    
    /// Export completed spans to external system
    pub async fn export_completed_spans(&self) -> ObservabilityResult<usize> {
        // Get all spans
        let spans = {
            let spans_lock = self.inner.spans.lock().unwrap();
            spans_lock.values()
                .filter(|span| span.end_time.is_some()) // Only completed spans
                .cloned()
                .collect::<Vec<_>>()
        };
        
        // Export spans
        if !spans.is_empty() {
            self.exporter.export_spans(spans.clone()).await?;
        }
        
        Ok(spans.len())
    }
    
    /// Shutdown the tracer
    pub async fn shutdown(&self) -> ObservabilityResult<()> {
        // Export any completed spans
        let _ = self.export_completed_spans().await;
        
        // Shutdown the exporter
        self.exporter.shutdown().await
    }
} 