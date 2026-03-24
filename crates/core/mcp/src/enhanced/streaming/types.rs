// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Stream domain types, configuration, and the [`StreamHandle`] trait.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex, RwLock};

use crate::error::Result;
use crate::resilience::retry::{BackoffStrategy, RetryConfig};

/// Universal streaming manager - handles streaming from ANY AI system
#[derive(Debug)]
pub struct StreamManager {
    /// Active streams
    pub(super) streams: Arc<RwLock<HashMap<String, ActiveStream>>>,

    /// Stream multiplexer
    pub(super) multiplexer: Arc<StreamMultiplexer>,

    /// Backpressure controller
    pub(super) backpressure: Arc<BackpressureController>,

    /// Configuration
    pub(super) config: StreamManagerConfig,

    /// Metrics
    pub(super) metrics: Arc<Mutex<StreamMetrics>>,
}

/// Active stream information
#[derive(Debug)]
pub struct ActiveStream {
    /// Stream ID
    pub id: String,

    /// Stream type
    pub stream_type: StreamType,

    /// Stream source
    pub source: StreamSource,

    /// Stream handle
    pub handle: Box<dyn StreamHandle>,

    /// Stream configuration
    pub config: StreamConfig,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Last activity
    pub last_activity: DateTime<Utc>,

    /// Stream metrics
    pub metrics: StreamStats,
}

/// Universal stream types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StreamType {
    /// AI text generation stream
    AITextGeneration,

    /// AI image generation stream
    AIImageGeneration,

    /// AI audio generation stream
    AIAudioGeneration,

    /// AI video generation stream
    AIVideoGeneration,

    /// Real-time data stream
    RealTimeData,

    /// Event stream
    EventStream,

    /// Metrics stream
    MetricsStream,

    /// Log stream
    LogStream,

    /// Custom stream type
    Custom(String),

    /// Future stream types
    Future(String),
}

/// Stream source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamSource {
    /// Source type (ai_provider, tool, system, etc.)
    pub source_type: String,

    /// Source identifier
    pub source_id: String,

    /// Source name
    pub source_name: String,

    /// Source metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Stream handle trait for any type of stream
#[async_trait::async_trait]
pub trait StreamHandle: Send + Sync + std::fmt::Debug {
    /// Start the stream
    async fn start(&mut self) -> Result<()>;

    /// Stop the stream
    async fn stop(&mut self) -> Result<()>;

    /// Pause the stream
    async fn pause(&mut self) -> Result<()>;

    /// Resume the stream
    async fn resume(&mut self) -> Result<()>;

    /// Get stream status
    fn status(&self) -> StreamStatus;

    /// Get next chunk of data
    async fn next_chunk(&mut self) -> Result<Option<StreamChunk>>;

    /// Check if stream is complete
    fn is_complete(&self) -> bool;

    /// Get stream statistics
    fn get_stats(&self) -> StreamStats;
}

/// Stream status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StreamStatus {
    /// Stream is starting
    Starting,

    /// Stream is running
    Running,

    /// Stream is paused
    Paused,

    /// Stream is stopping
    Stopping,

    /// Stream is stopped
    Stopped,

    /// Stream completed successfully
    Completed,

    /// Stream failed
    Failed(String),
}

/// Universal stream chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// Chunk ID
    pub id: String,

    /// Stream ID this chunk belongs to
    pub stream_id: String,

    /// Chunk sequence number
    pub sequence: u64,

    /// Chunk type
    pub chunk_type: ChunkType,

    /// Chunk data
    pub data: serde_json::Value,

    /// Chunk size in bytes
    pub size_bytes: usize,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Is this the final chunk?
    pub is_final: bool,

    /// Chunk metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Chunk types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChunkType {
    /// Text data
    Text,

    /// Binary data
    Binary,

    /// JSON data
    Json,

    /// Image data
    Image,

    /// Audio data
    Audio,

    /// Video data
    Video,

    /// Metadata chunk
    Metadata,

    /// Control chunk
    Control,

    /// Custom chunk type
    Custom(String),
}

/// Stream configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    /// Buffer size for the stream
    pub buffer_size: usize,

    /// Maximum chunk size
    pub max_chunk_size: usize,

    /// Stream timeout
    pub timeout: Duration,

    /// Backpressure settings
    pub backpressure: BackpressureConfig,

    /// Quality settings
    pub quality: QualityConfig,

    /// Retry settings
    pub retry: RetryConfig,
}

/// Backpressure configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpressureConfig {
    /// Enable backpressure
    pub enabled: bool,

    /// High water mark (buffer percentage)
    pub high_water_mark: f32,

    /// Low water mark (buffer percentage)
    pub low_water_mark: f32,

    /// Strategy to use when backpressure is triggered
    pub strategy: BackpressureStrategy,
}

/// Backpressure strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackpressureStrategy {
    /// Drop oldest chunks
    DropOldest,

    /// Drop newest chunks
    DropNewest,

    /// Pause stream
    PauseStream,

    /// Reduce quality
    ReduceQuality,

    /// Custom strategy
    Custom(String),
}

/// Quality configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    /// Quality level (0.0 - 1.0)
    pub level: f32,

    /// Adaptive quality enabled
    pub adaptive: bool,

    /// Minimum quality
    pub min_quality: f32,

    /// Maximum quality
    pub max_quality: f32,

    /// Compression enabled
    pub compression_enabled: bool,

    /// Stream priority
    pub priority: StreamPriority,
}

