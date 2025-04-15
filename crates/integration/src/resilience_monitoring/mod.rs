//! Integration between the Resilience Framework and Monitoring System
//!
//! This module provides adapters and utilities for connecting circuit breakers
//! and other resilience components to the monitoring system. This enables:
//!
//! 1. Metrics collection from resilience components
//! 2. Health checks based on resilience status
//! 3. Alerts for resilience-related events
//! 4. Recovery actions based on monitoring alerts

mod adapter;

pub use adapter::{
    CircuitBreakerAlertHandler,
    ResilienceMonitoringAdapter,
    ResilienceMonitoringConfig,
}; 