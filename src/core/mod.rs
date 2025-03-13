//! Core functionality for the Squirrel project
//!
//! This module provides the core functionality for the Squirrel project, including:
//! - Context management
//! - Command handling
//! - Error types
//! - Events
//! - Metrics

pub mod context;
pub mod commands;
pub mod error;
pub mod events;
pub mod metrics;

// Re-export commonly used types
pub use context::{Context, ContextConfig, ContextBuilder, ContextState, LifecycleStage};
pub use commands::Command;
pub use error::{Error, ContextError, CommandError, EventError, MetricsError};
pub use events::Event;
pub use metrics::Metrics;

// Re-export specific types from submodules
pub use context::{
    tracker::ContextTracker,
    sync::SyncManager,
    state::StateManager,
    recovery::RecoveryManager,
    persistence::PersistenceManager,
};
pub use commands::{
    lifecycle::CommandLifecycle,
    validation::CommandValidator,
    hooks::CommandHooks,
    resources::ResourceManager,
};
pub use events::{
    bus::EventBus,
    handler::EventHandler,
    filter::EventFilter,
};
pub use metrics::{
    collector::MetricsCollector,
    registry::MetricsRegistry,
    exporter::MetricsExporter,
}; 