//! Bidirectional Streaming for Enhanced MCP
//!
//! This module provides bidirectional streaming capabilities for real-time
//! AI interactions, multi-agent coordination, and continuous data flow between
//! client and server.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Mutex, mpsc, broadcast};
use tokio::time::{interval, Instant};
use tokio_stream::{Stream, StreamExt};
use futures::future::{AbortHandle, Abortable};
use tracing::{info, error, warn, debug, instrument};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::error::{Result, types::MCPError};
use crate::protocol::types::{MCPMessage, MessageType};
use super::{MCPEvent, EventType, StreamChunk, StreamType};

/// Bidirectional streaming manager for MCP
#[derive(Debug)]
pub struct BidirectionalStreamManager {
    /// Configuration
    config: Arc<StreamingConfig>,
    
    /// Active streams
    active_streams: Arc<RwLock<HashMap<String, Arc<MCPStream>>>>,
    
    /// Stream registry
    stream_registry: Arc<RwLock<HashMap<StreamType, Vec<String>>>>,
    
    /// Event broadcaster
    event_broadcaster: Arc<broadcast::Sender<StreamEvent>>,
    
    /// Metrics collector
    metrics: Arc<Mutex<StreamingMetrics>>,
    
    /// Cleanup task handle
    cleanup_task: Arc<Mutex<Option<AbortHandle>>>,
}

/// Bidirectional MCP stream
#[derive(Debug)]
pub struct MCPStream {
    /// Stream ID
    pub stream_id: String,
    
    /// Stream type
    pub stream_type: StreamType,
    
    /// Stream direction
    pub direction: StreamDirection,
    
    /// Stream state
    pub state: Arc<RwLock<StreamState>>,
    
    /// Inbound message channel
    pub inbound_tx: mpsc::Sender<MCPMessage>,
    pub inbound_rx: Arc<Mutex<Option<mpsc::Receiver<MCPMessage>>>>,
    
    /// Outbound message channel
    pub outbound_tx: mpsc::Sender<MCPMessage>,
    pub outbound_rx: Arc<Mutex<Option<mpsc::Receiver<MCPMessage>>>>,
    
    /// Stream metadata
    pub metadata: Arc<RwLock<StreamMetadata>>,
    
    /// Stream configuration
    pub config: StreamConfig,
    
    /// Statistics
    pub stats: Arc<Mutex<StreamStatistics>>,
}

/// Streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Maximum concurrent streams
    pub max_concurrent_streams: usize,
    
    /// Stream timeout
    pub stream_timeout: Duration,
    
    /// Buffer size for channels
    pub buffer_size: usize,
    
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    
    /// Cleanup interval
    pub cleanup_interval: Duration,
    
    /// Enable compression
    pub enable_compression: bool,
    
    /// Enable encryption
    pub enable_encryption: bool,
}

/// Stream configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    /// Stream name
    pub name: String,
    
    /// Stream description
    pub description: Option<String>,
    
    /// Stream tags
    pub tags: Vec<String>,
    
    /// Stream priority
    pub priority: StreamPriority,
    
    /// Stream options
    pub options: StreamOptions,
}

/// Stream options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamOptions {
    /// Enable batching
    pub enable_batching: bool,
    
    /// Batch size
    pub batch_size: usize,
    
    /// Batch timeout
    pub batch_timeout: Duration,
    
    /// Enable acknowledgments
    pub enable_acks: bool,
    
    /// Retry attempts
    pub retry_attempts: u32,
    
    /// Retry delay
    pub retry_delay: Duration,
}

/// Stream direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamDirection {
    /// Client to server
    Inbound,
    /// Server to client
    Outbound,
    /// Both directions
    Bidirectional,
}

/// Stream state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamState {
    /// Stream is initializing
    Initializing,
    /// Stream is active
    Active,
    /// Stream is paused
    Paused,
    /// Stream is draining
    Draining,
    /// Stream is closed
    Closed,
    /// Stream has error
    Error(String),
}

/// Stream priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Stream metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMetadata {
    /// Session ID
    pub session_id: String,
    
    /// Client ID
    pub client_id: String,
    
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Last activity
    pub last_activity: chrono::DateTime<chrono::Utc>,
    
    /// Stream labels
    pub labels: HashMap<String, String>,
    
    /// Stream annotations
    pub annotations: HashMap<String, serde_json::Value>,
}

