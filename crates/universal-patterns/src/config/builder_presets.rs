// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Named preset constructors for [`ConfigBuilder`].
//!
//! Each method returns a partially-filled builder for a specific deployment
//! profile or primal role. Call additional setters on the returned builder
//! before `.build()`.

use super::builder::ConfigBuilder;
use super::types::{LogFormat, LogLevel, PrimalType};

/// Preset constructors for specific primal configurations
impl ConfigBuilder {
    /// Create a configuration builder for Squirrel (AI coordinator)
    pub fn squirrel() -> Self {
        Self::new()
            .name("squirrel")
            .primal_type(PrimalType::Coordinator)
            .description("AI coordination and MCP protocol management")
            .port(8080)
    }

    /// Create a configuration builder for a security primal.
    ///
    /// Unlike `squirrel()` (self-knowledge), this provides sensible defaults
    /// for *any* security-domain primal without hardcoding a specific identity.
    pub fn security() -> Self {
        Self::new()
            .primal_type(PrimalType::Security)
            .description("Security and authentication management")
            .port(8081)
    }

    /// Create a configuration builder for an orchestration service.
    pub fn orchestration() -> Self {
        Self::new()
            .name("orchestration")
            .primal_type(PrimalType::Orchestration)
            .description("Orchestration and task management")
            .port(8082)
    }

    /// Create a configuration builder for development environment
    pub fn development() -> Self {
        Self::new()
            .environment("development")
            .log_level(LogLevel::Debug)
            .enable_structured_logging()
            .enable_tracing()
            .add_feature("debug_mode", true)
            .add_feature("hot_reload", true)
    }

    /// Create a configuration builder for production environment
    pub fn production() -> Self {
        Self::new()
            .environment("production")
            .log_level(LogLevel::Info)
            .log_format(LogFormat::Json)
            .enable_structured_logging()
            .enable_audit_logging()
            .enable_inter_primal_encryption()
            .enable_at_rest_encryption()
            .add_feature("debug_mode", false)
            .add_feature("hot_reload", false)
            .max_memory_mb(2048)
            .max_cpu_percent(80.0)
    }
}
