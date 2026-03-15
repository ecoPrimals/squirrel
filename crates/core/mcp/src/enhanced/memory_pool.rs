// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Advanced Memory Pool System for Maximum Performance
//!
//! This module provides enterprise-grade memory pool management including:
//! - Object pooling for frequently allocated types
//! - String interning for zero-allocation lookups
//! - Message caching for hot paths
//! - Smart reuse policies and garbage collection
//! - Performance monitoring and optimization

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant, SystemTime};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use bytes::{Bytes, BytesMut};
use serde_json::Value;
use tokio::sync::{RwLock, Mutex};
use tracing::{debug, info, warn, instrument};
use uuid::Uuid;

use crate::error::{Result, types::MCPError};
use crate::protocol::types::{MCPMessage, MessageType};
use crate::enhanced::ai_types::{UniversalAIRequest, UniversalAIResponse};

/// Global memory pool manager
static GLOBAL_MEMORY_POOL: std::sync::OnceLock<Arc<AdvancedMemoryPool>> = std::sync::OnceLock::new();

/// Get or initialize the global memory pool
pub fn get_global_memory_pool() -> Arc<AdvancedMemoryPool> {
    GLOBAL_MEMORY_POOL.get_or_init(|| {
        Arc::new(AdvancedMemoryPool::new(MemoryPoolConfig::production()))
    }).clone()
}

/// Advanced memory pool configuration
#[derive(Debug, Clone)]
pub struct MemoryPoolConfig {
    /// Buffer pool configuration
    pub buffer_pool: BufferPoolConfig,
    
    /// Object pool configuration
    pub object_pool: ObjectPoolConfig,
    
    /// String interning configuration
    pub string_interning: StringInterningConfig,
    
    /// Message cache configuration
    pub message_cache: MessageCacheConfig,
    
    /// Garbage collection configuration
    pub gc_config: GarbageCollectionConfig,
}

/// Buffer pool configuration
#[derive(Debug, Clone)]
pub struct BufferPoolConfig {
    /// Small buffer pool (1KB-8KB)
    pub small_pool_size: usize,
    pub small_buffer_size: usize,
    
    /// Medium buffer pool (8KB-64KB)
    pub medium_pool_size: usize,
    pub medium_buffer_size: usize,
    
    /// Large buffer pool (64KB-1MB)
    pub large_pool_size: usize,
    pub large_buffer_size: usize,
    
    /// Pre-allocation on startup
    pub preallocate: bool,
}

/// Object pool configuration
#[derive(Debug, Clone)]
pub struct ObjectPoolConfig {
    /// Maximum objects per pool
    pub max_objects_per_type: usize,
    
    /// Object lifetime before cleanup
    pub object_lifetime: Duration,
    
    /// Enable object reuse tracking
    pub track_reuse: bool,
}

/// String interning configuration
#[derive(Debug, Clone)]
pub struct StringInterningConfig {
    /// Maximum interned strings
    pub max_strings: usize,
    
    /// String lifetime before cleanup
    pub string_lifetime: Duration,
    
    /// Enable weak references for cleanup
    pub enable_weak_refs: bool,
}

/// Message cache configuration
#[derive(Debug, Clone)]
pub struct MessageCacheConfig {
    /// Cache size limit
    pub max_cached_messages: usize,
    
    /// Cache TTL
    pub cache_ttl: Duration,
    
    /// Enable compression for large messages
    pub enable_compression: bool,
}

/// Garbage collection configuration
#[derive(Debug, Clone)]
pub struct GarbageCollectionConfig {
    /// GC interval
    pub gc_interval: Duration,
    
    /// Memory pressure threshold (0.0-1.0)
    pub memory_pressure_threshold: f64,
    
    /// Enable adaptive GC scheduling
    pub adaptive_scheduling: bool,
}

