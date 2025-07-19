//! Tracer Implementation
//!
//! This module provides the main Tracer implementation for creating and managing
//! spans within the distributed tracing system.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use crate::observability::{ObservabilityError, ObservabilityResult};
use super::types::{Span, TracerConfig};
use super::active_span::{ActiveSpan, set_current_span, current_span};

/// Main tracer for creating and managing spans
#[derive(Debug)]
pub struct Tracer {
    /// Tracer configuration
    config: RwLock<TracerConfig>,
    /// All spans created by this tracer
    pub(crate) spans: Mutex<HashMap<String, Span>>,
}

impl Tracer {
    /// Create a new tracer
    pub fn new() -> Self {
        Self {
            config: RwLock::new(TracerConfig::default()),
            spans: Mutex::new(HashMap::new()),
        }
    }

    /// Initialize the tracer
    pub fn initialize(&self) -> ObservabilityResult<()> {
        // Tracer is already initialized upon creation
        Ok(())
    }

    /// Set the tracer configuration
    pub fn set_config(&self, config: TracerConfig) -> ObservabilityResult<()> {
        let mut tracer_config = self.config.write()
            .map_err(|e| ObservabilityError::Internal(format!("Failed to acquire config lock: {}", e)))?;
        *tracer_config = config;
        Ok(())
    }

    /// Get the tracer configuration
    pub fn get_config(&self) -> ObservabilityResult<TracerConfig> {
        let config = self.config.read()
            .map_err(|e| ObservabilityError::Internal(format!("Failed to acquire config lock: {}", e)))?;
        Ok(config.clone())
    }

    /// Start a new span
    pub fn start_span(&self, name: impl Into<String>) -> ObservabilityResult<Arc<Mutex<ActiveSpan>>> {
        self.start_span_with_parent(name, None)
    }

    /// Start a new span with a parent
    pub fn start_span_with_parent(
        &self,
        name: impl Into<String>,
        parent_span: Option<Arc<Mutex<ActiveSpan>>>,
    ) -> ObservabilityResult<Arc<Mutex<ActiveSpan>>> {
        let config = self.config.read()
            .map_err(|e| ObservabilityError::Internal(format!("Failed to acquire config lock: {}", e)))?;

        if !config.enabled {
            return Err(ObservabilityError::Internal("Tracing is disabled".to_string()));
        }

        // Determine trace ID and parent ID
        let (trace_id, parent_id) = if let Some(parent) = &parent_span {
            let parent_span = parent.lock()
                .map_err(|e| ObservabilityError::Internal(format!("Failed to lock parent span: {}", e)))?;
            (parent_span.span().trace_id().to_string(), Some(parent_span.span().id().to_string()))
        } else {
            // Create new trace ID if no parent
            (uuid::Uuid::new_v4().to_string(), None)
        };

        // Create the span
        let span = Span::new(name, trace_id, parent_id);
        let span_id = span.id().to_string();

        // Store the span
        {
            let mut spans = self.spans.lock()
                .map_err(|e| ObservabilityError::Internal(format!("Failed to acquire spans lock: {}", e)))?;
            
            // Check if we've exceeded max spans
            if spans.len() >= config.max_spans {
                // Remove oldest spans (simple FIFO eviction)
                let keys_to_remove: Vec<String> = spans.keys().take(spans.len() - config.max_spans + 1).cloned().collect();
                for key in keys_to_remove {
                    spans.remove(&key);
                }
            }
            
            spans.insert(span_id.clone(), span.clone());
        }

        // Create active span
        let active_span = Arc::new(Mutex::new(ActiveSpan::new(span)));

        // Set as current span if no parent
        if parent_span.is_none() {
            set_current_span(Some(active_span.clone()));
        }

        Ok(active_span)
    }

    /// Get the current span
    pub fn current_span(&self) -> ObservabilityResult<Option<Arc<Mutex<ActiveSpan>>>> {
        Ok(current_span())
    }

    /// Get a span by ID
    pub fn get_span(&self, span_id: &str) -> ObservabilityResult<Option<Span>> {
        let spans = self.spans.lock()
            .map_err(|e| ObservabilityError::Internal(format!("Failed to acquire spans lock: {}", e)))?;
        Ok(spans.get(span_id).cloned())
    }

    /// Get all spans for a trace
    pub fn get_trace_spans(&self, trace_id: &str) -> ObservabilityResult<Vec<Span>> {
        let spans = self.spans.lock()
            .map_err(|e| ObservabilityError::Internal(format!("Failed to acquire spans lock: {}", e)))?;
        
        let trace_spans = spans.values()
            .filter(|span| span.trace_id() == trace_id)
            .cloned()
            .collect();
        
        Ok(trace_spans)
    }

    /// Clear all spans
    pub fn clear_spans(&self) -> ObservabilityResult<()> {
        let mut spans = self.spans.lock()
            .map_err(|e| ObservabilityError::Internal(format!("Failed to acquire spans lock: {}", e)))?;
        spans.clear();
        Ok(())
    }

    /// Export a batch of spans as snapshots
    pub async fn export_spans_batch(&self, max_batch_size: usize) -> ObservabilityResult<Vec<super::types::SpanSnapshot>> {
        let spans = self.spans.lock()
            .map_err(|e| ObservabilityError::Internal(format!("Failed to acquire spans lock: {}", e)))?;
        
        let mut snapshots = Vec::new();
        let current_time = std::time::SystemTime::now();
        
        for span in spans.values().take(max_batch_size) {
            let events = span.events().iter().map(|event| {
                super::types::SpanEventSnapshot {
                    name: event.name().to_string(),
                    timestamp: current_time, // Convert from Instant to SystemTime
                    attributes: event.attributes().clone(),
                }
            }).collect();
            
            let duration_ms = span.duration().map(|d| d.as_millis() as u64);
            
            snapshots.push(super::types::SpanSnapshot {
                id: span.id().to_string(),
                trace_id: span.trace_id().to_string(),
                parent_id: span.parent_id().map(|s| s.to_string()),
                name: span.name().to_string(),
                start_time: current_time, // Would need to convert from Instant to SystemTime in real impl
                end_time: if span.is_active() { None } else { Some(current_time) },
                attributes: span.attributes().clone(),
                events,
                status: span.status(),
                duration_ms,
            });
        }
        
        Ok(snapshots)
    }

    /// Get count of active spans
    pub async fn get_active_spans_count(&self) -> ObservabilityResult<usize> {
        let spans = self.spans.lock()
            .map_err(|e| ObservabilityError::Internal(format!("Failed to acquire spans lock: {}", e)))?;
        
        let active_count = spans.values()
            .filter(|span| span.is_active())
            .count();
        
        Ok(active_count)
    }
}

impl Default for Tracer {
    fn default() -> Self {
        Self::new()
    }
} 