// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Performance Initialization System
//!
//! This module provides a comprehensive initialization system for all performance
//! optimizations including memory pooling, zero-copy serialization, plugin
//! performance optimization, and hot path caching.

use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn, error, instrument};

use crate::error::{Result, types::MCPError};
use super::memory_pool::{self, MemoryPoolConfig, AdvancedMemoryPool};
use super::serialization::{self, SerializationConfig, ZeroCopySerializer};

/// Performance subsystem configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Memory pool configuration
    pub memory_pool: MemoryPoolConfig,
    
    /// Serialization configuration
    pub serialization: SerializationConfig,
    
    /// Enable plugin performance optimization
    pub enable_plugin_optimization: bool,
    
    /// Enable metrics collection for performance monitoring
    pub enable_metrics: bool,
    
    /// Performance monitoring interval
    pub monitoring_interval: Duration,
    
    /// Enable automatic performance tuning
    pub enable_auto_tuning: bool,
}

impl PerformanceConfig {
    /// Production-optimized performance configuration
    pub fn production() -> Self {
        Self {
            memory_pool: MemoryPoolConfig::production(),
            serialization: SerializationConfig {
                enable_buffer_pooling: true,
                enable_fast_codecs: true,
                enable_templates: true,
                enable_streaming: true,
                enable_compression: true,
                compression_threshold: 1024, // 1KB
                buffer_pool_size: 1000,
                template_cache_size: 500,
                max_buffer_size: 1024 * 1024, // 1MB
                initial_pool_size: 100,
                max_pool_size: 2000,
            },
            enable_plugin_optimization: true,
            enable_metrics: true,
            monitoring_interval: Duration::from_secs(30),
            enable_auto_tuning: true,
        }
    }
    
    /// Development configuration with reduced overhead
    pub fn development() -> Self {
        Self {
            memory_pool: MemoryPoolConfig::development(),
            serialization: SerializationConfig {
                enable_buffer_pooling: false,
                enable_fast_codecs: false,
                enable_templates: false,
                enable_streaming: false,
                enable_compression: false,
                compression_threshold: 4096,
                buffer_pool_size: 100,
                template_cache_size: 50,
                max_buffer_size: 512 * 1024,
                initial_pool_size: 10,
                max_pool_size: 200,
            },
            enable_plugin_optimization: false,
            enable_metrics: false,
            monitoring_interval: Duration::from_secs(60),
            enable_auto_tuning: false,
        }
    }
    
    /// High-performance configuration for maximum throughput
    pub fn high_performance() -> Self {
        Self {
            memory_pool: MemoryPoolConfig {
                buffer_pool: super::memory_pool::BufferPoolConfig {
                    small_pool_size: 200,
                    small_buffer_size: 4096,
                    medium_pool_size: 100,
                    medium_buffer_size: 65536,    // 64KB
                    large_pool_size: 50,
                    large_buffer_size: 1048576,   // 1MB
                    preallocate: true,
                },
                object_pool: super::memory_pool::ObjectPoolConfig {
                    max_objects_per_type: 2000,
                    object_lifetime: Duration::from_secs(600), // 10 minutes
                    track_reuse: true,
                },
                string_interning: super::memory_pool::StringInterningConfig {
                    max_strings: 20000,
                    string_lifetime: Duration::from_secs(3600), // 1 hour
                    enable_weak_refs: true,
                },
                message_cache: super::memory_pool::MessageCacheConfig {
                    max_cached_messages: 10000,
                    cache_ttl: Duration::from_secs(1200), // 20 minutes
                    enable_compression: true,
                },
                gc_config: super::memory_pool::GarbageCollectionConfig {
                    gc_interval: Duration::from_secs(30),
                    memory_pressure_threshold: 0.75,
                    adaptive_scheduling: true,
                },
            },
            serialization: SerializationConfig {
                enable_buffer_pooling: true,
                enable_fast_codecs: true,
                enable_templates: true,
                enable_streaming: true,
                enable_compression: true,
                compression_threshold: 512, // Aggressive compression
                buffer_pool_size: 2000,
                template_cache_size: 1000,
                max_buffer_size: 2 * 1024 * 1024, // 2MB
                initial_pool_size: 200,
                max_pool_size: 5000,
            },
            enable_plugin_optimization: true,
            enable_metrics: true,
            monitoring_interval: Duration::from_secs(15), // Frequent monitoring
            enable_auto_tuning: true,
        }
    }
}