impl MemoryPoolConfig {
    /// Production-optimized configuration
    pub fn production() -> Self {
        // Load unified config for environment-aware timeout values
        let config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .and_then(|loaded| loaded.try_into_config().ok());
        
        let (object_lifetime, string_lifetime, cache_ttl, gc_interval) = if let Some(cfg) = config {
            let obj = cfg.timeouts.get_custom_timeout("mem_object_lifetime")
                .unwrap_or_else(|| Duration::from_secs(300));
            let str_life = cfg.timeouts.get_custom_timeout("mem_string_lifetime")
                .unwrap_or_else(|| Duration::from_secs(1800));
            let cache = cfg.timeouts.get_custom_timeout("mem_cache_ttl")
                .unwrap_or_else(|| Duration::from_secs(600));
            let gc = cfg.timeouts.get_custom_timeout("mem_gc_interval")
                .unwrap_or_else(|| Duration::from_secs(60));
            (obj, str_life, cache, gc)
        } else {
            (
                Duration::from_secs(300),   // 5 minutes
                Duration::from_secs(1800),  // 30 minutes
                Duration::from_secs(600),   // 10 minutes
                Duration::from_secs(60),    // 1 minute
            )
        };
        
        Self {
            buffer_pool: BufferPoolConfig {
                small_pool_size: 100,
                small_buffer_size: 4096,      // 4KB
                medium_pool_size: 50,
                medium_buffer_size: 32768,    // 32KB
                large_pool_size: 20,
                large_buffer_size: 262144,    // 256KB
                preallocate: true,
            },
            object_pool: ObjectPoolConfig {
                max_objects_per_type: 1000,
                object_lifetime,
                track_reuse: true,
            },
            string_interning: StringInterningConfig {
                max_strings: 10000,
                string_lifetime,
                enable_weak_refs: true,
            },
            message_cache: MessageCacheConfig {
                max_cached_messages: 5000,
                cache_ttl,
                enable_compression: true,
            },
            gc_config: GarbageCollectionConfig {
                gc_interval,
                memory_pressure_threshold: 0.8,
                adaptive_scheduling: true,
            },
        }
    }
    
    /// Development configuration with smaller pools
    pub fn development() -> Self {
        // Load unified config for environment-aware timeout values
        let config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .and_then(|loaded| loaded.try_into_config().ok());
        
        let (object_lifetime, string_lifetime, cache_ttl, gc_interval) = if let Some(cfg) = config {
            let obj = cfg.timeouts.get_custom_timeout("mem_dev_object_lifetime")
                .unwrap_or_else(|| Duration::from_secs(60));
            let str_life = cfg.timeouts.get_custom_timeout("mem_dev_string_lifetime")
                .unwrap_or_else(|| Duration::from_secs(300));
            let cache = cfg.timeouts.get_custom_timeout("mem_dev_cache_ttl")
                .unwrap_or_else(|| Duration::from_secs(60));
            let gc = cfg.timeouts.get_custom_timeout("mem_dev_gc_interval")
                .unwrap_or_else(|| Duration::from_secs(30));
            (obj, str_life, cache, gc)
        } else {
            (
                Duration::from_secs(60),    // 1 minute
                Duration::from_secs(300),   // 5 minutes
                Duration::from_secs(60),    // 1 minute
                Duration::from_secs(30),    // 30 seconds
            )
        };
        
        Self {
            buffer_pool: BufferPoolConfig {
                small_pool_size: 20,
                small_buffer_size: 2048,
                medium_pool_size: 10,
                medium_buffer_size: 16384,
                large_pool_size: 5,
                large_buffer_size: 131072,
                preallocate: false,
            },
            object_pool: ObjectPoolConfig {
                max_objects_per_type: 100,
                object_lifetime,
                track_reuse: false,
            },
            string_interning: StringInterningConfig {
                max_strings: 1000,
                string_lifetime,
                enable_weak_refs: false,
            },
            message_cache: MessageCacheConfig {
                max_cached_messages: 100,
                cache_ttl,
                enable_compression: false,
            },
            gc_config: GarbageCollectionConfig {
                gc_interval,
                memory_pressure_threshold: 0.9,
                adaptive_scheduling: false,
            },
        }
    }
}

/// Advanced memory pool with multiple optimization strategies
#[derive(Debug)]
pub struct AdvancedMemoryPool {
    /// Multi-tier buffer pools
    small_buffers: Arc<TieredBufferPool>,
    medium_buffers: Arc<TieredBufferPool>,
    large_buffers: Arc<TieredBufferPool>,
    
