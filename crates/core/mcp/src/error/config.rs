// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Errors related to configuration management.

use thiserror::Error;

/// Errors that can occur during configuration loading or validation
#[derive(Error, Debug, Clone)]
pub enum ConfigError {
    /// Configuration file not found
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    /// Error parsing configuration file
    #[error("Configuration parsing error: {0}")]
    ParseError(String),

    /// Invalid configuration value
    #[error("Invalid configuration value for key '{key}': {message}")]
    InvalidValue { key: String, message: String },

    /// Missing required configuration key
    #[error("Missing required configuration key: {0}")]
    MissingKey(String),

    /// Generic internal configuration error
    #[error("Internal configuration error: {0}")]
    InternalError(String),
}
