// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Buffer Pool for Zero-Copy Serialization
//!
//! This module provides a high-performance buffer pool that reuses BytesMut buffers
//! to eliminate memory allocations during frequent serialization operations.

use std::collections::VecDeque;
use std::sync::Arc;
use bytes::{BytesMut, BufMut};
use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::error::{Result, types::MCPError};

/// Configuration for buffer pool
#[derive(Debug, Clone)]
pub struct BufferPoolConfig {
    /// Initial number of buffers in the pool
    pub initial_size: usize,
    
    /// Maximum number of buffers to keep in the pool
    pub max_size: usize,
    
    /// Maximum size of individual buffers
    pub max_buffer_size: usize,
}

impl Default for BufferPoolConfig {
    fn default() -> Self {
        Self {
            initial_size: 10,
            max_size: 100,
            max_buffer_size: 1024 * 1024, // 1MB
        }
    }
}

/// High-performance buffer pool for reusing BytesMut buffers
#[derive(Debug)]
pub struct BufferPool {
    /// Pool of available buffers
    buffers: Arc<Mutex<VecDeque<BytesMut>>>,
    
    /// Pool configuration
    config: BufferPoolConfig,
    
    /// Pool statistics
    stats: Arc<Mutex<BufferPoolStats>>,
}

/// Buffer pool statistics
#[derive(Debug, Default, Clone)]
pub struct BufferPoolStats {
    /// Total buffers created
    pub total_created: u64,
    
    /// Total buffers reused
    pub total_reused: u64,
    
    /// Current pool size
    pub current_size: usize,
    
    /// Peak pool size
    pub peak_size: usize,
    
    /// Total gets from pool
    pub total_gets: u64,
    
    /// Total returns to pool
    pub total_returns: u64,
    
    /// Buffer cache hits
    pub cache_hits: u64,
    
    /// Buffer cache misses
    pub cache_misses: u64,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new(config: BufferPoolConfig) -> Self {
        let buffers = Arc::new(Mutex::new(VecDeque::with_capacity(config.initial_size)));
        let stats = Arc::new(Mutex::new(BufferPoolStats::default()));
        
        let pool = Self {
            buffers,
            config,
            stats,
        };
        
        // Pre-allocate initial buffers
        tokio::spawn({
            let pool = pool.clone();
            async move {
                pool.preallocate_buffers().await;
            }
        });
        
        pool
    }
    
    /// Get a buffer from the pool
    pub async fn get_buffer(&self) -> BytesMut {
        let mut stats = self.stats.lock().await;
        stats.total_gets += 1;
        
        let mut buffers = self.buffers.lock().await;
        
        if let Some(mut buffer) = buffers.pop_front() {
            // Reuse existing buffer
            buffer.clear();
            stats.total_reused += 1;
            stats.cache_hits += 1;
            stats.current_size = buffers.len();
            
            debug!("Reused buffer from pool, pool size: {}", buffers.len());
            buffer
        } else {
            // Create new buffer
            stats.total_created += 1;
            stats.cache_misses += 1;
            
            let buffer = BytesMut::with_capacity(8192); // Start with 8KB
            debug!("Created new buffer, total created: {}", stats.total_created);
            buffer
        }
    }
    
    /// Return a buffer to the pool
    pub async fn return_buffer(&self, buffer: BytesMut) {
        let mut stats = self.stats.lock().await;
        stats.total_returns += 1;
        
        // Check buffer size limits
        if buffer.capacity() > self.config.max_buffer_size {
            debug!("Buffer too large for pool ({} bytes), discarding", buffer.capacity());
            return;
        }
        
        let mut buffers = self.buffers.lock().await;
        
        // Check pool size limits
        if buffers.len() >= self.config.max_size {
            debug!("Buffer pool full ({} buffers), discarding buffer", buffers.len());
            return;
        }
        
        buffers.push_back(buffer);
        stats.current_size = buffers.len();
        
        if stats.current_size > stats.peak_size {
            stats.peak_size = stats.current_size;
        }
        
        debug!("Returned buffer to pool, pool size: {}", buffers.len());
    }
    
    /// Get buffer pool statistics
    pub async fn get_stats(&self) -> BufferPoolStats {
        self.stats.lock().await.clone()
    }
    
    /// Pre-allocate initial buffers
    async fn preallocate_buffers(&self) {
        let mut buffers = self.buffers.lock().await;
        let mut stats = self.stats.lock().await;
        
        for _ in 0..self.config.initial_size {
            let buffer = BytesMut::with_capacity(8192);
            buffers.push_back(buffer);
            stats.total_created += 1;
        }
        
        stats.current_size = buffers.len();
        stats.peak_size = stats.current_size;
        
        debug!("Pre-allocated {} buffers", self.config.initial_size);
    }
    
