//! Dashboard module for Squirrel monitoring system
//!
//! This module provides functionality for creating and managing dashboards.

pub mod components;
pub mod config;
pub mod component;
pub mod error;
pub mod manager;
pub mod plugins;
pub mod security;
pub mod secure_server;
pub mod stats;

// Re-exports
pub use component::{DashboardComponent, Update};
pub use error::DashboardError;
pub use manager::DashboardManager;
pub use plugins::DashboardPluginRegistry;