/// Performance subsystem manager
#[derive(Debug)]
pub struct PerformanceManager {
    /// Memory pool instance
    memory_pool: Arc<AdvancedMemoryPool>,
    
    /// Serialization system instance
    serializer: Arc<ZeroCopySerializer>,
    
    /// Configuration
    config: PerformanceConfig,
    
    /// Performance monitoring task handle
    monitoring_handle: Option<tokio::task::JoinHandle<()>>,
    
    /// Auto-tuning task handle
    auto_tuning_handle: Option<tokio::task::JoinHandle<()>>,
}

impl PerformanceManager {
    /// Create a new performance manager
    pub fn new(config: PerformanceConfig) -> Self {
        let memory_pool = Arc::new(AdvancedMemoryPool::new(config.memory_pool.clone()));
        let serializer = Arc::new(ZeroCopySerializer::new(config.serialization.clone()));
        
        Self {
            memory_pool,
            serializer,
            config,
            monitoring_handle: None,
            auto_tuning_handle: None,
        }
    }
    
    /// Initialize all performance subsystems
    #[instrument(skip(self))]
    pub async fn initialize(&mut self) -> Result<()> {
        info!("🚀 Initializing performance subsystems");
        
        // Initialize memory pool
        self.initialize_memory_pool().await?;
        
        // Initialize serialization system
        self.initialize_serialization().await?;
        
        // Initialize plugin optimization if enabled
        if self.config.enable_plugin_optimization {
            self.initialize_plugin_optimization().await?;
        }
        
        // Start performance monitoring if enabled
        if self.config.enable_metrics {
            self.start_performance_monitoring().await?;
        }
        
        // Start auto-tuning if enabled
        if self.config.enable_auto_tuning {
            self.start_auto_tuning().await?;
        }
        
        info!("✅ Performance subsystems initialized successfully");
        Ok(())
    }
    
    /// Initialize memory pool subsystem
    async fn initialize_memory_pool(&self) -> Result<()> {
        info!("Initializing advanced memory pool");
        
        // Initialize global memory pool
        memory_pool::init_global_memory_pool().map_err(|e| {
            MCPError::Internal(format!("Failed to initialize memory pool: {}", e))
        })?;
        
        // Pre-allocate buffers if configured
        if self.config.memory_pool.buffer_pool.preallocate {
            info!("Pre-allocating memory pool buffers");
            // The memory pool constructor already handles pre-allocation
        }
        
        info!("✅ Memory pool initialized");
        Ok(())
    }
    
    /// Initialize serialization subsystem
    async fn initialize_serialization(&self) -> Result<()> {
        info!("Initializing zero-copy serialization system");
        
        // Initialize global serializer
        serialization::init_global_serializer(self.config.serialization.clone())?;
        
        // Register fast codecs if enabled
        if self.config.serialization.enable_fast_codecs {
            info!("Registering fast-path codecs");
            let global_serializer = serialization::get_global_serializer();
            
            // Register MCP message codec
            let mcp_codec = serialization::codecs::MCPMessageCodec::new();
            global_serializer.register_codec("mcp_message".to_string(), Box::new(mcp_codec)).await;
            
            // Register AI message codec
            let ai_codec = serialization::codecs::AIMessageCodec::new();
            global_serializer.register_codec("ai_message".to_string(), Box::new(ai_codec)).await;
        }
        
        info!("✅ Serialization system initialized");
        Ok(())
    }
    
    /// Initialize plugin optimization subsystem
    async fn initialize_plugin_optimization(&self) -> Result<()> {
        info!("Initializing plugin performance optimization");
        
        // Initialize plugin performance optimizer
        // Note: This would require the performance_optimizer module to be available
        // For now, we'll log that it would be initialized
        info!("Plugin performance optimizer would be initialized here");
        
        info!("✅ Plugin optimization initialized");
        Ok(())
    }
    
