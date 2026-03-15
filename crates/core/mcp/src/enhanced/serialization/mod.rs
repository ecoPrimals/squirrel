// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Zero-Copy Serialization System for MCP
//!
//! This module provides high-performance serialization optimizations including:
//! - Buffer pooling to reduce allocations
//! - Zero-copy serialization using Bytes and streaming
//! - Fast-path serializers for common message types
//! - Message template systems for repeated structures
//! - Custom codecs optimized for specific use cases

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use bytes::{Bytes, BytesMut, BufMut};
use tokio::sync::{RwLock, Mutex};
use serde::{Serialize, Deserialize};
use serde_json;
use tracing::{info, debug, warn, instrument};

use crate::error::{Result, types::MCPError};
use crate::protocol::types::MCPMessage;
use crate::enhanced::ai_types::{UniversalAIRequest, UniversalAIResponse};

pub mod buffer_pool;
pub mod streaming;
pub mod codecs;
pub mod templates;

pub use buffer_pool::*;
pub use streaming::*;
pub use codecs::*;
pub use templates::*;

/// High-performance serialization manager with zero-copy optimizations
#[derive(Debug)]
pub struct ZeroCopySerializer {
    /// Buffer pool for reusing allocation buffers
    buffer_pool: Arc<BufferPool>,
    
    /// Message template cache for common structures
    template_cache: Arc<RwLock<MessageTemplateCache>>,
    
    /// Fast-path codecs for specific message types
    codecs: Arc<RwLock<HashMap<String, Box<dyn FastCodec + Send + Sync>>>>,
    
    /// Serialization metrics for performance monitoring
    metrics: Arc<Mutex<SerializationMetrics>>,
    
    /// Configuration for serialization behavior
    config: SerializationConfig,
}

/// Configuration for zero-copy serialization
#[derive(Debug, Clone)]
pub struct SerializationConfig {
    /// Enable buffer pooling
    pub enable_buffer_pooling: bool,
    
    /// Maximum buffer size in pool
    pub max_buffer_size: usize,
    
    /// Initial buffer pool size
    pub initial_pool_size: usize,
    
    /// Maximum pool size
    pub max_pool_size: usize,
    
    /// Enable streaming serialization
    pub enable_streaming: bool,
    
    /// Enable message templates
    pub enable_templates: bool,
    
    /// Template cache size
    pub template_cache_size: usize,
    
    /// Enable fast-path codecs
    pub enable_fast_codecs: bool,
    
    /// Minimum message size for compression
    pub compression_threshold: usize,
}

impl Default for SerializationConfig {
    fn default() -> Self {
        Self {
            enable_buffer_pooling: true,
            max_buffer_size: 1024 * 1024, // 1MB
            initial_pool_size: 10,
            max_pool_size: 100,
            enable_streaming: true,
            enable_templates: true,
            template_cache_size: 1000,
            enable_fast_codecs: true,
            compression_threshold: 1024, // 1KB
        }
    }
}

/// Serialization performance metrics
#[derive(Debug, Default, Clone)]
pub struct SerializationMetrics {
    /// Total serializations performed
    pub total_serializations: u64,
    
    /// Total deserializations performed
    pub total_deserializations: u64,
    
    /// Buffer pool hits
    pub buffer_pool_hits: u64,
    
    /// Buffer pool misses
    pub buffer_pool_misses: u64,
    
    /// Template cache hits
    pub template_cache_hits: u64,
    
    /// Template cache misses
    pub template_cache_misses: u64,
    
    /// Fast codec usage
    pub fast_codec_usage: u64,
    
    /// Fallback codec usage
    pub fallback_codec_usage: u64,
    
    /// Total bytes serialized
    pub bytes_serialized: u64,
    
    /// Total bytes deserialized
    pub bytes_deserialized: u64,
    
    /// Average serialization time in microseconds
    pub avg_serialization_time_us: f64,
    
