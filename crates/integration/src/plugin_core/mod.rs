//! Plugin-Core integration module
//!
//! This module provides adapters and utilities for integrating the Plugin system with Core components.

mod adapter;
mod config;
pub mod error;
pub mod example_usage;
#[cfg(test)]
pub mod mock_plugins;
#[cfg(test)]
mod testing;
#[cfg(test)]
mod tests;

pub use adapter::PluginCoreAdapter;
pub use config::PluginCoreConfig; 