//! Event system for observability framework
//!
//! This module provides event handling and pub/sub functionality
//! for the observability system.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;
use tokio::sync::{RwLock, mpsc};

use crate::observability::{ObservabilityError, ObservabilityResult};
use crate::observability::health::HealthStatus;

/// Event bus for publishing and subscribing to observability events
pub struct EventBus {
    subscribers: Arc<RwLock<Vec<mpsc::UnboundedSender<ObservabilityEvent>>>>,
    events_processed: Arc<AtomicU64>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(Vec::new())),
            events_processed: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Subscribe to observability events
    pub async fn subscribe(&self) -> ObservabilityResult<mpsc::UnboundedReceiver<ObservabilityEvent>> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.subscribers.write().await.push(tx);
        Ok(rx)
    }

    /// Publish an event to all subscribers
    pub async fn publish(&self, event: ObservabilityEvent) -> ObservabilityResult<()> {
        let subscribers = self.subscribers.read().await;
        let mut failed_subscribers = Vec::new();

        for (index, subscriber) in subscribers.iter().enumerate() {
            if let Err(_) = subscriber.send(event.clone()) {
                failed_subscribers.push(index);
            }
        }

        // Remove failed subscribers
        if !failed_subscribers.is_empty() {
            drop(subscribers);
            let mut subscribers = self.subscribers.write().await;
            for &index in failed_subscribers.iter().rev() {
                subscribers.remove(index);
            }
        }

        self.events_processed.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    /// Get the number of events processed
    pub async fn get_events_processed_count(&self) -> u64 {
        self.events_processed.load(Ordering::SeqCst)
    }

    /// Get the number of active subscribers
    pub async fn get_subscriber_count(&self) -> usize {
        self.subscribers.read().await.len()
    }

    /// Shutdown the event bus
    pub async fn shutdown(&self) -> ObservabilityResult<()> {
        let subscribers = self.subscribers.read().await;
        
        // Close all subscriber channels
        for subscriber in subscribers.iter() {
            // Channels will be closed when dropped
        }
        
        Ok(())
    }
}

/// Events that can be published through the observability event bus
#[derive(Debug, Clone)]
pub enum ObservabilityEvent {
    /// Component health status changed
    HealthStatusChanged {
        component_id: String,
        old_status: HealthStatus,
        new_status: HealthStatus,
        timestamp: SystemTime,
    },
    /// Metric threshold exceeded
    MetricThresholdExceeded {
        metric_name: String,
        current_value: f64,
        threshold: f64,
        timestamp: SystemTime,
    },
    /// Alert triggered
    AlertTriggered {
        alert_id: String,
        severity: String,
        message: String,
        timestamp: SystemTime,
    },
    /// Dashboard synchronization error
    DashboardSyncError {
        component: String,
        error: String,
    },
    /// Performance anomaly detected
    PerformanceAnomaly {
        metric_type: String,
        deviation: f64,
        timestamp: SystemTime,
    },
    /// Component registered
    ComponentRegistered {
        component_id: String,
        component_type: String,
        timestamp: SystemTime,
    },
    /// Component unregistered
    ComponentUnregistered {
        component_id: String,
        timestamp: SystemTime,
    },
    /// Trace span completed
    TraceSpanCompleted {
        span_id: String,
        operation: String,
        duration_ms: u64,
        success: bool,
        timestamp: SystemTime,
    },
    /// Metrics collection completed
    MetricsCollected {
        metrics_count: usize,
        collection_duration_ms: u64,
        timestamp: SystemTime,
    },
}

impl ObservabilityEvent {
    /// Get the timestamp of the event
    pub fn timestamp(&self) -> SystemTime {
        match self {
            ObservabilityEvent::HealthStatusChanged { timestamp, .. } => *timestamp,
            ObservabilityEvent::MetricThresholdExceeded { timestamp, .. } => *timestamp,
            ObservabilityEvent::AlertTriggered { timestamp, .. } => *timestamp,
            ObservabilityEvent::DashboardSyncError { .. } => SystemTime::now(),
            ObservabilityEvent::PerformanceAnomaly { timestamp, .. } => *timestamp,
            ObservabilityEvent::ComponentRegistered { timestamp, .. } => *timestamp,
            ObservabilityEvent::ComponentUnregistered { timestamp, .. } => *timestamp,
            ObservabilityEvent::TraceSpanCompleted { timestamp, .. } => *timestamp,
            ObservabilityEvent::MetricsCollected { timestamp, .. } => *timestamp,
        }
    }

    /// Get the event type as a string
    pub fn event_type(&self) -> &'static str {
        match self {
            ObservabilityEvent::HealthStatusChanged { .. } => "health_status_changed",
            ObservabilityEvent::MetricThresholdExceeded { .. } => "metric_threshold_exceeded",
            ObservabilityEvent::AlertTriggered { .. } => "alert_triggered",
            ObservabilityEvent::DashboardSyncError { .. } => "dashboard_sync_error",
            ObservabilityEvent::PerformanceAnomaly { .. } => "performance_anomaly",
            ObservabilityEvent::ComponentRegistered { .. } => "component_registered",
            ObservabilityEvent::ComponentUnregistered { .. } => "component_unregistered",
            ObservabilityEvent::TraceSpanCompleted { .. } => "trace_span_completed",
            ObservabilityEvent::MetricsCollected { .. } => "metrics_collected",
        }
    }

    /// Get the component ID associated with this event, if any
    pub fn component_id(&self) -> Option<&str> {
        match self {
            ObservabilityEvent::HealthStatusChanged { component_id, .. } => Some(component_id),
            ObservabilityEvent::DashboardSyncError { component, .. } => Some(component),
            ObservabilityEvent::ComponentRegistered { component_id, .. } => Some(component_id),
            ObservabilityEvent::ComponentUnregistered { component_id, .. } => Some(component_id),
            _ => None,
        }
    }
} 