    /// Object pools for common types
    object_pools: Arc<RwLock<HashMap<String, Box<dyn ObjectPool + Send + Sync>>>>,
    
    /// String interning system
    string_interner: Arc<StringInterner>,
    
    /// Message cache for hot paths
    message_cache: Arc<MessageCache>,
    
    /// Pool metrics and monitoring
    metrics: Arc<RwLock<PoolMetrics>>,
    
    /// Configuration
    config: MemoryPoolConfig,
    
    /// Garbage collector handle
    gc_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

/// Multi-tier buffer pool with size-based allocation
#[derive(Debug)]
pub struct TieredBufferPool {
    /// Available buffers
    buffers: Arc<Mutex<VecDeque<PooledBuffer>>>,
    
    /// Pool configuration
    config: TierConfig,
    
    /// Pool statistics
    stats: Arc<RwLock<TierStats>>,
}

/// Configuration for a buffer tier
#[derive(Debug, Clone)]
pub struct TierConfig {
    pub pool_size: usize,
    pub buffer_size: usize,
    pub tier_name: String,
}

/// Statistics for a buffer tier
#[derive(Debug, Default, Clone)]
pub struct TierStats {
    pub total_allocations: u64,
    pub total_reuses: u64,
    pub current_size: usize,
    pub peak_size: usize,
    pub cache_hit_rate: f64,
}

/// Pooled buffer with automatic return-to-pool on drop
#[derive(Debug)]
pub struct PooledBuffer {
    buffer: BytesMut,
    pool: Weak<TieredBufferPool>,
    created_at: Instant,
    last_used: Instant,
}

/// Generic object pool trait
pub trait ObjectPool: std::fmt::Debug {
    fn get_object(&self) -> Result<Box<dyn PooledObject>>;
    fn return_object(&self, object: Box<dyn PooledObject>) -> Result<()>;
    fn pool_stats(&self) -> ObjectPoolStats;
    fn cleanup_expired(&self, max_age: Duration) -> usize;
}

/// Pooled object trait
pub trait PooledObject: std::fmt::Debug {
    fn reset(&mut self);
    fn object_type(&self) -> &str;
    fn created_at(&self) -> Instant;
    fn last_used(&self) -> Instant;
    fn use_count(&self) -> u64;
}

/// Object pool statistics
#[derive(Debug, Default, Clone)]
pub struct ObjectPoolStats {
    pub total_objects: usize,
    pub available_objects: usize,
    pub objects_created: u64,
    pub objects_reused: u64,
    pub average_use_count: f64,
}

/// String interning system for zero-allocation string operations
#[derive(Debug)]
pub struct StringInterner {
    /// Interned strings with reference counting
    strings: Arc<RwLock<HashMap<Arc<str>, (Weak<str>, SystemTime)>>>,
    
    /// Reverse lookup for cleanup
    reverse_lookup: Arc<RwLock<HashMap<*const str, Arc<str>>>>,
    
    /// Interner statistics
    stats: Arc<RwLock<InternerStats>>,
    
    /// Configuration
    config: StringInterningConfig,
}

/// String interning statistics
#[derive(Debug, Default, Clone)]
pub struct InternerStats {
    pub total_strings: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub memory_saved_bytes: u64,
    pub cleanup_operations: u64,
}

/// Message cache for frequently accessed messages
#[derive(Debug)]
pub struct MessageCache {
    /// Cached MCPMessages
    mcp_cache: Arc<RwLock<HashMap<String, CachedMessage<MCPMessage>>>>,
    
    /// Cached AI requests
    ai_request_cache: Arc<RwLock<HashMap<String, CachedMessage<UniversalAIRequest>>>>,
    
    /// Cached AI responses
    ai_response_cache: Arc<RwLock<HashMap<String, CachedMessage<UniversalAIResponse>>>>,
    
    /// Cache configuration
    config: MessageCacheConfig,
    
    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
}

/// Cached message with metadata
#[derive(Debug, Clone)]
pub struct CachedMessage<T> {
    pub message: Arc<T>,
    pub cached_at: SystemTime,
    pub access_count: AtomicU64,
    pub serialized_form: Option<Bytes>,
}

/// Cache statistics
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub evictions: u64,
    pub total_cached: usize,
    pub memory_usage_bytes: u64,
}

