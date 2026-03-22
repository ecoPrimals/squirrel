// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! [`StreamManager`] lifecycle and orchestration.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::error::Result;

use super::types::{
    ActiveStream, BackpressureController, BackpressureControllerConfig, MultiplexerConfig,
    StreamChunk, StreamConfig, StreamHandle, StreamManager, StreamManagerConfig, StreamMetrics,
    StreamMultiplexer, StreamSource, StreamStats, StreamStatus, StreamType,
};

impl StreamManager {
    /// Create new stream manager
    pub async fn new(config: StreamManagerConfig) -> Result<Self> {
        let multiplexer = Arc::new(StreamMultiplexer::new(MultiplexerConfig::default()).await?);
        let backpressure = Arc::new(
            BackpressureController::new(BackpressureControllerConfig::default()).await?,
        );

        let manager = Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            multiplexer,
            backpressure,
            config,
            metrics: Arc::new(Mutex::new(StreamMetrics::default())),
        };

        // Start background tasks
        manager.start_cleanup_task();
        if manager.config.enable_metrics {
            manager.start_metrics_collection();
        }

        info!("Stream manager initialized");
        Ok(manager)
    }

    /// Create new stream
    #[instrument(skip(self, handle))]
    pub async fn create_stream(
        &self,
        stream_type: StreamType,
        source: StreamSource,
        handle: Box<dyn StreamHandle>,
        config: Option<StreamConfig>,
    ) -> Result<String> {
        let stream_id = Uuid::new_v4().to_string();
        let config = config.unwrap_or_default();

        let stream = ActiveStream {
            id: stream_id.clone(),
            stream_type: stream_type.clone(),
            source,
            handle,
            config,
            created_at: Utc::now(),
            last_activity: Utc::now(),
            metrics: StreamStats::default(),
        };

        let mut streams = self.streams.write().await;

        // Check concurrent stream limit
        if streams.len() >= self.config.max_concurrent_streams {
            return Err(crate::error::types::MCPError::ResourceExhausted(
                "Maximum concurrent streams reached".to_string(),
            ));
        }

        streams.insert(stream_id.clone(), stream);

        // Update metrics
        self.update_metrics_stream_created().await;

        info!("Created stream: {} of type {:?}", stream_id, stream_type);
        Ok(stream_id)
    }

    /// Start stream
    pub async fn start_stream(&self, stream_id: &str) -> Result<()> {
        let mut streams = self.streams.write().await;
        let stream = streams.get_mut(stream_id).ok_or_else(|| {
            crate::error::types::MCPError::NotFound(format!("Stream not found: {}", stream_id))
        })?;

        stream.handle.start().await?;
        stream.last_activity = Utc::now();

        info!("Started stream: {}", stream_id);
        Ok(())
    }

    /// Stop stream
    pub async fn stop_stream(&self, stream_id: &str) -> Result<()> {
        let mut streams = self.streams.write().await;
        let stream = streams.get_mut(stream_id).ok_or_else(|| {
            crate::error::types::MCPError::NotFound(format!("Stream not found: {}", stream_id))
        })?;

        stream.handle.stop().await?;
        stream.last_activity = Utc::now();

        info!("Stopped stream: {}", stream_id);
        Ok(())
    }

    /// Get stream status
    pub async fn get_stream_status(&self, stream_id: &str) -> Result<StreamStatus> {
        let streams = self.streams.read().await;
        let stream = streams.get(stream_id).ok_or_else(|| {
            crate::error::types::MCPError::NotFound(format!("Stream not found: {}", stream_id))
        })?;

        Ok(stream.handle.status())
    }

    /// Get next chunk from stream
    pub async fn next_chunk(&self, stream_id: &str) -> Result<Option<StreamChunk>> {
        let mut streams = self.streams.write().await;
        let stream = streams.get_mut(stream_id).ok_or_else(|| {
            crate::error::types::MCPError::NotFound(format!("Stream not found: {}", stream_id))
        })?;

        let chunk = stream.handle.next_chunk().await?;

        if chunk.is_some() {
            stream.last_activity = Utc::now();
        }

        Ok(chunk)
    }

    /// Remove completed or failed streams
    pub async fn cleanup_streams(&self) -> Result<u64> {
        let mut streams = self.streams.write().await;
        let mut to_remove = Vec::new();

        for (stream_id, stream) in streams.iter() {
            let status = stream.handle.status();
            if matches!(status, StreamStatus::Completed | StreamStatus::Failed(_)) {
                to_remove.push(stream_id.clone());
            }
        }

        let removed_count = to_remove.len() as u64;
        for stream_id in to_remove {
            streams.remove(&stream_id);
            info!("Cleaned up stream: {}", stream_id);
        }

        Ok(removed_count)
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> StreamMetrics {
        self.metrics.lock().await.clone()
    }

    /// List active streams
    pub async fn list_streams(&self) -> Result<Vec<String>> {
        let streams = self.streams.read().await;
        Ok(streams.keys().cloned().collect())
    }

    /// Start cleanup task
    fn start_cleanup_task(&self) {
        let streams = self.streams.clone();
        let cleanup_interval = self.config.cleanup_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);

            loop {
                interval.tick().await;

                // Cleanup logic would be implemented here
                debug!("Running stream cleanup task");
            }
        });
    }

    /// Start metrics collection
    fn start_metrics_collection(&self) {
        let metrics = self.metrics.clone();
        let streams = self.streams.clone();

        tokio::spawn(async move {
            // Load unified config for environment-aware metrics interval
            let config = squirrel_mcp_config::unified::ConfigLoader::load()
                .ok()
                .and_then(|loaded| loaded.try_into_config().ok());

            let interval_duration = if let Some(cfg) = config {
                cfg.timeouts
                    .get_custom_timeout("stream_metrics_interval")
                    .unwrap_or_else(|| Duration::from_secs(5))
            } else {
                Duration::from_secs(5)
            };

            let mut interval = tokio::time::interval(interval_duration);

            loop {
                interval.tick().await;

                // Update metrics
                let stream_count = {
                    let streams = streams.read().await;
                    streams.len() as u64
                };

                let mut metrics = metrics.lock().await;
                metrics.active_streams = stream_count;
            }
        });
    }

    /// Update metrics when stream is created
    async fn update_metrics_stream_created(&self) {
        let mut metrics = self.metrics.lock().await;
        metrics.total_streams_created += 1;
        metrics.active_streams += 1;
    }
}
