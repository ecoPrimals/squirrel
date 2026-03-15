// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Tests for the resilience framework

mod circuit_breaker_tests;
mod retry_tests;
mod recovery_tests;
mod state_sync_tests;
mod integration_tests;

use std::error::Error;
use std::fmt;

/// Test error type for resilience tests
#[derive(Debug, Clone)]
pub enum TestError {
    /// Generic error for testing
    Generic(String),
    /// Error simulating a timeout
    Timeout(String),
    /// Error simulating a connection issue
    Connection(String),
    /// Error simulating a resource issue
    Resource(String),
    /// Error simulating an authentication issue
    Authentication(String),
}

impl TestError {
    /// Create a new generic test error
    pub fn generic<S: Into<String>>(message: S) -> Self {
        Self::Generic(message.into())
    }
    
    /// Create a new timeout test error
    pub fn timeout<S: Into<String>>(message: S) -> Self {
        Self::Timeout(message.into())
    }
    
    /// Create a new connection test error
    pub fn connection<S: Into<String>>(message: S) -> Self {
        Self::Connection(message.into())
    }
    
    /// Create a new resource test error
    pub fn resource<S: Into<String>>(message: S) -> Self {
        Self::Resource(message.into())
    }
    
    /// Create a new authentication test error
    pub fn authentication<S: Into<String>>(message: S) -> Self {
        Self::Authentication(message.into())
    }
}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic(msg) => write!(f, "Test error: {}", msg),
            Self::Timeout(msg) => write!(f, "Timeout error: {}", msg),
            Self::Connection(msg) => write!(f, "Connection error: {}", msg),
            Self::Resource(msg) => write!(f, "Resource error: {}", msg),
            Self::Authentication(msg) => write!(f, "Authentication error: {}", msg),
        }
    }
}

impl Error for TestError {} 