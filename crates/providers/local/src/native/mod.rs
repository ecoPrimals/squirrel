// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Native AI Provider Module
//!
//! This module provides native AI model integration to replace mock implementations
//! with actual AI processing capabilities. The module is organized into focused
//! submodules for better maintainability and separation of concerns.
//!
//! ## Architecture
//!
//! The native AI provider is organized into several focused areas:
//!
//! * **models**: Configuration types, data structures, and model-related code
//! * **runtime**: Core runtime functionality and AI processing methods
//!
//! ## Features
//!
//! * **Native Model Support**: Direct integration with local AI models
//! * **Multiple AI Tasks**: Text generation, completion, embedding, classification, etc.
//! * **Resource Management**: Memory, CPU, and concurrency limits
//! * **Health Monitoring**: Continuous health checks and performance metrics
//! * **Request Queuing**: Efficient request processing with queue management
//! * **Model Loading**: Dynamic model loading and unloading
//! * **Performance Tracking**: Detailed metrics and monitoring
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use native_provider::{NativeAIProvider, NativeAIConfig};
//! 
//! // Create configuration
//! let config = NativeAIConfig::default();
//! 
//! // Create provider
//! let provider = NativeAIProvider::new(config);
//! 
//! // Initialize and start processing
//! provider.initialize().await?;
//! 
//! // Process requests
//! let response = provider.queue_request(request).await?;
//! ```

// Core modules
pub mod models;
pub mod runtime;

// Re-export main types for convenience
pub use models::{
    NativeAIConfig, ModelConfig, ResourceLimits, PerformanceConfig, HealthCheckConfig,
    PerformanceMetrics, ModelInstance, ModelStatus, RequestInfo, QueuedRequest, ProviderState,
};
pub use runtime::NativeAIProvider;

// Re-export for backward compatibility
pub use models::NativeAIConfig as Config;
pub use runtime::NativeAIProvider as Provider; 