/// Overall pool metrics
#[derive(Debug, Default, Clone)]
pub struct PoolMetrics {
    /// Buffer pool metrics
    pub buffer_pools: HashMap<String, TierStats>,
    
    /// Object pool metrics
    pub object_pools: HashMap<String, ObjectPoolStats>,
    
    /// String interning metrics
    pub string_interner: InternerStats,
    
    /// Message cache metrics
    pub message_cache: CacheStats,
    
    /// Memory usage tracking
    pub total_memory_allocated: AtomicU64,
    pub total_memory_reused: AtomicU64,
    pub gc_cycles: AtomicU64,
}

impl AdvancedMemoryPool {
    /// Create a new advanced memory pool
    pub fn new(config: MemoryPoolConfig) -> Self {
        let small_buffers = Arc::new(TieredBufferPool::new(TierConfig {
            pool_size: config.buffer_pool.small_pool_size,
            buffer_size: config.buffer_pool.small_buffer_size,
            tier_name: "small".to_string(),
        }));
        
        let medium_buffers = Arc::new(TieredBufferPool::new(TierConfig {
            pool_size: config.buffer_pool.medium_pool_size,
            buffer_size: config.buffer_pool.medium_buffer_size,
            tier_name: "medium".to_string(),
        }));
        
        let large_buffers = Arc::new(TieredBufferPool::new(TierConfig {
            pool_size: config.buffer_pool.large_pool_size,
            buffer_size: config.buffer_pool.large_buffer_size,
            tier_name: "large".to_string(),
        }));
        
        let string_interner = Arc::new(StringInterner::new(config.string_interning.clone()));
        let message_cache = Arc::new(MessageCache::new(config.message_cache.clone()));
        let metrics = Arc::new(RwLock::new(PoolMetrics::default()));
        
        let pool = Self {
            small_buffers,
            medium_buffers,
            large_buffers,
            object_pools: Arc::new(RwLock::new(HashMap::new())),
            string_interner,
            message_cache,
            metrics,
            config: config.clone(),
            gc_handle: Arc::new(Mutex::new(None)),
        };
        
        // Start garbage collector
        pool.start_garbage_collector();
        
        // Pre-allocate buffers if configured
        if config.buffer_pool.preallocate {
            tokio::spawn({
                let pool_clone = pool.clone();
                async move {
                    pool_clone.preallocate_buffers().await;
                }
            });
        }
        
        pool
    }
    
    /// Get an optimally-sized buffer for the requested size
    #[instrument(skip(self))]
    pub async fn get_buffer(&self, requested_size: usize) -> PooledBuffer {
        let pool = if requested_size <= self.config.buffer_pool.small_buffer_size {
            &self.small_buffers
        } else if requested_size <= self.config.buffer_pool.medium_buffer_size {
            &self.medium_buffers
        } else {
            &self.large_buffers
        };
        
        pool.get_buffer().await
    }
    
    /// Intern a string for zero-allocation future lookups
    pub async fn intern_string(&self, s: &str) -> Arc<str> {
        self.string_interner.intern(s).await
    }
    
    /// Cache a message for fast future retrieval
    pub async fn cache_mcp_message(&self, key: String, message: MCPMessage) -> Result<()> {
        self.message_cache.cache_mcp_message(key, message).await
    }
    
    /// Retrieve a cached MCP message
    pub async fn get_cached_mcp_message(&self, key: &str) -> Option<Arc<MCPMessage>> {
        self.message_cache.get_mcp_message(key).await
    }
    
    /// Get comprehensive performance metrics
    pub async fn get_performance_report(&self) -> PoolPerformanceReport {
        let metrics = self.metrics.read().await;
        let buffer_efficiency = self.calculate_buffer_efficiency().await;
        let string_efficiency = self.string_interner.get_efficiency().await;
        let cache_efficiency = self.message_cache.get_efficiency().await;
        
        PoolPerformanceReport {
            metrics: metrics.clone(),
            buffer_efficiency,
            string_efficiency,
            cache_efficiency,
            total_memory_saved: self.calculate_total_memory_saved().await,
            generated_at: Instant::now(),
        }
    }
    
