// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![forbid(unsafe_code)]
#![allow(clippy::missing_const_for_fn, clippy::significant_drop_tightening)] // const/drop tightening deferred

//! Context adapter for Squirrel
//!
//! This crate provides adapters for interfacing with the context subsystem,
//! including context management, persistence, and synchronization.
//!
//! ## Plugin Support
//!
//! The context adapter now supports plugins for transformations and format conversions:
//!
//! ```rust,no_run
//! use squirrel_context_adapter::adapter::{ContextAdapterConfig, create_context_adapter_with_plugins};
//! use squirrel_context::plugins::ContextPluginManager;
//! use std::sync::Arc;
//!
//! async fn example() {
//!     // Create a plugin manager
//!     let plugin_manager = Arc::new(ContextPluginManager::new());
//!     
//!     // Create configuration
//!     let config = ContextAdapterConfig::default();
//!     
//!     // Create adapter with plugin support
//!     let adapter = create_context_adapter_with_plugins(config, plugin_manager);
//!     
//!     // Initialize plugins
//!     adapter.initialize_plugins().await.expect("should succeed");
//!     
//!     // Now you can use transformations and adapters
//!     let data = serde_json::json!({ "example": "data" });
//!     
//!     // Transform data
//!     let transformed = adapter
//!         .transform_data("some.transformation", data.clone())
//!         .await
//!         .expect("should succeed");
//!     
//!     // Convert data format
//!     let converted = adapter.convert_data("some.adapter", data).await.expect("should succeed");
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

/// Context adapter implementation
pub mod adapter;

/// Re-export the main adapter module
pub use adapter::*;

/// Re-export common interfaces
pub use squirrel_interfaces::context::{
    AdapterMetadata, ContextAdapterPlugin, ContextPlugin, ContextTransformation,
};

/// Tests for the context adapter
#[cfg(test)]
pub mod tests;