    /// Average deserialization time in microseconds
    pub avg_deserialization_time_us: f64,
    
    /// Memory saved through optimizations (bytes)
    pub memory_saved_bytes: u64,
}

/// Zero-copy serialization result
#[derive(Debug)]
pub struct SerializationResult {
    /// Serialized data as zero-copy Bytes
    pub data: Bytes,
    
    /// Metadata about the serialization
    pub metadata: SerializationMetadata,
}

/// Metadata about serialization operation
#[derive(Debug, Clone)]
pub struct SerializationMetadata {
    /// Original size before optimization
    pub original_size: usize,
    
    /// Final size after optimization
    pub final_size: usize,
    
    /// Compression ratio (if compression was used)
    pub compression_ratio: Option<f64>,
    
    /// Serialization method used
    pub method: SerializationMethod,
    
    /// Time taken for serialization
    pub duration: Duration,
    
    /// Whether buffer pooling was used
    pub used_buffer_pool: bool,
    
    /// Whether template was used
    pub used_template: bool,
}

/// Serialization methods available
#[derive(Debug, Clone, PartialEq)]
pub enum SerializationMethod {
    /// Standard serde_json serialization
    Standard,
    
    /// Fast codec serialization
    FastCodec,
    
    /// Template-based serialization
    Template,
    
    /// Streaming serialization
    Streaming,
    
    /// Direct binary serialization
    Binary,
}

impl ZeroCopySerializer {
    /// Create a new zero-copy serializer
    pub fn new(config: SerializationConfig) -> Self {
        let buffer_pool = Arc::new(BufferPool::new(BufferPoolConfig {
            initial_size: config.initial_pool_size,
            max_size: config.max_pool_size,
            max_buffer_size: config.max_buffer_size,
        }));
        
        let template_cache = Arc::new(RwLock::new(MessageTemplateCache::new(config.template_cache_size)));
        let codecs = Arc::new(RwLock::new(HashMap::new()));
        let metrics = Arc::new(Mutex::new(SerializationMetrics::default()));
        
        Self {
            buffer_pool,
            template_cache,
            codecs,
            metrics,
            config,
        }
    }
    
    /// Register a fast codec for a specific message type
    pub async fn register_codec(&self, message_type: String, codec: Box<dyn FastCodec + Send + Sync>) {
        if self.config.enable_fast_codecs {
            let mut codecs = self.codecs.write().await;
            codecs.insert(message_type, codec);
        }
    }
    
    /// Serialize an MCPMessage with zero-copy optimizations
    #[instrument(skip(self, message))]
    pub async fn serialize_mcp_message(&self, message: &MCPMessage) -> Result<SerializationResult> {
        let start_time = Instant::now();
        let message_type = format!("{:?}", message.type_);
        
        // Try fast codec first
        if self.config.enable_fast_codecs {
            let codecs = self.codecs.read().await;
            if let Some(codec) = codecs.get(&message_type) {
                match codec.encode(message).await {
                    Ok(result) => {
                        self.update_metrics_success(start_time, result.data.len(), SerializationMethod::FastCodec, true, false).await;
                        return Ok(result);
                    }
                    Err(e) => {
                        debug!("Fast codec failed for {}: {}, falling back to standard", message_type, e);
                    }
                }
            }
        }
        
        // Try template-based serialization
        if self.config.enable_templates {
            let template_cache = self.template_cache.read().await;
            if let Some(template) = template_cache.get_template(&message_type) {
                match self.serialize_with_template(message, template).await {
                    Ok(result) => {
                        self.update_metrics_success(start_time, result.data.len(), SerializationMethod::Template, true, true).await;
                        return Ok(result);
                    }
                    Err(e) => {
                        debug!("Template serialization failed for {}: {}, falling back to standard", message_type, e);
                    }
                }
            }
        }
        
        // Use standard serialization with buffer pooling
        self.serialize_standard(message, start_time).await
    }
    
