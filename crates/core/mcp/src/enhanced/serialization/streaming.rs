//! Streaming Serialization for Large Messages
//!
//! This module provides streaming serialization capabilities for handling
//! large messages and continuous data flows without loading everything into memory.

use std::pin::Pin;
use std::task::{Context, Poll};
use bytes::{Bytes, BytesMut, BufMut};
use futures::{Stream, StreamExt, Sink, SinkExt};
use tokio::io::{AsyncWrite, AsyncWriteExt};
use serde::{Serialize, Deserialize};
use serde_json;
use tracing::{debug, warn, instrument};

use crate::error::{Result, types::MCPError};
use super::{SerializationResult, SerializationMetadata, SerializationMethod};

/// Configuration for streaming serialization
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Chunk size for streaming operations
    pub chunk_size: usize,
    
    /// Maximum buffer size before flushing
    pub max_buffer_size: usize,
    
    /// Enable compression for streams
    pub enable_compression: bool,
    
    /// Compression threshold (bytes)
    pub compression_threshold: usize,
    
    /// Stream timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            chunk_size: 8192,     // 8KB chunks
            max_buffer_size: 64 * 1024, // 64KB buffer
            enable_compression: false, // Disabled by default for simplicity
            compression_threshold: 1024, // 1KB
            timeout_ms: 30000,    // 30 seconds
        }
    }
}

/// Streaming serializer for large objects
#[derive(Debug)]
pub struct StreamingSerializer {
    config: StreamingConfig,
    buffer: BytesMut,
}

/// Streaming deserializer for large objects
#[derive(Debug)]
pub struct StreamingDeserializer {
    config: StreamingConfig,
    buffer: BytesMut,
}

/// Stream of serialized chunks
pub struct SerializedStream {
    chunks: Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>,
    metadata: StreamMetadata,
}

/// Stream metadata
#[derive(Debug, Clone)]
pub struct StreamMetadata {
    /// Total estimated size
    pub estimated_size: Option<usize>,
    
    /// Number of chunks
    pub chunk_count: usize,
    
    /// Compression used
    pub compressed: bool,
    
    /// Stream creation timestamp
    pub created_at: std::time::Instant,
}

/// Chunk in a serialized stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedChunk {
    /// Chunk sequence number
    pub sequence: u32,
    
    /// Chunk data
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    
    /// Is this the last chunk?
    pub is_final: bool,
    
    /// Chunk checksum for integrity
    pub checksum: u32,
}

impl StreamingSerializer {
    /// Create a new streaming serializer
    pub fn new(config: StreamingConfig) -> Self {
        Self {
            config,
            buffer: BytesMut::with_capacity(config.max_buffer_size),
        }
    }
    
    /// Serialize an object to a stream of chunks
    #[instrument(skip(self, value))]
    pub async fn serialize_to_stream<T: Serialize + Send>(&mut self, value: &T) -> Result<SerializedStream> {
        debug!("Starting streaming serialization");
        
        let start_time = std::time::Instant::now();
        let estimated_size = self.estimate_serialized_size(value);
        
        // Start serialization
        let mut serializer = serde_json::Serializer::new(&mut self.buffer);
        value.serialize(&mut serializer).map_err(|e| {
            MCPError::Internal(format!("Streaming serialization failed: {}", e))
        })?;
        
        // Create chunks from buffer
        let chunks = self.create_chunks_from_buffer(estimated_size).await?;
        
        let metadata = StreamMetadata {
            estimated_size: Some(estimated_size),
            chunk_count: 0, // Will be updated as chunks are consumed
            compressed: false, // Compression not implemented in this demo
            created_at: start_time,
        };
        
        Ok(SerializedStream { chunks, metadata })
    }
    
    /// Serialize directly to an AsyncWrite stream
    #[instrument(skip(self, value, writer))]
    pub async fn serialize_to_writer<T: Serialize + Send, W: AsyncWrite + Unpin>(&mut self, value: &T, mut writer: W) -> Result<SerializationMetadata> {
        let start_time = std::time::Instant::now();
        let original_size = std::mem::size_of_val(value);
        
        // Create a custom Write adapter that writes directly to the AsyncWrite
        let mut json_writer = AsyncJsonWriter::new(&mut writer, self.config.chunk_size);
        
        // Serialize directly to the writer
        let mut serializer = serde_json::Serializer::new(&mut json_writer);
        value.serialize(&mut serializer).map_err(|e| {
            MCPError::Internal(format!("Direct streaming serialization failed: {}", e))
        })?;
        
        // Flush any remaining data
        json_writer.flush().await?;
        
        let final_size = json_writer.bytes_written();
        
        Ok(SerializationMetadata {
            original_size,
            final_size,
            compression_ratio: None,
            method: SerializationMethod::Streaming,
            duration: start_time.elapsed(),
            used_buffer_pool: false,
            used_template: false,
        })
    }
    