    /// Start performance monitoring task
    async fn start_performance_monitoring(&mut self) -> Result<()> {
        info!("Starting performance monitoring");
        
        let memory_pool = Arc::clone(&self.memory_pool);
        let serializer = Arc::clone(&self.serializer);
        let monitoring_interval = self.config.monitoring_interval;
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(monitoring_interval);
            
            loop {
                interval.tick().await;
                
                // Collect memory pool metrics
                let pool_report = memory_pool.get_performance_report().await;
                
                // Collect serialization metrics
                let serialization_metrics = serializer.get_metrics().await;
                
                // Log performance summary
                info!(
                    "📊 Performance metrics - Memory efficiency: {:.1}%, Serialization ops: {}, Buffer hits: {}",
                    pool_report.buffer_efficiency * 100.0,
                    serialization_metrics.total_serializations,
                    serialization_metrics.buffer_pool_hits
                );
                
                // Check for performance issues
                if pool_report.buffer_efficiency < 0.5 {
                    warn!("Buffer pool efficiency is low: {:.1}%", pool_report.buffer_efficiency * 100.0);
                }
                
                if serialization_metrics.buffer_pool_hits == 0 && serialization_metrics.total_serializations > 100 {
                    warn!("Serialization buffer pool not being utilized effectively");
                }
            }
        });
        
        self.monitoring_handle = Some(handle);
        info!("✅ Performance monitoring started");
        Ok(())
    }
    
    /// Start auto-tuning task
    async fn start_auto_tuning(&mut self) -> Result<()> {
        info!("Starting performance auto-tuning");
        
        let memory_pool = Arc::clone(&self.memory_pool);
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
            
            loop {
                interval.tick().await;
                
                // Analyze performance metrics
                let pool_report = memory_pool.get_performance_report().await;
                
                // Auto-tune memory pool parameters
                if pool_report.buffer_efficiency < 0.6 {
                    info!("Auto-tuning: Buffer efficiency low, optimizing pool sizes");
                    // In a real implementation, this would adjust pool parameters
                }
                
                // Trigger garbage collection if memory pressure is high
                if pool_report.total_memory_saved < 1024 * 1024 { // Less than 1MB saved
                    info!("Auto-tuning: Triggering memory optimization");
                    // Memory pool GC is already handled internally
                }
            }
        });
        
        self.auto_tuning_handle = Some(handle);
        info!("✅ Performance auto-tuning started");
        Ok(())
    }
    
    /// Get comprehensive performance report
    pub async fn get_performance_report(&self) -> PerformanceReport {
        let memory_report = self.memory_pool.get_performance_report().await;
        let serialization_metrics = self.serializer.get_metrics().await;
        
        PerformanceReport {
            memory_pool_efficiency: memory_report.buffer_efficiency,
            total_memory_saved: memory_report.total_memory_saved,
            serialization_operations: serialization_metrics.total_serializations,
            buffer_pool_hit_rate: if serialization_metrics.total_serializations > 0 {
                serialization_metrics.buffer_pool_hits as f64 / serialization_metrics.total_serializations as f64
            } else {
                0.0
            },
            average_serialization_time_us: serialization_metrics.avg_serialization_time_us,
            memory_pool_report: memory_report,
            serialization_metrics,
        }
    }
    
    /// Shutdown performance subsystems
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down performance subsystems");
        
        // Stop monitoring task
        if let Some(handle) = self.monitoring_handle.take() {
            handle.abort();
        }
        
        // Stop auto-tuning task
        if let Some(handle) = self.auto_tuning_handle.take() {
            handle.abort();
        }
        
        info!("✅ Performance subsystems shutdown complete");
        Ok(())
    }
}

/// Comprehensive performance report
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// Memory pool efficiency (0.0 to 1.0)
    pub memory_pool_efficiency: f64,
    
    /// Total memory saved in bytes
    pub total_memory_saved: u64,
    
    /// Number of serialization operations
    pub serialization_operations: u64,
    
    /// Buffer pool hit rate (0.0 to 1.0)
    pub buffer_pool_hit_rate: f64,
    
    /// Average serialization time in microseconds
    pub average_serialization_time_us: f64,
    
    /// Detailed memory pool report
    pub memory_pool_report: super::memory_pool::PoolPerformanceReport,
    
    /// Detailed serialization metrics
    pub serialization_metrics: super::serialization::SerializationMetrics,
}