    /// Serialize a UniversalAIRequest with optimizations
    #[instrument(skip(self, request))]
    pub async fn serialize_ai_request(&self, request: &UniversalAIRequest) -> Result<SerializationResult> {
        let start_time = Instant::now();
        
        // Try fast codec for AI requests
        if self.config.enable_fast_codecs {
            let codecs = self.codecs.read().await;
            if let Some(codec) = codecs.get("UniversalAIRequest") {
                match codec.encode_generic(request).await {
                    Ok(result) => {
                        self.update_metrics_success(start_time, result.data.len(), SerializationMethod::FastCodec, true, false).await;
                        return Ok(result);
                    }
                    Err(e) => {
                        debug!("AI request fast codec failed: {}, falling back to standard", e);
                    }
                }
            }
        }
        
        // Use streaming serialization for large requests
        if self.config.enable_streaming && self.estimate_size(request) > self.config.compression_threshold {
            return self.serialize_streaming(request, start_time).await;
        }
        
        // Standard serialization with buffer pooling
        self.serialize_generic_standard(request, start_time).await
    }
    
    /// Serialize a UniversalAIResponse with optimizations
    #[instrument(skip(self, response))]
    pub async fn serialize_ai_response(&self, response: &UniversalAIResponse) -> Result<SerializationResult> {
        let start_time = Instant::now();
        
        // Try fast codec for AI responses
        if self.config.enable_fast_codecs {
            let codecs = self.codecs.read().await;
            if let Some(codec) = codecs.get("UniversalAIResponse") {
                match codec.encode_generic(response).await {
                    Ok(result) => {
                        self.update_metrics_success(start_time, result.data.len(), SerializationMethod::FastCodec, true, false).await;
                        return Ok(result);
                    }
                    Err(e) => {
                        debug!("AI response fast codec failed: {}, falling back to standard", e);
                    }
                }
            }
        }
        
        // Standard serialization
        self.serialize_generic_standard(response, start_time).await
    }
    
    /// Deserialize data into an MCPMessage with zero-copy optimizations
    #[instrument(skip(self, data))]
    pub async fn deserialize_mcp_message(&self, data: &Bytes) -> Result<MCPMessage> {
        let start_time = Instant::now();
        
        // Try fast codec first
        if self.config.enable_fast_codecs {
            if let Ok(message) = self.try_fast_decode_mcp(data).await {
                self.update_deserialize_metrics_success(start_time, data.len()).await;
                return Ok(message);
            }
        }
        
        // Standard deserialization
        let message = serde_json::from_slice(data).map_err(|e| {
            MCPError::Internal(format!("Failed to deserialize MCPMessage: {}", e))
        })?;
        
        self.update_deserialize_metrics_success(start_time, data.len()).await;
        Ok(message)
    }
    
    /// Deserialize data into a UniversalAIRequest with Arc<str> optimization
    pub async fn deserialize_ai_request(&self, data: &Bytes) -> Result<UniversalAIRequest> {
        let start_time = Instant::now();
        
        let request = serde_json::from_slice(data).map_err(|e| {
            MCPError::Internal(format!("Failed to deserialize UniversalAIRequest: {}", e))
        })?;
        
        self.update_deserialize_metrics_success(start_time, data.len()).await;
        Ok(request)
    }
    
    /// Deserialize data into a UniversalAIResponse with Arc<str> optimization
    pub async fn deserialize_ai_response(&self, data: &Bytes) -> Result<UniversalAIResponse> {
        let start_time = Instant::now();
        
        let response = serde_json::from_slice(data).map_err(|e| {
            MCPError::Internal(format!("Failed to deserialize UniversalAIResponse: {}", e))
        })?;
        
        self.update_deserialize_metrics_success(start_time, data.len()).await;
        Ok(response)
    }
    