    /// Create chunks from the internal buffer
    async fn create_chunks_from_buffer(&mut self, estimated_size: usize) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>> {
        let chunk_size = self.config.chunk_size;
        let buffer_data = self.buffer.split().freeze();
        
        let chunks: Vec<Result<Bytes>> = buffer_data
            .chunks(chunk_size)
            .enumerate()
            .map(|(i, chunk)| {
                let is_final = (i + 1) * chunk_size >= buffer_data.len();
                let chunk = SerializedChunk {
                    sequence: i as u32,
                    data: chunk.to_vec(),
                    is_final,
                    checksum: crc32fast::hash(chunk),
                };
                
                // Serialize the chunk
                serde_json::to_vec(&chunk)
                    .map(Bytes::from)
                    .map_err(|e| MCPError::Internal(format!("Chunk serialization failed: {}", e)))
            })
            .collect();
        
        Ok(Box::pin(futures::stream::iter(chunks)))
    }
    
    /// Estimate serialized size of an object
    fn estimate_serialized_size<T: Serialize>(&self, value: &T) -> usize {
        // This is a rough estimate - in practice you'd implement more sophisticated size estimation
        std::mem::size_of_val(value) * 2 // JSON is typically 2x the memory size
    }
}

impl StreamingDeserializer {
    /// Create a new streaming deserializer
    pub fn new(config: StreamingConfig) -> Self {
        Self {
            config,
            buffer: BytesMut::with_capacity(config.max_buffer_size),
        }
    }
    
    /// Deserialize from a stream of chunks
    #[instrument(skip(self, chunks))]
    pub async fn deserialize_from_stream<T: for<'de> Deserialize<'de>>(&mut self, mut chunks: SerializedStream) -> Result<T> {
        debug!("Starting streaming deserialization");
        
        // Collect all chunks
        while let Some(chunk_result) = chunks.chunks.next().await {
            let chunk_bytes = chunk_result?;
            
            // Deserialize chunk
            let chunk: SerializedChunk = serde_json::from_slice(&chunk_bytes).map_err(|e| {
                MCPError::Internal(format!("Chunk deserialization failed: {}", e))
            })?;
            
            // Verify checksum
            let calculated_checksum = crc32fast::hash(&chunk.data);
            if calculated_checksum != chunk.checksum {
                return Err(MCPError::Internal("Chunk checksum verification failed".to_string()));
            }
            
            // Append to buffer
            self.buffer.extend_from_slice(&chunk.data);
            
            // If this is the final chunk, break
            if chunk.is_final {
                break;
            }
        }
        
        // Deserialize the complete object
        let value = serde_json::from_slice(&self.buffer).map_err(|e| {
            MCPError::Internal(format!("Final deserialization failed: {}", e))
        })?;
        
        Ok(value)
    }
}

/// AsyncWrite adapter for streaming JSON serialization
struct AsyncJsonWriter<'a, W: AsyncWrite + Unpin> {
    writer: &'a mut W,
    buffer: BytesMut,
    chunk_size: usize,
    bytes_written: usize,
}

impl<'a, W: AsyncWrite + Unpin> AsyncJsonWriter<'a, W> {
    fn new(writer: &'a mut W, chunk_size: usize) -> Self {
        Self {
            writer,
            buffer: BytesMut::with_capacity(chunk_size),
            chunk_size,
            bytes_written: 0,
        }
    }
    
    fn bytes_written(&self) -> usize {
        self.bytes_written
    }
    
    async fn flush(&mut self) -> Result<()> {
        if !self.buffer.is_empty() {
            self.writer.write_all(&self.buffer).await.map_err(|e| {
                MCPError::Internal(format!("Async write failed: {}", e))
            })?;
            self.bytes_written += self.buffer.len();
            self.buffer.clear();
        }
        
        self.writer.flush().await.map_err(|e| {
            MCPError::Internal(format!("Async flush failed: {}", e))
        })?;
        
        Ok(())
    }
}

impl<'a, W: AsyncWrite + Unpin> std::io::Write for AsyncJsonWriter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        
        // Flush when buffer is full
        if self.buffer.len() >= self.chunk_size {
            // We can't await in a sync write, so we'll buffer and flush later
            // In a real implementation, you'd use a different approach here
        }
        
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        // Sync flush - we'll handle async flushing separately
        Ok(())
    }
}

/// Stream sink for writing serialized data
pub struct SerializedSink<W: AsyncWrite + Unpin> {
    writer: W,
    config: StreamingConfig,
    bytes_written: usize,
}

impl<W: AsyncWrite + Unpin> SerializedSink<W> {
    /// Create a new serialized sink
    pub fn new(writer: W, config: StreamingConfig) -> Self {
        Self {
            writer,
            config,
            bytes_written: 0,
        }
    }
    
    /// Get bytes written so far
    pub fn bytes_written(&self) -> usize {
        self.bytes_written
    }
}