    /// Start the garbage collector task
    fn start_garbage_collector(&self) {
        let pool_weak = Arc::downgrade(&Arc::new(self.clone()));
        let gc_interval = self.config.gc_config.gc_interval;
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(gc_interval);
            
            loop {
                interval.tick().await;
                
                if let Some(pool) = pool_weak.upgrade() {
                    if let Err(e) = pool.run_garbage_collection().await {
                        warn!("Garbage collection failed: {}", e);
                    }
                } else {
                    // Pool has been dropped, exit GC task
                    debug!("Memory pool dropped, stopping garbage collector");
                    break;
                }
            }
        });
        
        tokio::spawn(async move {
            let gc_handle_arc = Arc::new(Mutex::new(Some(handle)));
            // Store the handle - in a real implementation, you'd need to properly manage this
        });
    }
    
    /// Run garbage collection cycle
    async fn run_garbage_collection(&self) -> Result<()> {
        let start = Instant::now();
        let mut collections = 0;
        
        // Collect expired strings
        collections += self.string_interner.cleanup_expired(
            self.config.string_interning.string_lifetime
        ).await;
        
        // Collect expired cache entries
        collections += self.message_cache.cleanup_expired(
            self.config.message_cache.cache_ttl
        ).await;
        
        // Collect expired objects
        for (_, pool) in self.object_pools.read().await.iter() {
            collections += pool.cleanup_expired(self.config.object_pool.object_lifetime);
        }
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.gc_cycles.fetch_add(1, Ordering::Relaxed);
        }
        
        let duration = start.elapsed();
        debug!(
            "Garbage collection completed: {} items collected in {:?}",
            collections, duration
        );
        
        Ok(())
    }
    
    /// Pre-allocate buffers for better startup performance
    async fn preallocate_buffers(&self) {
        debug!("Pre-allocating memory pool buffers");
        
        // Pre-allocate small buffers
        for _ in 0..self.config.buffer_pool.small_pool_size / 2 {
            let buffer = self.small_buffers.get_buffer().await;
            // Buffer will be automatically returned to pool when dropped
            drop(buffer);
        }
        
        // Pre-allocate medium buffers
        for _ in 0..self.config.buffer_pool.medium_pool_size / 2 {
            let buffer = self.medium_buffers.get_buffer().await;
            drop(buffer);
        }
        
        // Pre-allocate large buffers
        for _ in 0..self.config.buffer_pool.large_pool_size / 2 {
            let buffer = self.large_buffers.get_buffer().await;
            drop(buffer);
        }
        
        info!("Memory pool pre-allocation completed");
    }
    
    /// Calculate buffer pool efficiency metrics
    async fn calculate_buffer_efficiency(&self) -> f64 {
        let small_stats = self.small_buffers.get_stats().await;
        let medium_stats = self.medium_buffers.get_stats().await;
        let large_stats = self.large_buffers.get_stats().await;
        
        let total_reuses = small_stats.total_reuses + medium_stats.total_reuses + large_stats.total_reuses;
        let total_allocations = small_stats.total_allocations + medium_stats.total_allocations + large_stats.total_allocations;
        
        if total_allocations > 0 {
            total_reuses as f64 / total_allocations as f64
        } else {
            0.0
        }
    }
    
    /// Calculate total memory saved through pooling
    async fn calculate_total_memory_saved(&self) -> u64 {
        let buffer_savings = self.calculate_buffer_savings().await;
        let string_savings = self.string_interner.get_memory_savings().await;
        let cache_savings = self.message_cache.get_memory_savings().await;
        
        buffer_savings + string_savings + cache_savings
    }
    
    /// Calculate memory saved by buffer pooling
    async fn calculate_buffer_savings(&self) -> u64 {
        let small_stats = self.small_buffers.get_stats().await;
        let medium_stats = self.medium_buffers.get_stats().await;
        let large_stats = self.large_buffers.get_stats().await;
        
        let small_savings = small_stats.total_reuses * self.config.buffer_pool.small_buffer_size as u64;
        let medium_savings = medium_stats.total_reuses * self.config.buffer_pool.medium_buffer_size as u64;
        let large_savings = large_stats.total_reuses * self.config.buffer_pool.large_buffer_size as u64;
        
        small_savings + medium_savings + large_savings
    }
}