    /// Standard serialization with buffer pooling
    async fn serialize_standard(&self, message: &MCPMessage, start_time: Instant) -> Result<SerializationResult> {
        let original_size = std::mem::size_of_val(message);
        
        if self.config.enable_buffer_pooling {
            // Get buffer from pool
            let mut buffer = self.buffer_pool.get_buffer().await;
            
            // Serialize directly into buffer
            serde_json::to_writer(&mut buffer, message).map_err(|e| {
                MCPError::Internal(format!("Failed to serialize MCPMessage: {}", e))
            })?;
            
            let data = buffer.freeze();
            let final_size = data.len();
            
            // Return buffer to pool
            self.buffer_pool.return_buffer(buffer.into()).await;
            
            let metadata = SerializationMetadata {
                original_size,
                final_size,
                compression_ratio: None,
                method: SerializationMethod::Standard,
                duration: start_time.elapsed(),
                used_buffer_pool: true,
                used_template: false,
            };
            
            self.update_metrics_success(start_time, final_size, SerializationMethod::Standard, true, false).await;
            
            Ok(SerializationResult { data, metadata })
        } else {
            // Standard serialization without pooling
            let json = serde_json::to_vec(message).map_err(|e| {
                MCPError::Internal(format!("Failed to serialize MCPMessage: {}", e))
            })?;
            
            let data = Bytes::from(json);
            let final_size = data.len();
            
            let metadata = SerializationMetadata {
                original_size,
                final_size,
                compression_ratio: None,
                method: SerializationMethod::Standard,
                duration: start_time.elapsed(),
                used_buffer_pool: false,
                used_template: false,
            };
            
            self.update_metrics_success(start_time, final_size, SerializationMethod::Standard, false, false).await;
            
            Ok(SerializationResult { data, metadata })
        }
    }
    
    /// Generic standard serialization for any serializable type
    async fn serialize_generic_standard<T: Serialize>(&self, value: &T, start_time: Instant) -> Result<SerializationResult> {
        let original_size = std::mem::size_of_val(value);
        
        if self.config.enable_buffer_pooling {
            let mut buffer = self.buffer_pool.get_buffer().await;
            
            serde_json::to_writer(&mut buffer, value).map_err(|e| {
                MCPError::Internal(format!("Failed to serialize value: {}", e))
            })?;
            
            let data = buffer.freeze();
            let final_size = data.len();
            
            self.buffer_pool.return_buffer(buffer.into()).await;
            
            let metadata = SerializationMetadata {
                original_size,
                final_size,
                compression_ratio: None,
                method: SerializationMethod::Standard,
                duration: start_time.elapsed(),
                used_buffer_pool: true,
                used_template: false,
            };
            
            self.update_metrics_success(start_time, final_size, SerializationMethod::Standard, true, false).await;
            
            Ok(SerializationResult { data, metadata })
        } else {
            let json = serde_json::to_vec(value).map_err(|e| {
                MCPError::Internal(format!("Failed to serialize value: {}", e))
            })?;
            
            let data = Bytes::from(json);
            let final_size = data.len();
            
            let metadata = SerializationMetadata {
                original_size,
                final_size,
                compression_ratio: None,
                method: SerializationMethod::Standard,
                duration: start_time.elapsed(),
                used_buffer_pool: false,
                used_template: false,
            };
            
            self.update_metrics_success(start_time, final_size, SerializationMethod::Standard, false, false).await;
            
            Ok(SerializationResult { data, metadata })
        }
    }
    