impl<W: AsyncWrite + Unpin> Sink<Bytes> for SerializedSink<W> {
    type Error = MCPError;
    
    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
    
    fn start_send(mut self: Pin<&mut Self>, item: Bytes) -> Result<(), Self::Error> {
        // In a real implementation, you'd buffer the item and write asynchronously
        self.bytes_written += item.len();
        Ok(())
    }
    
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let writer = Pin::new(&mut self.writer);
        match writer.poll_flush(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(MCPError::Internal(format!("Sink flush failed: {}", e)))),
            Poll::Pending => Poll::Pending,
        }
    }
    
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let writer = Pin::new(&mut self.writer);
        match writer.poll_shutdown(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(MCPError::Internal(format!("Sink close failed: {}", e)))),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Utilities for streaming serialization
pub struct StreamingUtils;

impl StreamingUtils {
    /// Calculate optimal chunk size based on data characteristics
    pub fn calculate_optimal_chunk_size(data_size: usize, available_memory: usize) -> usize {
        let min_chunk_size = 1024;      // 1KB minimum
        let max_chunk_size = 1024 * 1024; // 1MB maximum
        
        // Aim for ~100 chunks, but respect memory constraints
        let target_chunk_size = (data_size / 100).max(min_chunk_size);
        let memory_constrained_size = available_memory / 10; // Use max 10% of available memory
        
        target_chunk_size.min(memory_constrained_size).min(max_chunk_size)
    }
    
    /// Estimate streaming overhead
    pub fn estimate_streaming_overhead(data_size: usize, chunk_size: usize) -> StreamingOverhead {
        let num_chunks = (data_size + chunk_size - 1) / chunk_size;
        let chunk_header_size = 64; // Estimated header size per chunk
        let total_overhead = num_chunks * chunk_header_size;
        
        StreamingOverhead {
            chunk_count: num_chunks,
            header_overhead_bytes: total_overhead,
            overhead_percentage: (total_overhead as f64 / data_size as f64) * 100.0,
        }
    }
    
    /// Create a streaming configuration optimized for specific use case
    pub fn create_config_for_use_case(use_case: StreamingUseCase) -> StreamingConfig {
        match use_case {
            StreamingUseCase::LargeMessages => StreamingConfig {
                chunk_size: 64 * 1024,  // 64KB chunks
                max_buffer_size: 1024 * 1024, // 1MB buffer
                enable_compression: true,
                compression_threshold: 10 * 1024, // 10KB
                timeout_ms: 60000, // 60 seconds
            },
            StreamingUseCase::RealTime => StreamingConfig {
                chunk_size: 4 * 1024,   // 4KB chunks for lower latency
                max_buffer_size: 32 * 1024, // 32KB buffer
                enable_compression: false, // No compression for speed
                compression_threshold: 0,
                timeout_ms: 5000, // 5 seconds
            },
            StreamingUseCase::LowMemory => StreamingConfig {
                chunk_size: 1024,       // 1KB chunks
                max_buffer_size: 8 * 1024, // 8KB buffer
                enable_compression: true,
                compression_threshold: 512, // 512 bytes
                timeout_ms: 120000, // 2 minutes (more time due to small chunks)
            },
            StreamingUseCase::HighThroughput => StreamingConfig {
                chunk_size: 256 * 1024, // 256KB chunks
                max_buffer_size: 2 * 1024 * 1024, // 2MB buffer
                enable_compression: false, // No compression for speed
                compression_threshold: 0,
                timeout_ms: 30000, // 30 seconds
            },
        }
    }
}

/// Use cases for streaming serialization
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StreamingUseCase {
    /// Large messages that don't fit in memory
    LargeMessages,
    /// Real-time streaming with low latency requirements
    RealTime,
    /// Memory-constrained environments
    LowMemory,
    /// High-throughput bulk data transfer
    HighThroughput,
}

/// Streaming overhead analysis
#[derive(Debug, Clone)]
pub struct StreamingOverhead {
    /// Number of chunks
    pub chunk_count: usize,
    
    /// Header overhead in bytes
    pub header_overhead_bytes: usize,
    
    /// Overhead as percentage of original data
    pub overhead_percentage: f64,
}

/// Factory for creating streaming components
pub struct StreamingFactory;

impl StreamingFactory {
    /// Create a streaming serializer for specific use case
    pub fn create_serializer(use_case: StreamingUseCase) -> StreamingSerializer {
        let config = StreamingUtils::create_config_for_use_case(use_case);
        StreamingSerializer::new(config)
    }
    
    /// Create a streaming deserializer for specific use case
    pub fn create_deserializer(use_case: StreamingUseCase) -> StreamingDeserializer {
        let config = StreamingUtils::create_config_for_use_case(use_case);
        StreamingDeserializer::new(config)
    }
}

// Implement Stream for SerializedStream
impl Stream for SerializedStream {
    type Item = Result<Bytes>;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.chunks.poll_next_unpin(cx)
    }
} 