/// Stream statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStatistics {
    /// Total messages sent
    pub messages_sent: u64,
    
    /// Total messages received
    pub messages_received: u64,
    
    /// Total bytes sent
    pub bytes_sent: u64,
    
    /// Total bytes received
    pub bytes_received: u64,
    
    /// Error count
    pub error_count: u64,
    
    /// Average latency (ms)
    pub average_latency_ms: f64,
    
    /// Throughput (messages/second)
    pub throughput_mps: f64,
}

/// Stream event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    /// Event type
    pub event_type: StreamEventType,
    
    /// Stream ID
    pub stream_id: String,
    
    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Event data
    pub data: serde_json::Value,
}

/// Stream event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamEventType {
    StreamCreated,
    StreamActivated,
    StreamPaused,
    StreamResumed,
    StreamClosed,
    StreamError,
    MessageSent,
    MessageReceived,
    LatencyUpdated,
    ThroughputUpdated,
}

/// Streaming metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingMetrics {
    /// Total streams created
    pub total_streams: u64,
    
    /// Active streams
    pub active_streams: u64,
    
    /// Total messages processed
    pub total_messages: u64,
    
    /// Total bytes processed
    pub total_bytes: u64,
    
    /// Average stream lifetime (seconds)
    pub average_stream_lifetime: f64,
    
    /// System throughput (messages/second)
    pub system_throughput: f64,
    
    /// System latency (ms)
    pub system_latency: f64,
}

impl BidirectionalStreamManager {
    /// Create a new bidirectional stream manager
    pub async fn new(config: StreamingConfig) -> Result<Self> {
        info!("Initializing Bidirectional Stream Manager");
        
        let (event_tx, _) = broadcast::channel(1000);
        
        let manager = Self {
            config: Arc::new(config),
            active_streams: Arc::new(RwLock::new(HashMap::new())),
            stream_registry: Arc::new(RwLock::new(HashMap::new())),
            event_broadcaster: Arc::new(event_tx),
            metrics: Arc::new(Mutex::new(StreamingMetrics::default())),
            cleanup_task: Arc::new(Mutex::new(None)),
        };
        
        info!("Bidirectional Stream Manager initialized");
        Ok(manager)
    }
    
    /// Start the stream manager
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        info!("Starting Bidirectional Stream Manager");
        
        // Start cleanup task
        self.start_cleanup_task().await?;
        
        // Start metrics collection
        self.start_metrics_collection().await?;
        