    /// Streaming serialization for large objects
    async fn serialize_streaming<T: Serialize>(&self, value: &T, start_time: Instant) -> Result<SerializationResult> {
        let original_size = std::mem::size_of_val(value);
        
        let mut buffer = BytesMut::with_capacity(8192);
        let mut serializer = serde_json::Serializer::new(&mut buffer);
        
        value.serialize(&mut serializer).map_err(|e| {
            MCPError::Internal(format!("Failed to stream serialize: {}", e))
        })?;
        
        let data = buffer.freeze();
        let final_size = data.len();
        
        let metadata = SerializationMetadata {
            original_size,
            final_size,
            compression_ratio: None,
            method: SerializationMethod::Streaming,
            duration: start_time.elapsed(),
            used_buffer_pool: false,
            used_template: false,
        };
        
        self.update_metrics_success(start_time, final_size, SerializationMethod::Streaming, false, false).await;
        
        Ok(SerializationResult { data, metadata })
    }
    
    /// Serialize using a message template
    async fn serialize_with_template(&self, message: &MCPMessage, template: &MessageTemplate) -> Result<SerializationResult> {
        // Template-based serialization implementation would go here
        // This is a placeholder for the actual template serialization logic
        Err(MCPError::Internal("Template serialization not yet implemented".to_string()))
    }
    
    /// Try fast decode for MCP messages
    async fn try_fast_decode_mcp(&self, data: &Bytes) -> Result<MCPMessage> {
        let codecs = self.codecs.read().await;
        if let Some(codec) = codecs.get("MCPMessage") {
            codec.decode(data).await
        } else {
            Err(MCPError::Internal("No fast codec available for MCPMessage".to_string()))
        }
    }
    
    /// Estimate size of a serializable object
    fn estimate_size<T>(&self, _value: &T) -> usize {
        // This is a placeholder - in practice, you'd implement size estimation
        // based on the structure of the object
        1024 // Default estimate
    }
    
    /// Update metrics after successful serialization
    async fn update_metrics_success(
        &self, 
        start_time: Instant, 
        final_size: usize, 
        method: SerializationMethod,
        used_buffer_pool: bool,
        used_template: bool
    ) {
        let mut metrics = self.metrics.lock().await;
        metrics.total_serializations += 1;
        metrics.bytes_serialized += final_size as u64;
        
        if used_buffer_pool {
            metrics.buffer_pool_hits += 1;
        } else {
            metrics.buffer_pool_misses += 1;
        }
        
        if used_template {
            metrics.template_cache_hits += 1;
        } else {
            metrics.template_cache_misses += 1;
        }
        
        match method {
            SerializationMethod::FastCodec => metrics.fast_codec_usage += 1,
            _ => metrics.fallback_codec_usage += 1,
        }
        
        // Update average serialization time (exponential moving average)
        let duration_us = start_time.elapsed().as_micros() as f64;
        if metrics.avg_serialization_time_us == 0.0 {
            metrics.avg_serialization_time_us = duration_us;
        } else {
            metrics.avg_serialization_time_us = (metrics.avg_serialization_time_us * 0.9) + (duration_us * 0.1);
        }
    }
    
    /// Update metrics after successful deserialization
    async fn update_deserialize_metrics_success(&self, start_time: Instant, size: usize) {
        let mut metrics = self.metrics.lock().await;
        metrics.total_deserializations += 1;
        metrics.bytes_deserialized += size as u64;
        
        // Update average deserialization time (exponential moving average)
        let duration_us = start_time.elapsed().as_micros() as f64;
        if metrics.avg_deserialization_time_us == 0.0 {
            metrics.avg_deserialization_time_us = duration_us;
        } else {
            metrics.avg_deserialization_time_us = (metrics.avg_deserialization_time_us * 0.9) + (duration_us * 0.1);
        }
    }
    
    /// Get current serialization metrics
    pub async fn get_metrics(&self) -> SerializationMetrics {
        self.metrics.lock().await.clone()
    }
    
