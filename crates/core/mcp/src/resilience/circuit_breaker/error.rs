// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Error types for circuit breaker operations
//!
//! This module defines error types used by the circuit breaker pattern.

use std::fmt;
use std::error::Error;

/// Result type for circuit breaker operations
pub type BreakerResult<T> = Result<T, BreakerError>;

/// Error types for circuit breaker operations
#[derive(Debug, Clone)]
pub enum BreakerError {
    /// The circuit is open and requests are being rejected
    CircuitOpen {
        /// Name of the circuit breaker
        name: String,
        /// Milliseconds remaining until the circuit might transition to half-open
        reset_time_ms: u64,
    },
    
    /// A request timed out
    Timeout {
        /// Name of the circuit breaker
        name: String,
        /// Timeout duration in milliseconds
        timeout_ms: u64,
    },
    
    /// The underlying operation failed
    OperationFailed {
        /// Name of the circuit breaker
        name: String,
        /// Reason for the failure
        reason: String,
    },
    
    /// An internal error occurred in the circuit breaker
    Internal {
        /// Name of the circuit breaker
        name: String,
        /// Error details
        details: String,
    },
}

impl BreakerError {
    /// Create a new circuit open error
    pub fn circuit_open(name: impl Into<String>, reset_time_ms: u64) -> Self {
        Self::CircuitOpen {
            name: name.into(),
            reset_time_ms,
        }
    }
    
    /// Create a new timeout error
    pub fn timeout(name: impl Into<String>, timeout_ms: u64) -> Self {
        Self::Timeout {
            name: name.into(),
            timeout_ms,
        }
    }
    
    /// Create a new operation failed error
    pub fn operation_failed(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::OperationFailed {
            name: name.into(),
            reason: reason.into(),
        }
    }
    
    /// Create a new internal error
    pub fn internal(name: impl Into<String>, details: impl Into<String>) -> Self {
        Self::Internal {
            name: name.into(),
            details: details.into(),
        }
    }
    
    /// Get the name of the circuit breaker
    pub fn name(&self) -> &str {
        match self {
            Self::CircuitOpen { name, .. } => name,
            Self::Timeout { name, .. } => name,
            Self::OperationFailed { name, .. } => name,
            Self::Internal { name, .. } => name,
        }
    }
    
    /// Check if this is a circuit open error
    pub fn is_circuit_open(&self) -> bool {
        matches!(self, Self::CircuitOpen { .. })
    }
    
    /// Check if this is a timeout error
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout { .. })
    }
    
    /// Check if this is an operation failed error
    pub fn is_operation_failed(&self) -> bool {
        matches!(self, Self::OperationFailed { .. })
    }
    
    /// Check if this is an internal error
    pub fn is_internal(&self) -> bool {
        matches!(self, Self::Internal { .. })
    }
}

impl fmt::Display for BreakerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CircuitOpen { name, reset_time_ms } => {
                write!(f, "Circuit '{}' is OPEN (resets in {} ms)", name, reset_time_ms)
            },
            Self::Timeout { name, timeout_ms } => {
                write!(f, "Circuit '{}' operation timed out after {} ms", name, timeout_ms)
            },
            Self::OperationFailed { name, reason } => {
                write!(f, "Circuit '{}' operation failed: {}", name, reason)
            },
            Self::Internal { name, details } => {
                write!(f, "Circuit '{}' internal error: {}", name, details)
            },
        }
    }
}

impl Error for BreakerError {}

// From implementations for common error types
impl From<&str> for BreakerError {
    fn from(s: &str) -> Self {
        Self::operation_failed("unknown", s)
    }
}

impl From<String> for BreakerError {
    fn from(s: String) -> Self {
        Self::operation_failed("unknown", s)
    }
}

impl From<Box<dyn Error + Send + Sync>> for BreakerError {
    fn from(e: Box<dyn Error + Send + Sync>) -> Self {
        Self::operation_failed("unknown", e.to_string())
    }
} 