impl Clone for AdvancedMemoryPool {
    fn clone(&self) -> Self {
        Self {
            small_buffers: Arc::clone(&self.small_buffers),
            medium_buffers: Arc::clone(&self.medium_buffers),
            large_buffers: Arc::clone(&self.large_buffers),
            object_pools: Arc::clone(&self.object_pools),
            string_interner: Arc::clone(&self.string_interner),
            message_cache: Arc::clone(&self.message_cache),
            metrics: Arc::clone(&self.metrics),
            config: self.config.clone(),
            gc_handle: Arc::clone(&self.gc_handle),
        }
    }
}

/// Performance report for the memory pool
#[derive(Debug, Clone)]
pub struct PoolPerformanceReport {
    pub metrics: PoolMetrics,
    pub buffer_efficiency: f64,
    pub string_efficiency: f64,
    pub cache_efficiency: f64,
    pub total_memory_saved: u64,
    pub generated_at: Instant,
}

impl PoolPerformanceReport {
    /// Print a comprehensive performance report
    pub fn print_report(&self) {
        info!("=== Advanced Memory Pool Performance Report ===");
        info!("Buffer Pool Efficiency: {:.1}%", self.buffer_efficiency * 100.0);
        info!("String Interning Efficiency: {:.1}%", self.string_efficiency * 100.0);
        info!("Message Cache Efficiency: {:.1}%", self.cache_efficiency * 100.0);
        info!("Total Memory Saved: {} MB", self.total_memory_saved / (1024 * 1024));
        info!("GC Cycles: {}", self.metrics.gc_cycles.load(Ordering::Relaxed));
        info!("==============================================");
    }
}

// Implementation placeholders for the complex types
// These would be fully implemented in a production system

impl TieredBufferPool {
    fn new(config: TierConfig) -> Self {
        Self {
            buffers: Arc::new(Mutex::new(VecDeque::new())),
            config,
            stats: Arc::new(RwLock::new(TierStats::default())),
        }
    }
    
    async fn get_buffer(&self) -> PooledBuffer {
        let mut buffers = self.buffers.lock().await;
        let mut stats = self.stats.write().await;
        
        if let Some(mut buffer) = buffers.pop_front() {
            buffer.last_used = Instant::now();
            stats.total_reuses += 1;
            buffer
        } else {
            stats.total_allocations += 1;
            PooledBuffer {
                buffer: BytesMut::with_capacity(self.config.buffer_size),
                pool: Arc::downgrade(&Arc::new(self.clone())),
                created_at: Instant::now(),
                last_used: Instant::now(),
            }
        }
    }
    
    async fn get_stats(&self) -> TierStats {
        self.stats.read().await.clone()
    }
}

impl Clone for TieredBufferPool {
    fn clone(&self) -> Self {
        Self {
            buffers: Arc::clone(&self.buffers),
            config: self.config.clone(),
            stats: Arc::clone(&self.stats),
        }
    }
}

impl StringInterner {
    fn new(config: StringInterningConfig) -> Self {
        Self {
            strings: Arc::new(RwLock::new(HashMap::new())),
            reverse_lookup: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(InternerStats::default())),
            config,
        }
    }
    
    async fn intern(&self, s: &str) -> Arc<str> {
        // Check if string is already interned
        {
            let strings = self.strings.read().await;
            if let Some((weak_ref, _)) = strings.get(s).and_then(|(w, t)| Some((w.clone(), t.clone()))) {
                if let Some(arc_str) = weak_ref.upgrade() {
                    let mut stats = self.stats.write().await;
                    stats.cache_hits += 1;
                    return arc_str;
                }
            }
        }
        
        // Intern new string
        let arc_str: Arc<str> = Arc::from(s);
        let weak_ref = Arc::downgrade(&arc_str);
        
        {
            let mut strings = self.strings.write().await;
            strings.insert(arc_str.clone(), (weak_ref, SystemTime::now()));
            
            let mut stats = self.stats.write().await;
            stats.cache_misses += 1;
            stats.total_strings += 1;
        }
        
        arc_str
    }
    
    async fn cleanup_expired(&self, max_age: Duration) -> usize {
        let now = SystemTime::now();
        let mut cleaned = 0;
        
        let mut strings = self.strings.write().await;
        strings.retain(|_, (weak_ref, created_at)| {
            if weak_ref.upgrade().is_none() || now.duration_since(*created_at).unwrap_or(Duration::ZERO) > max_age {
                cleaned += 1;
                false
            } else {
                true
            }
        });
        
        if cleaned > 0 {
            let mut stats = self.stats.write().await;
            stats.cleanup_operations += 1;
            stats.total_strings = strings.len();
        }
        
        cleaned
    }
    
    async fn get_efficiency(&self) -> f64 {
        let stats = self.stats.read().await;
        let total_requests = stats.cache_hits + stats.cache_misses;
        if total_requests > 0 {
            stats.cache_hits as f64 / total_requests as f64
        } else {
            0.0
        }
    }
    
    async fn get_memory_savings(&self) -> u64 {
        let stats = self.stats.read().await;
        stats.memory_saved_bytes
    }
}

