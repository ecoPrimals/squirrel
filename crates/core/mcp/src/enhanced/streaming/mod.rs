// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Streaming Manager
//!
//! This module provides universal streaming capabilities for ANY AI system,
//! with backpressure handling, multiplexing, and real-time data processing.

mod components;
mod defaults;
mod manager;
mod types;

pub use types::{
    ActiveStream, BackpressureAction, BackpressureConfig, BackpressureController,
    BackpressureControllerConfig, BackpressurePolicy, BackpressureStrategy, BufferState,
    ChunkType, MultiplexerConfig, QualityConfig, RoutingRule, StreamChunk, StreamConfig,
    StreamHandle, StreamManager, StreamManagerConfig, StreamMetrics, StreamMultiplexer,
    StreamPriority, StreamSource, StreamStats, StreamStatus, StreamType, TransformConfig,
    TransformType,
};
