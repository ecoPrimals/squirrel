//! Universal AI Provider - Pure Capability-Based Ecosystem Integration
//!
//! This module provides a completely agnostic AI provider that discovers and integrates
//! with ANY service in the ecosystem based purely on advertised capabilities. No
//! hardcoded provider names, primal identities, or service assumptions.
//!
//! Each service knows only its own capabilities and discovers others through
//! capability announcements. New primals and AI providers integrate automatically.

// Re-export all universal provider functionality from modular implementation
pub use universal::*;

// The actual implementation is now split across multiple modules for maintainability
pub mod universal;