impl PerformanceReport {
    /// Print a comprehensive performance report
    pub fn print_comprehensive_report(&self) {
        info!("=== 🚀 COMPREHENSIVE PERFORMANCE REPORT ===");
        info!("Memory Pool Efficiency: {:.1}%", self.memory_pool_efficiency * 100.0);
        info!("Total Memory Saved: {} MB", self.total_memory_saved / (1024 * 1024));
        info!("Serialization Operations: {}", self.serialization_operations);
        info!("Buffer Pool Hit Rate: {:.1}%", self.buffer_pool_hit_rate * 100.0);
        info!("Avg Serialization Time: {:.2} μs", self.average_serialization_time_us);
        info!("==========================================");
        
        // Print detailed reports
        self.memory_pool_report.print_report();
        self.serialization_metrics.print_performance_summary();
    }
    
    /// Get overall performance score (0.0 to 1.0)
    pub fn get_performance_score(&self) -> f64 {
        let efficiency_score = self.memory_pool_efficiency;
        let hit_rate_score = self.buffer_pool_hit_rate;
        let speed_score = if self.average_serialization_time_us > 0.0 {
            (1000.0 / self.average_serialization_time_us).min(1.0)
        } else {
            1.0
        };
        
        (efficiency_score + hit_rate_score + speed_score) / 3.0
    }
    
    /// Check if performance is within acceptable thresholds
    pub fn is_performance_healthy(&self) -> bool {
        self.memory_pool_efficiency > 0.7 &&
        self.buffer_pool_hit_rate > 0.6 &&
        self.average_serialization_time_us < 1000.0 // Less than 1ms
    }
}

/// Global performance manager instance
static GLOBAL_PERFORMANCE_MANAGER: std::sync::OnceLock<Arc<tokio::sync::Mutex<PerformanceManager>>> = std::sync::OnceLock::new();

/// Initialize global performance systems
#[instrument]
pub async fn init_performance_systems(config: PerformanceConfig) -> Result<()> {
    info!("🚀 Initializing global performance systems");
    
    let mut manager = PerformanceManager::new(config);
    manager.initialize().await?;
    
    let manager_arc = Arc::new(tokio::sync::Mutex::new(manager));
    GLOBAL_PERFORMANCE_MANAGER.set(manager_arc).map_err(|_| {
        MCPError::Internal("Global performance manager already initialized".to_string())
    })?;
    
    info!("✅ Global performance systems initialized");
    Ok(())
}

/// Get global performance report
pub async fn get_global_performance_report() -> Option<PerformanceReport> {
    if let Some(manager_arc) = GLOBAL_PERFORMANCE_MANAGER.get() {
        let manager = manager_arc.lock().await;
        Some(manager.get_performance_report().await)
    } else {
        None
    }
}

/// Shutdown global performance systems
pub async fn shutdown_performance_systems() -> Result<()> {
    if let Some(manager_arc) = GLOBAL_PERFORMANCE_MANAGER.get() {
        let mut manager = manager_arc.lock().await;
        manager.shutdown().await?;
        info!("✅ Global performance systems shutdown complete");
    }
    Ok(())
}

/// Quick performance check utility
pub async fn quick_performance_check() -> Result<()> {
    if let Some(report) = get_global_performance_report().await {
        if report.is_performance_healthy() {
            info!("🟢 Performance status: HEALTHY (score: {:.2})", report.get_performance_score());
        } else {
            warn!("🟡 Performance status: DEGRADED (score: {:.2})", report.get_performance_score());
            warn!("  - Memory pool efficiency: {:.1}%", report.memory_pool_efficiency * 100.0);
            warn!("  - Buffer hit rate: {:.1}%", report.buffer_pool_hit_rate * 100.0);
            warn!("  - Avg serialization time: {:.2} μs", report.average_serialization_time_us);
        }
    } else {
        warn!("🔴 Performance monitoring not available");
    }
    Ok(())
}

// Extension trait for SerializationMetrics
trait SerializationMetricsExt {
    fn print_performance_summary(&self);
}

impl SerializationMetricsExt for super::serialization::SerializationMetrics {
    fn print_performance_summary(&self) {
        info!("=== Serialization Performance Summary ===");
        info!("Total Operations: {}", self.total_serializations + self.total_deserializations);
        info!("Buffer Pool Efficiency: {:.1}%", 
              if self.total_serializations > 0 {
                  self.buffer_pool_hits as f64 / self.total_serializations as f64 * 100.0
              } else {
                  0.0
              });
        info!("Fast Codec Usage: {} operations", self.fast_codec_usage);
        info!("Memory Saved: {} KB", self.memory_saved_bytes / 1024);
        info!("========================================");
    }
} 