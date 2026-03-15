// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Infrastructure module for error handling, configuration, logging, and utilities

// Error handling
pub mod error;
pub use error::{
    EnhancedError, ErrorCategory, ErrorContext, ErrorSeverity, PluginError, ValidationError,
};

// Configuration
pub mod config;
pub use config::{
    HttpConfig, LoggingConfig, McpClientConfig, NetworkConfig, PerformanceConfig, PluginConfig,
    PluginMetadata, PluginSdkConfig, SandboxConfig, SecurityLevel,
};

// Logging
pub mod logging;
pub use logging::{
    LogEntry, LogLevel, Logger, LoggerConfig, ScopedLogger, SerializableLoggerConfig,
};

// Utilities
pub mod utils;
pub use utils::{
    console_log, current_timestamp, current_timestamp_iso, generate_uuid, js_to_serde, safe_lock,
    safe_lock_or_fallback, serde_to_js, set_panic_hook, to_js_result,
};