        info!("Bidirectional Stream Manager started");
        Ok(())
    }
    
    /// Stop the stream manager
    #[instrument(skip(self))]
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Bidirectional Stream Manager");
        
        // Stop cleanup task
        if let Some(handle) = self.cleanup_task.lock().await.take() {
            handle.abort();
        }
        
        // Close all active streams
        self.close_all_streams().await?;
        
        info!("Bidirectional Stream Manager stopped");
        Ok(())
    }
    
    /// Create a new bidirectional stream
    #[instrument(skip(self))]
    pub async fn create_stream(
        &self,
        session_id: String,
        client_id: String,
        stream_type: StreamType,
        direction: StreamDirection,
        config: StreamConfig,
    ) -> Result<String> {
        debug!("Creating stream for session: {}", session_id);
        
        // Check stream limits
        self.check_stream_limits().await?;
        
        // Create stream
        let stream_id = Uuid::new_v4().to_string();
        let (inbound_tx, inbound_rx) = mpsc::channel(self.config.buffer_size);
        let (outbound_tx, outbound_rx) = mpsc::channel(self.config.buffer_size);
        
        let stream = MCPStream {
            stream_id: stream_id.clone(),
            stream_type,
            direction,
            state: Arc::new(RwLock::new(StreamState::Initializing)),
            inbound_tx,
            inbound_rx: Arc::new(Mutex::new(Some(inbound_rx))),
            outbound_tx,
            outbound_rx: Arc::new(Mutex::new(Some(outbound_rx))),
            metadata: Arc::new(RwLock::new(StreamMetadata {
                session_id,
                client_id,
                created_at: chrono::Utc::now(),
                last_activity: chrono::Utc::now(),
                labels: HashMap::new(),
                annotations: HashMap::new(),
            })),
            config,
            stats: Arc::new(Mutex::new(StreamStatistics::default())),
        };
        
        // Store stream
        let stream_arc = Arc::new(stream);
        {
            let mut streams = self.active_streams.write().await;
            streams.insert(stream_id.clone(), stream_arc.clone());
        }
        
        // Register stream by type
        {
            let mut registry = self.stream_registry.write().await;
            registry.entry(stream_type).or_insert_with(Vec::new).push(stream_id.clone());
        }
        
        // Activate stream
        self.activate_stream(&stream_id).await?;
        
        // Update metrics
        self.update_stream_metrics().await;
        
        // Emit event
        self.emit_stream_event(StreamEventType::StreamCreated, &stream_id, serde_json::json!({
            "stream_type": stream_type,
            "direction": direction
        })).await;
        
        info!("Created stream: {}", stream_id);
        Ok(stream_id)
    }
    
    /// Activate a stream
    #[instrument(skip(self))]
    pub async fn activate_stream(&self, stream_id: &str) -> Result<()> {
        debug!("Activating stream: {}", stream_id);
        
        let streams = self.active_streams.read().await;
        if let Some(stream) = streams.get(stream_id) {
            // Set state to active
            *stream.state.write().await = StreamState::Active;
            
            // Update activity
            stream.metadata.write().await.last_activity = chrono::Utc::now();
            
            // Emit event
            self.emit_stream_event(StreamEventType::StreamActivated, stream_id, serde_json::json!({})).await;
            
            info!("Stream activated: {}", stream_id);
            Ok(())
        } else {
            Err(MCPError::NotFound(format!("Stream not found: {}", stream_id)))
        }
    }
    
    /// Send message to stream
    #[instrument(skip(self, message))]
    pub async fn send_message(&self, stream_id: &str, message: MCPMessage) -> Result<()> {
        debug!("Sending message to stream: {}", stream_id);
        
        let streams = self.active_streams.read().await;
        if let Some(stream) = streams.get(stream_id) {
            // Check stream state
            let state = stream.state.read().await;
            if *state != StreamState::Active {
                return Err(MCPError::Protocol(format!("Stream not active: {}", stream_id)));
            }
            drop(state);
            
            // Send message
            stream.outbound_tx.send(message).await
                .map_err(|e| MCPError::Transport(format!("Failed to send message: {}", e).into()))?;
            
            // Update statistics
            {
                let mut stats = stream.stats.lock().await;
                stats.messages_sent += 1;
                stats.bytes_sent += 1024; // Approximate size
            }
            
            // Update activity
            stream.metadata.write().await.last_activity = chrono::Utc::now();
            
            // Emit event
            self.emit_stream_event(StreamEventType::MessageSent, stream_id, serde_json::json!({})).await;
            
            debug!("Message sent to stream: {}", stream_id);
            Ok(())
        } else {
            Err(MCPError::NotFound(format!("Stream not found: {}", stream_id)))
        }
    }
    
    /// Receive message from stream
    #[instrument(skip(self))]
    pub async fn receive_message(&self, stream_id: &str) -> Result<Option<MCPMessage>> {
        debug!("Receiving message from stream: {}", stream_id);
        
        let streams = self.active_streams.read().await;
        if let Some(stream) = streams.get(stream_id) {
            // Check stream state
            let state = stream.state.read().await;
            if *state != StreamState::Active {
                return Err(MCPError::Protocol(format!("Stream not active: {}", stream_id)));
            }
            drop(state);
            
            // Try to receive message (non-blocking)
            let mut rx_guard = stream.inbound_rx.lock().await;
            if let Some(rx) = rx_guard.as_mut() {
                match rx.try_recv() {
                    Ok(message) => {
                        // Update statistics
                        {
                            let mut stats = stream.stats.lock().await;
                            stats.messages_received += 1;
                            stats.bytes_received += 1024; // Approximate size
                        }
                        
                        // Update activity
                        stream.metadata.write().await.last_activity = chrono::Utc::now();
                        
                        // Emit event
                        self.emit_stream_event(StreamEventType::MessageReceived, stream_id, serde_json::json!({})).await;
                        
                        debug!("Message received from stream: {}", stream_id);
                        Ok(Some(message))
                    }
                    Err(mpsc::error::TryRecvError::Empty) => Ok(None),
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        warn!("Stream channel disconnected: {}", stream_id);
                        Ok(None)
                    }
                }
            } else {
                Err(MCPError::Transport("Stream receiver not available".into()))
            }
        } else {
            Err(MCPError::NotFound(format!("Stream not found: {}", stream_id)))
        }
    }
    
    /// Close a stream
    #[instrument(skip(self))]
    pub async fn close_stream(&self, stream_id: &str) -> Result<()> {
        debug!("Closing stream: {}", stream_id);
        
        let mut streams = self.active_streams.write().await;
        if let Some(stream) = streams.remove(stream_id) {
            // Set state to closed
            *stream.state.write().await = StreamState::Closed;
            
            // Remove from registry
            {
                let mut registry = self.stream_registry.write().await;
                if let Some(stream_ids) = registry.get_mut(&stream.stream_type) {
                    stream_ids.retain(|id| id != stream_id);
                }
            }
            
            // Emit event
            self.emit_stream_event(StreamEventType::StreamClosed, stream_id, serde_json::json!({})).await;
            
            info!("Stream closed: {}", stream_id);
            Ok(())
        } else {
            Err(MCPError::NotFound(format!("Stream not found: {}", stream_id)))
        }
    }
    
    /// Get stream statistics
    pub async fn get_stream_stats(&self, stream_id: &str) -> Result<StreamStatistics> {
        let streams = self.active_streams.read().await;
        if let Some(stream) = streams.get(stream_id) {
            Ok(stream.stats.lock().await.clone())
        } else {
            Err(MCPError::NotFound(format!("Stream not found: {}", stream_id)))
        }
    }
    
    /// Get streaming metrics
    pub async fn get_metrics(&self) -> StreamingMetrics {
        self.metrics.lock().await.clone()
    }
    
    /// Subscribe to stream events
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<StreamEvent> {
        self.event_broadcaster.subscribe()
    }
    
    // Private helper methods
    async fn check_stream_limits(&self) -> Result<()> {
        let streams = self.active_streams.read().await;
        if streams.len() >= self.config.max_concurrent_streams {
            return Err(MCPError::Protocol("Maximum concurrent streams exceeded".to_string()));
        }
        Ok(())
    }
    
    async fn start_cleanup_task(&self) -> Result<()> {
        let streams = self.active_streams.clone();
        let cleanup_interval = self.config.cleanup_interval;
        let stream_timeout = self.config.stream_timeout;
        
        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        
        let cleanup_task = tokio::spawn(Abortable::new(async move {
            let mut interval = interval(cleanup_interval);
            
            loop {
                interval.tick().await;
                
                // Clean up inactive streams
                let mut to_remove = Vec::new();
                let now = chrono::Utc::now();
                
                {
                    let streams_guard = streams.read().await;
                    for (stream_id, stream) in streams_guard.iter() {
                        let metadata = stream.metadata.read().await;
                        let inactive_duration = now - metadata.last_activity;
                        
                        if inactive_duration > chrono::Duration::from_std(stream_timeout).unwrap() {
                            to_remove.push(stream_id.clone());
                        }
                    }
                }
                
                // Remove inactive streams
                if !to_remove.is_empty() {
                    let mut streams_guard = streams.write().await;
                    for stream_id in to_remove {
                        if let Some(stream) = streams_guard.remove(&stream_id) {
                            *stream.state.write().await = StreamState::Closed;
                            warn!("Stream timeout, closed: {}", stream_id);
                        }
                    }
                }
            }
        }, abort_registration));
        
        *self.cleanup_task.lock().await = Some(abort_handle);
        
        tokio::spawn(async move {
            if let Err(e) = cleanup_task.await {
                if !e.is_cancelled() {
                    error!("Cleanup task failed: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    async fn start_metrics_collection(&self) -> Result<()> {
        let metrics = self.metrics.clone();
        let streams = self.active_streams.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                let mut metrics_guard = metrics.lock().await;
                let streams_guard = streams.read().await;
                
                metrics_guard.active_streams = streams_guard.len() as u64;
                
                // Calculate aggregate statistics
                let mut total_messages = 0;
                let mut total_bytes = 0;
                let mut total_latency = 0.0;
                let mut stream_count = 0;
                
                for stream in streams_guard.values() {
                    let stats = stream.stats.lock().await;
                    total_messages += stats.messages_sent + stats.messages_received;
                    total_bytes += stats.bytes_sent + stats.bytes_received;
                    total_latency += stats.average_latency_ms;
                    stream_count += 1;
                }
                
                metrics_guard.total_messages = total_messages;
                metrics_guard.total_bytes = total_bytes;
                if stream_count > 0 {
                    metrics_guard.system_latency = total_latency / stream_count as f64;
                }
            }
        });
        
        Ok(())
    }
    
    async fn close_all_streams(&self) -> Result<()> {
        let mut streams = self.active_streams.write().await;
        for (stream_id, stream) in streams.iter() {
            *stream.state.write().await = StreamState::Closed;
            info!("Stream closed during shutdown: {}", stream_id);
        }
        streams.clear();
        Ok(())
    }
    
    async fn update_stream_metrics(&self) {
        let mut metrics = self.metrics.lock().await;
        metrics.total_streams += 1;
    }
    
    async fn emit_stream_event(&self, event_type: StreamEventType, stream_id: &str, data: serde_json::Value) {
        let event = StreamEvent {
            event_type,
            stream_id: stream_id.to_string(),
            timestamp: chrono::Utc::now(),
            data,
        };
        
        if let Err(e) = self.event_broadcaster.send(event) {
            warn!("Failed to emit stream event: {}", e);
        }
    }
}

impl Default for StreamingConfig {
    fn default() -> Self {
        // Load unified config for environment-aware timeout values
        let config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .and_then(|loaded| loaded.try_into_config().ok());
        
        let (stream_timeout, heartbeat_interval, cleanup_interval) = if let Some(cfg) = config {
            let timeout = cfg.timeouts.get_custom_timeout("bidir_stream_timeout")
                .unwrap_or_else(|| Duration::from_secs(300));
            let heartbeat = cfg.timeouts.get_custom_timeout("bidir_heartbeat")
                .unwrap_or_else(|| Duration::from_secs(30));
            let cleanup = cfg.timeouts.get_custom_timeout("bidir_cleanup")
                .unwrap_or_else(|| Duration::from_secs(60));
            (timeout, heartbeat, cleanup)
        } else {
            (
                Duration::from_secs(300),  // 5 minutes
                Duration::from_secs(30),   // 30 seconds
                Duration::from_secs(60),   // 1 minute
            )
        };
        
        Self {
            max_concurrent_streams: 1000,
            stream_timeout,
            buffer_size: 1000,
            heartbeat_interval,
            cleanup_interval,
            enable_compression: false,
            enable_encryption: false,
        }
    }
}

impl Default for StreamStatistics {
    fn default() -> Self {
        Self {
            messages_sent: 0,
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            error_count: 0,
            average_latency_ms: 0.0,
            throughput_mps: 0.0,
        }
    }
}

impl Default for StreamingMetrics {
    fn default() -> Self {
        Self {
            total_streams: 0,
            active_streams: 0,
            total_messages: 0,
            total_bytes: 0,
            average_stream_lifetime: 0.0,
            system_throughput: 0.0,
            system_latency: 0.0,
        }
    }
}

impl Default for StreamOptions {
    fn default() -> Self {
        // Load unified config for environment-aware timeout values
        let config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .and_then(|loaded| loaded.try_into_config().ok());
        
        let (batch_timeout, retry_delay) = if let Some(cfg) = config {
            let batch = cfg.timeouts.get_custom_timeout("bidir_batch_timeout")
                .unwrap_or_else(|| Duration::from_millis(100));
            let retry = cfg.timeouts.get_custom_timeout("bidir_retry_delay")
                .unwrap_or_else(|| Duration::from_millis(1000));
            (batch, retry)
        } else {
            (Duration::from_millis(100), Duration::from_millis(1000))
        };
        
        Self {
            enable_batching: false,
            batch_size: 100,
            batch_timeout,
            enable_acks: false,
            retry_attempts: 3,
            retry_delay,
        }
    }
}

/// Bidirectional streaming client interface
pub trait BidirectionalStreamingClient: Send + Sync {
    /// Create a new stream
    async fn create_stream(&self, config: StreamConfig) -> Result<String>;
    
    /// Send message to stream
    async fn send_message(&self, stream_id: &str, message: MCPMessage) -> Result<()>;
    
    /// Receive message from stream
    async fn receive_message(&self, stream_id: &str) -> Result<Option<MCPMessage>>;
    
    /// Close stream
    async fn close_stream(&self, stream_id: &str) -> Result<()>;
}

/// Bidirectional streaming server interface
pub trait BidirectionalStreamingServer: Send + Sync {
    /// Handle incoming stream creation
    async fn handle_stream_creation(&self, session_id: &str, config: StreamConfig) -> Result<String>;
    
    /// Handle incoming message
    async fn handle_message(&self, stream_id: &str, message: MCPMessage) -> Result<()>;
    
    /// Handle stream closure
    async fn handle_stream_closure(&self, stream_id: &str) -> Result<()>;
} 