/// Stream statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStats {
    /// Total chunks processed
    pub chunks_processed: u64,

    /// Total bytes processed
    pub bytes_processed: u64,

    /// Current chunks per second
    pub chunks_per_second: f64,

    /// Current bytes per second
    pub bytes_per_second: f64,

    /// Error count
    pub error_count: u64,

    /// Last error
    pub last_error: Option<String>,

    /// Stream duration
    pub duration: Duration,

    /// Buffer utilization (0.0 - 1.0)
    pub buffer_utilization: f32,
}

/// Stream multiplexer for handling multiple streams
#[derive(Debug)]
pub struct StreamMultiplexer {
    /// Input streams
    pub(super) inputs: Arc<RwLock<HashMap<String, broadcast::Receiver<StreamChunk>>>>,

    /// Output channels
    pub(super) outputs: Arc<RwLock<HashMap<String, broadcast::Sender<StreamChunk>>>>,

    /// Routing rules
    pub(super) routing: Arc<RwLock<Vec<RoutingRule>>>,

    /// Configuration
    pub(super) config: MultiplexerConfig,
}

/// Routing rule for stream multiplexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// Rule name
    pub name: String,

    /// Source stream pattern
    pub source_pattern: String,

    /// Target output channels
    pub target_channels: Vec<String>,

    /// Filter condition
    pub filter: Option<serde_json::Value>,

    /// Transform configuration
    pub transform: Option<TransformConfig>,
}

/// Transform configuration for stream data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformConfig {
    /// Transform type
    pub transform_type: TransformType,

    /// Transform parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Transform types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformType {
    /// Pass through (no transformation)
    PassThrough,

    /// Format conversion
    FormatConversion,

    /// Data aggregation
    Aggregation,

    /// Data filtering
    Filtering,

    /// Custom transformation
    Custom(String),
}

/// Backpressure controller
#[derive(Debug)]
pub struct BackpressureController {
    /// Stream buffer states
    pub(super) buffer_states: Arc<RwLock<HashMap<String, BufferState>>>,

    /// Backpressure policies
    pub(super) policies: Arc<RwLock<HashMap<String, BackpressurePolicy>>>,

    /// Configuration
    pub(super) config: BackpressureControllerConfig,
}

/// Buffer state for a stream
#[derive(Debug, Clone)]
pub struct BufferState {
    /// Current buffer size
    pub current_size: usize,

    /// Maximum buffer size
    pub max_size: usize,

    /// Buffer utilization percentage
    pub utilization: f32,

    /// Last update time
    pub last_update: DateTime<Utc>,
}

/// Backpressure policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpressurePolicy {
    /// Policy name
    pub name: String,

    /// Trigger threshold
    pub trigger_threshold: f32,

    /// Recovery threshold
    pub recovery_threshold: f32,

    /// Actions to take
    pub actions: Vec<BackpressureAction>,
}

/// Backpressure actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackpressureAction {
    /// Pause stream
    PauseStream,

    /// Drop chunks
    DropChunks(u32),

    /// Reduce quality
    ReduceQuality(f32),

    /// Notify upstream
    NotifyUpstream,

    /// Custom action
    Custom(serde_json::Value),
}

/// Stream priority for resource allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamPriority {
    /// Low priority
    Low,

    /// Normal priority
    Normal,

    /// High priority
    High,

    /// Critical priority
    Critical,
}

impl Default for StreamPriority {
    fn default() -> Self {
        StreamPriority::Normal
    }
}

/// Configuration structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamManagerConfig {
    /// Maximum concurrent streams
    pub max_concurrent_streams: usize,

    /// Default stream timeout
    pub default_timeout: Duration,

    /// Cleanup interval for inactive streams
    pub cleanup_interval: Duration,

    /// Enable metrics collection
    pub enable_metrics: bool,

    /// Default buffer size
    pub default_buffer_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiplexerConfig {
    /// Maximum input streams
    pub max_input_streams: usize,

    /// Maximum output channels
    pub max_output_channels: usize,

    /// Processing buffer size
    pub processing_buffer_size: usize,
}

impl Default for MultiplexerConfig {
    fn default() -> Self {
        Self {
            max_input_streams: 1000,
            max_output_channels: 100,
            processing_buffer_size: 8192,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpressureControllerConfig {
    /// Monitoring interval
    pub monitoring_interval: Duration,

    /// Default high water mark
    pub default_high_water_mark: f32,

    /// Default low water mark
    pub default_low_water_mark: f32,
}

impl Default for BackpressureControllerConfig {
    fn default() -> Self {
        // Load unified config for environment-aware timeout values
        let config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .and_then(|loaded| loaded.try_into_config().ok());

        let monitoring_interval = if let Some(cfg) = config {
            cfg.timeouts
                .get_custom_timeout("stream_monitoring")
                .unwrap_or_else(|| Duration::from_secs(5))
        } else {
            Duration::from_secs(5)
        };

        Self {
            monitoring_interval,
            default_high_water_mark: 0.8,
            default_low_water_mark: 0.5,
        }
    }
}

/// Overall streaming metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMetrics {
    /// Total streams created
    pub total_streams_created: u64,

    /// Currently active streams
    pub active_streams: u64,

    /// Total chunks processed
    pub total_chunks_processed: u64,

    /// Total bytes processed
    pub total_bytes_processed: u64,

    /// Current throughput (chunks/second)
    pub current_throughput: f64,

    /// Current bandwidth (bytes/second)
    pub current_bandwidth: f64,

    /// Error rate
    pub error_rate: f64,

    /// Average stream duration
    pub avg_stream_duration: Duration,
}
