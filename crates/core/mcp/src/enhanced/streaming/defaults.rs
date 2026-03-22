// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Default configurations for stream manager and per-stream settings.

use std::time::Duration;

use crate::resilience::retry::RetryConfig;

use super::types::{
    BackpressureConfig, BackpressureStrategy, QualityConfig, StreamConfig, StreamManagerConfig,
    StreamPriority,
};

impl Default for StreamManagerConfig {
    fn default() -> Self {
        // Load unified config for environment-aware timeout values
        let config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .and_then(|loaded| loaded.try_into_config().ok());

        let (default_timeout, cleanup_interval) = if let Some(cfg) = config {
            let timeout = cfg
                .timeouts
                .get_custom_timeout("stream_default")
                .unwrap_or_else(|| Duration::from_secs(300)); // 5 minutes
            let cleanup = cfg
                .timeouts
                .get_custom_timeout("stream_cleanup")
                .unwrap_or_else(|| Duration::from_secs(60)); // 1 minute
            (timeout, cleanup)
        } else {
            (Duration::from_secs(300), Duration::from_secs(60))
        };

        Self {
            max_concurrent_streams: 1000,
            default_timeout,
            cleanup_interval,
            enable_metrics: true,
            default_buffer_size: 8192,
        }
    }
}

impl Default for StreamConfig {
    fn default() -> Self {
        // Load unified config for environment-aware timeout values
        let config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .and_then(|loaded| loaded.try_into_config().ok());

        let timeout = if let Some(cfg) = config {
            cfg.timeouts
                .get_custom_timeout("stream_timeout")
                .unwrap_or_else(|| Duration::from_secs(300))
        } else {
            Duration::from_secs(300)
        };

        Self {
            buffer_size: 8192,
            max_chunk_size: 65536, // 64KB
            timeout,
            backpressure: BackpressureConfig::default(),
            quality: QualityConfig::default(),
            retry: RetryConfig::default(),
        }
    }
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            high_water_mark: 0.8,
            low_water_mark: 0.5,
            strategy: BackpressureStrategy::DropOldest,
        }
    }
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            level: 1.0,
            adaptive: true,
            min_quality: 0.1,
            max_quality: 1.0,
            compression_enabled: false,
            priority: StreamPriority::Normal,
        }
    }
}
