// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Multiplexing, backpressure, and chunk helpers.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::Result;

use super::types::{
    BackpressureController, BackpressureControllerConfig, ChunkType, MultiplexerConfig,
    StreamChunk, StreamMetrics, StreamMultiplexer, StreamStats,
};

impl StreamMultiplexer {
    /// Create new stream multiplexer
    pub(crate) async fn new(config: MultiplexerConfig) -> Result<Self> {
        Ok(Self {
            inputs: Arc::new(RwLock::new(HashMap::new())),
            outputs: Arc::new(RwLock::new(HashMap::new())),
            routing: Arc::new(RwLock::new(Vec::new())),
            config,
        })
    }
}

impl BackpressureController {
    /// Create new backpressure controller
    pub(crate) async fn new(config: BackpressureControllerConfig) -> Result<Self> {
        Ok(Self {
            buffer_states: Arc::new(RwLock::new(HashMap::new())),
            policies: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }
}

impl Default for StreamStats {
    fn default() -> Self {
        Self {
            chunks_processed: 0,
            bytes_processed: 0,
            chunks_per_second: 0.0,
            bytes_per_second: 0.0,
            error_count: 0,
            last_error: None,
            duration: Duration::from_secs(0),
            buffer_utilization: 0.0,
        }
    }
}

impl Default for StreamMetrics {
    fn default() -> Self {
        Self {
            total_streams_created: 0,
            active_streams: 0,
            total_chunks_processed: 0,
            total_bytes_processed: 0,
            current_throughput: 0.0,
            current_bandwidth: 0.0,
            error_rate: 0.0,
            avg_stream_duration: Duration::from_secs(0),
        }
    }
}

impl StreamChunk {
    /// Create new stream chunk
    pub fn new(
        stream_id: String,
        sequence: u64,
        chunk_type: ChunkType,
        data: serde_json::Value,
    ) -> Self {
        let data_str = data.to_string();
        Self {
            id: Uuid::new_v4().to_string(),
            stream_id,
            sequence,
            chunk_type,
            data,
            size_bytes: data_str.len(),
            timestamp: Utc::now(),
            is_final: false,
            metadata: HashMap::new(),
        }
    }

    /// Mark as final chunk
    pub fn mark_final(mut self) -> Self {
        self.is_final = true;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}