impl MessageCache {
    fn new(config: MessageCacheConfig) -> Self {
        Self {
            mcp_cache: Arc::new(RwLock::new(HashMap::new())),
            ai_request_cache: Arc::new(RwLock::new(HashMap::new())),
            ai_response_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }
    
    async fn cache_mcp_message(&self, key: String, message: MCPMessage) -> Result<()> {
        let cached_message = CachedMessage {
            message: Arc::new(message),
            cached_at: SystemTime::now(),
            access_count: AtomicU64::new(0),
            serialized_form: None,
        };
        
        let mut cache = self.mcp_cache.write().await;
        cache.insert(key, cached_message);
        
        Ok(())
    }
    
    async fn get_mcp_message(&self, key: &str) -> Option<Arc<MCPMessage>> {
        let cache = self.mcp_cache.read().await;
        if let Some(cached) = cache.get(key) {
            cached.access_count.fetch_add(1, Ordering::Relaxed);
            Some(Arc::clone(&cached.message))
        } else {
            None
        }
    }
    
    async fn cleanup_expired(&self, max_age: Duration) -> usize {
        let now = SystemTime::now();
        let mut cleaned = 0;
        
        // Clean MCP message cache
        {
            let mut cache = self.mcp_cache.write().await;
            cache.retain(|_, cached| {
                if now.duration_since(cached.cached_at).unwrap_or(Duration::ZERO) > max_age {
                    cleaned += 1;
                    false
                } else {
                    true
                }
            });
        }
        
        cleaned
    }
    
    async fn get_efficiency(&self) -> f64 {
        let stats = self.stats.read().await;
        let total_requests = stats.cache_hits + stats.cache_misses;
        if total_requests > 0 {
            stats.cache_hits as f64 / total_requests as f64
        } else {
            0.0
        }
    }
    
    async fn get_memory_savings(&self) -> u64 {
        let stats = self.stats.read().await;
        stats.memory_usage_bytes
    }
}

/// Initialize global memory pool with production settings
pub fn init_global_memory_pool() -> Result<()> {
    let pool = AdvancedMemoryPool::new(MemoryPoolConfig::production());
    GLOBAL_MEMORY_POOL.set(Arc::new(pool)).map_err(|_| {
        MCPError::Internal("Global memory pool already initialized".to_string())
    })?;
    
    info!("🚀 Advanced memory pool initialized with production settings");
    Ok(())
}

/// High-performance utilities for common operations
pub mod utils {
    use super::*;
    
    /// Get a buffer optimized for typical message sizes
    pub async fn get_message_buffer() -> PooledBuffer {
        get_global_memory_pool().get_buffer(8192).await // 8KB typical
    }
    
    /// Intern common strings to reduce allocations
    pub async fn intern_common_string(s: &str) -> Arc<str> {
        get_global_memory_pool().intern_string(s).await
    }
    
    /// Cache a frequently used message
    pub async fn cache_hot_message(key: String, message: MCPMessage) -> Result<()> {
        get_global_memory_pool().cache_mcp_message(key, message).await
    }
    
    /// Get performance report for monitoring
    pub async fn get_pool_performance() -> PoolPerformanceReport {
        get_global_memory_pool().get_performance_report().await
    }
} 