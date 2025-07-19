//! Metrics for circuit breaker operations
//!
//! This module defines metrics types used by the circuit breaker pattern.

use std::fmt;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use super::state::BreakerState;

/// Metrics for a circuit breaker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakerMetrics {
    /// Name of the circuit breaker
    pub name: String,
    
    /// Current state
    pub state: BreakerState,
    
    /// Number of successful operations
    pub success_count: u64,
    
    /// Number of failed operations
    pub failure_count: u64,
    
    /// Number of rejected operations
    pub rejected_count: u64,
    
    /// Number of timeout operations
    pub timeout_count: u64,
    
    /// Last time an operation failed
    pub last_failure_time: Option<DateTime<Utc>>,
    
    /// Last time an operation succeeded
    pub last_success_time: Option<DateTime<Utc>>,
}

impl Default for BreakerMetrics {
    fn default() -> Self {
        Self {
            name: "unknown".to_string(),
            state: BreakerState::Closed,
            success_count: 0,
            failure_count: 0,
            rejected_count: 0,
            timeout_count: 0,
            last_failure_time: None,
            last_success_time: None,
        }
    }
}

impl BreakerMetrics {
    /// Create a new metrics instance with the given name
    pub fn new(name: impl Into<String>) -> Self {
        let mut metrics = Self::default();
        metrics.name = name.into();
        metrics
    }
    
    /// Calculate the current failure rate
    pub fn failure_rate(&self) -> f64 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            return 0.0;
        }
        (self.failure_count as f64) / (total as f64)
    }
}

impl fmt::Display for BreakerMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let failure_rate = self.failure_rate() * 100.0;

        write!(
            f,
            "CircuitBreaker '{}' State: {}, Success: {}, Failure: {}, Rejected: {}, Timeouts: {}, Failure Rate: {:.1}%",
            self.name,
            self.state,
            self.success_count,
            self.failure_count,
            self.rejected_count,
            self.timeout_count,
            failure_rate
        )?;

        // Add additional state-specific information
        if self.state == BreakerState::Open {
            if let Some(time) = self.last_failure_time {
                write!(f, ", last failure: {}", time.format("%Y-%m-%d %H:%M:%S"))?;
            }
        }

        Ok(())
    }
}

impl BreakerMetrics {
    /// Format as a detailed string for logging
    pub fn details(&self) -> String {
        let failure_rate = self.failure_rate() * 100.0;
        
        let mut details = format!(
            "CircuitBreaker '{}' state={}, success={}, failure={}, rejected={}, timeout={}, failure_rate={:.1}%",
            self.name,
            self.state,
            self.success_count,
            self.failure_count,
            self.rejected_count,
            self.timeout_count,
            failure_rate
        );
        
        if let Some(time) = self.last_failure_time {
            details.push_str(&format!(", last_failure={}", time.format("%Y-%m-%d %H:%M:%S")));
        }
        
        if let Some(time) = self.last_success_time {
            details.push_str(&format!(", last_success={}", time.format("%Y-%m-%d %H:%M:%S")));
        }
        
        details
    }
} 