    /// Generate performance report
    pub async fn generate_performance_report(&self) -> SerializationPerformanceReport {
        let metrics = self.get_metrics().await;
        
        SerializationPerformanceReport {
            metrics: metrics.clone(),
            buffer_pool_efficiency: if metrics.buffer_pool_hits + metrics.buffer_pool_misses > 0 {
                metrics.buffer_pool_hits as f64 / (metrics.buffer_pool_hits + metrics.buffer_pool_misses) as f64
            } else {
                0.0
            },
            template_cache_efficiency: if metrics.template_cache_hits + metrics.template_cache_misses > 0 {
                metrics.template_cache_hits as f64 / (metrics.template_cache_hits + metrics.template_cache_misses) as f64
            } else {
                0.0
            },
            fast_codec_usage_rate: if metrics.fast_codec_usage + metrics.fallback_codec_usage > 0 {
                metrics.fast_codec_usage as f64 / (metrics.fast_codec_usage + metrics.fallback_codec_usage) as f64
            } else {
                0.0
            },
            average_throughput_mbps: if metrics.avg_serialization_time_us > 0.0 {
                let avg_size = metrics.bytes_serialized as f64 / metrics.total_serializations as f64;
                (avg_size * 8.0) / (metrics.avg_serialization_time_us / 1_000_000.0) / 1_000_000.0 // Convert to Mbps
            } else {
                0.0
            },
            generated_at: Instant::now(),
        }
    }
}

/// Performance report for serialization operations
#[derive(Debug, Clone)]
pub struct SerializationPerformanceReport {
    /// Current metrics
    pub metrics: SerializationMetrics,
    
    /// Buffer pool hit rate (0.0 to 1.0)
    pub buffer_pool_efficiency: f64,
    
    /// Template cache hit rate (0.0 to 1.0)
    pub template_cache_efficiency: f64,
    
    /// Fast codec usage rate (0.0 to 1.0)
    pub fast_codec_usage_rate: f64,
    
    /// Average throughput in Mbps
    pub average_throughput_mbps: f64,
    
    /// Report generation time
    pub generated_at: Instant,
}

impl SerializationPerformanceReport {
    /// Print a formatted performance report
    pub fn print_report(&self) {
        info!("=== Zero-Copy Serialization Performance Report ===");
        info!("Total Serializations: {}", self.metrics.total_serializations);
        info!("Total Deserializations: {}", self.metrics.total_deserializations);
        info!("Bytes Serialized: {} MB", self.metrics.bytes_serialized / (1024 * 1024));
        info!("Bytes Deserialized: {} MB", self.metrics.bytes_deserialized / (1024 * 1024));
        info!("Avg Serialization Time: {:.2} μs", self.metrics.avg_serialization_time_us);
        info!("Avg Deserialization Time: {:.2} μs", self.metrics.avg_deserialization_time_us);
        info!("Buffer Pool Efficiency: {:.1}%", self.buffer_pool_efficiency * 100.0);
        info!("Template Cache Efficiency: {:.1}%", self.template_cache_efficiency * 100.0);
        info!("Fast Codec Usage Rate: {:.1}%", self.fast_codec_usage_rate * 100.0);
        info!("Average Throughput: {:.2} Mbps", self.average_throughput_mbps);
        info!("Memory Saved: {} KB", self.metrics.memory_saved_bytes / 1024);
        info!("================================================");
    }
}

/// Global zero-copy serializer instance
static GLOBAL_SERIALIZER: std::sync::OnceLock<Arc<ZeroCopySerializer>> = std::sync::OnceLock::new();

/// Get or initialize the global zero-copy serializer
pub fn get_global_serializer() -> &'static Arc<ZeroCopySerializer> {
    GLOBAL_SERIALIZER.get_or_init(|| {
        Arc::new(ZeroCopySerializer::new(SerializationConfig::default()))
    })
}

/// Initialize global serializer with custom config
pub fn init_global_serializer(config: SerializationConfig) -> Result<()> {
    let serializer = Arc::new(ZeroCopySerializer::new(config));
    GLOBAL_SERIALIZER.set(serializer).map_err(|_| {
        MCPError::Internal("Global serializer already initialized".to_string())
    })?;
    Ok(())
} 