    /// Shrink pool to optimal size
    pub async fn shrink_to_fit(&self) {
        let mut buffers = self.buffers.lock().await;
        let mut stats = self.stats.lock().await;
        
        let target_size = (self.config.initial_size * 2).min(buffers.len());
        
        while buffers.len() > target_size {
            buffers.pop_back();
        }
        
        stats.current_size = buffers.len();
        debug!("Shrunk buffer pool to {} buffers", buffers.len());
    }
    
    /// Get pool efficiency metrics
    pub async fn get_efficiency_metrics(&self) -> BufferPoolEfficiency {
        let stats = self.get_stats().await;
        
        let hit_rate = if stats.total_gets > 0 {
            stats.cache_hits as f64 / stats.total_gets as f64
        } else {
            0.0
        };
        
        let reuse_rate = if stats.total_created > 0 {
            stats.total_reused as f64 / stats.total_created as f64
        } else {
            0.0
        };
        
        let utilization = if self.config.max_size > 0 {
            stats.current_size as f64 / self.config.max_size as f64
        } else {
            0.0
        };
        
        BufferPoolEfficiency {
            hit_rate,
            reuse_rate,
            utilization,
            memory_saved_bytes: stats.total_reused * 8192, // Approximate savings
        }
    }
}

/// Buffer pool efficiency metrics
#[derive(Debug, Clone)]
pub struct BufferPoolEfficiency {
    /// Cache hit rate (0.0 to 1.0)
    pub hit_rate: f64,
    
    /// Buffer reuse rate (0.0 to 1.0) 
    pub reuse_rate: f64,
    
    /// Pool utilization (0.0 to 1.0)
    pub utilization: f64,
    
    /// Estimated memory saved in bytes
    pub memory_saved_bytes: u64,
}

impl Clone for BufferPool {
    fn clone(&self) -> Self {
        Self {
            buffers: Arc::clone(&self.buffers),
            config: self.config.clone(),
            stats: Arc::clone(&self.stats),
        }
    }
}

/// Pooled buffer wrapper that automatically returns buffer to pool when dropped
pub struct PooledBuffer {
    buffer: Option<BytesMut>,
    pool: Arc<BufferPool>,
}

impl PooledBuffer {
    /// Create a new pooled buffer
    pub fn new(buffer: BytesMut, pool: Arc<BufferPool>) -> Self {
        Self {
            buffer: Some(buffer),
            pool,
        }
    }
    
    /// Get mutable reference to the buffer
    pub fn get_mut(&mut self) -> &mut BytesMut {
        self.buffer.as_mut().expect("Buffer already taken")
    }
    
    /// Get reference to the buffer
    pub fn get(&self) -> &BytesMut {
        self.buffer.as_ref().expect("Buffer already taken")
    }
    
    /// Take the buffer out of the wrapper
    pub fn take(mut self) -> BytesMut {
        self.buffer.take().expect("Buffer already taken")
    }
}

impl Drop for PooledBuffer {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            let pool = Arc::clone(&self.pool);
            tokio::spawn(async move {
                pool.return_buffer(buffer).await;
            });
        }
    }
}

impl AsRef<BytesMut> for PooledBuffer {
    fn as_ref(&self) -> &BytesMut {
        self.get()
    }
}

impl AsMut<BytesMut> for PooledBuffer {
    fn as_mut(&mut self) -> &mut BytesMut {
        self.get_mut()
    }
}

/// Buffer pool factory for creating optimized pools
pub struct BufferPoolFactory;

impl BufferPoolFactory {
    /// Create a high-performance buffer pool
    pub fn create_high_performance() -> BufferPool {
        BufferPool::new(BufferPoolConfig {
            initial_size: 20,
            max_size: 200,
            max_buffer_size: 2 * 1024 * 1024, // 2MB
        })
    }
    
    /// Create a memory-efficient buffer pool
    pub fn create_memory_efficient() -> BufferPool {
        BufferPool::new(BufferPoolConfig {
            initial_size: 5,
            max_size: 50,
            max_buffer_size: 512 * 1024, // 512KB
        })
    }
    
    /// Create a buffer pool optimized for small messages
    pub fn create_small_message() -> BufferPool {
        BufferPool::new(BufferPoolConfig {
            initial_size: 50,
            max_size: 500,
            max_buffer_size: 64 * 1024, // 64KB
        })
    }
    
    /// Create a buffer pool optimized for large messages
    pub fn create_large_message() -> BufferPool {
        BufferPool::new(BufferPoolConfig {
            initial_size: 10,
            max_size: 50,
            max_buffer_size: 10 * 1024 * 1024, // 10MB
        })
    }
} 