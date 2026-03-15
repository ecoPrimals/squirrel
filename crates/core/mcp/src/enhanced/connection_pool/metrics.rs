// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Connection Pool Metrics
//!
//! This module provides detailed metrics tracking for the HTTP connection pool,
//! including connection usage, performance statistics, and health monitoring.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Connection pool metrics for monitoring and analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectionPoolMetrics {
    /// Total number of connections created
    pub total_connections_created: u64,
    
    /// Current active connections
    pub active_connections: u64,
    
    /// Total requests processed
    pub total_requests: u64,
    
    /// Successful requests
    pub successful_requests: u64,
    
    /// Failed requests
    pub failed_requests: u64,
    
    /// Connections cleaned up due to idle timeout
    pub connections_cleaned_up: u64,
    
    /// Average response time across all providers
    pub avg_response_time_ms: f64,
    
    /// Connection pool efficiency (successful requests / total requests)
    pub efficiency_rate: f64,
    
    /// Provider-specific metrics
    pub provider_metrics: HashMap<String, ProviderMetrics>,
    
    /// Connection pool uptime
    pub uptime_seconds: u64,
    
    /// Pool creation timestamp
    pub created_at: Instant,
    
    /// Last metrics update timestamp
    pub last_updated: Instant,
}

/// Metrics for individual AI providers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderMetrics {
    /// Provider name
    pub provider_name: String,
    
    /// Total requests to this provider
    pub total_requests: u64,
    
    /// Successful requests
    pub successful_requests: u64,
    
    /// Failed requests
    pub failed_requests: u64,
    
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    
    /// Minimum response time in milliseconds
    pub min_response_time_ms: f64,
    
    /// Maximum response time in milliseconds
    pub max_response_time_ms: f64,
    
    /// Total bytes sent to provider
    pub bytes_sent: u64,
    
    /// Total bytes received from provider
    pub bytes_received: u64,
    
    /// Current concurrent connections
    pub active_connections: u32,
    
    /// Maximum concurrent connections reached
    pub peak_connections: u32,
    
    /// Connection errors count
    pub connection_errors: u64,
    
    /// Timeout errors count
    pub timeout_errors: u64,
    
    /// Rate limit errors count
    pub rate_limit_errors: u64,
    
    /// Current health status
    pub is_healthy: bool,
    
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
    
    /// Last error message
    pub last_error: Option<String>,
    
    /// Last successful request timestamp
    pub last_success: Option<Instant>,
    
    /// Last error timestamp
    pub last_error_at: Option<Instant>,
}

/// Performance analytics for connection pool optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalytics {
    /// Request latency percentiles
    pub latency_percentiles: LatencyPercentiles,
    
    /// Throughput statistics
    pub throughput: ThroughputStats,
    
    /// Resource utilization
    pub resource_utilization: ResourceUtilization,
    
    /// Error analysis
    pub error_analysis: ErrorAnalysis,
    
    /// Efficiency trends
    pub efficiency_trends: EfficiencyTrends,
}

/// Request latency percentiles for performance analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyPercentiles {
    /// 50th percentile (median) response time in milliseconds
    pub p50_ms: f64,
    
    /// 90th percentile response time in milliseconds
    pub p90_ms: f64,
    
    /// 95th percentile response time in milliseconds
    pub p95_ms: f64,
    
    /// 99th percentile response time in milliseconds
    pub p99_ms: f64,
    
    /// 99.9th percentile response time in milliseconds
    pub p999_ms: f64,
}

/// Throughput statistics for load analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputStats {
    /// Requests per second (current)
    pub requests_per_second: f64,
    
    /// Peak requests per second
    pub peak_rps: f64,
    
    /// Average requests per minute
    pub avg_requests_per_minute: f64,
    
    /// Total data transferred (bytes)
    pub total_bytes_transferred: u64,
    
    /// Data transfer rate (bytes per second)
    pub bytes_per_second: f64,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    /// Connection pool utilization (0.0 to 1.0)
    pub pool_utilization: f64,
    
    /// Memory usage estimate (bytes)
    pub estimated_memory_bytes: u64,
    
    /// CPU usage estimate (0.0 to 1.0)
    pub estimated_cpu_usage: f64,
    
    /// Network bandwidth utilization
    pub network_utilization: f64,
}

/// Error analysis for debugging and optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAnalysis {
    /// Error breakdown by type
    pub error_types: HashMap<String, u64>,
    
    /// Error rate trend (increasing/decreasing/stable)
    pub error_trend: ErrorTrend,
    
    /// Most common error
    pub most_common_error: Option<String>,
    
    /// Error frequency by time period
    pub error_frequency: HashMap<String, u64>,
}

/// Error trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorTrend {
    /// Error rate is increasing
    Increasing,
    /// Error rate is decreasing
    Decreasing,
    /// Error rate is stable
    Stable,
    /// Not enough data to determine trend
    Unknown,
}

/// Efficiency trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyTrends {
    /// Connection reuse rate
    pub connection_reuse_rate: f64,
    
    /// Cache hit rate (if applicable)
    pub cache_hit_rate: f64,
    
    /// Resource optimization score (0.0 to 1.0)
    pub optimization_score: f64,
    
    /// Performance improvement over time
    pub performance_improvement: f64,
}

impl ConnectionPoolMetrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            created_at: now,
            last_updated: now,
            ..Default::default()
        }
    }
    
    /// Update efficiency rate based on current statistics
    pub fn update_efficiency_rate(&mut self) {
        if self.total_requests > 0 {
            self.efficiency_rate = self.successful_requests as f64 / self.total_requests as f64;
        } else {
            self.efficiency_rate = 1.0;
        }
        self.last_updated = Instant::now();
    }
    
    /// Update uptime
    pub fn update_uptime(&mut self) {
        self.uptime_seconds = self.created_at.elapsed().as_secs();
        self.last_updated = Instant::now();
    }
    
    /// Record a successful request
    pub fn record_success(&mut self, provider_name: &str, response_time: Duration, bytes_sent: u64, bytes_received: u64) {
        self.total_requests += 1;
        self.successful_requests += 1;
        self.update_average_response_time(response_time.as_millis() as f64);
        
        // Update provider-specific metrics
        let provider_metrics = self.provider_metrics
            .entry(provider_name.to_string())
            .or_insert_with(|| ProviderMetrics {
                provider_name: provider_name.to_string(),
                ..Default::default()
            });
        
        provider_metrics.record_success(response_time, bytes_sent, bytes_received);
        
        self.update_efficiency_rate();
    }
    
    /// Record a failed request
    pub fn record_failure(&mut self, provider_name: &str, error: &str, response_time: Option<Duration>) {
        self.total_requests += 1;
        self.failed_requests += 1;
        
        if let Some(duration) = response_time {
            self.update_average_response_time(duration.as_millis() as f64);
        }
        
        // Update provider-specific metrics
        let provider_metrics = self.provider_metrics
            .entry(provider_name.to_string())
            .or_insert_with(|| ProviderMetrics {
                provider_name: provider_name.to_string(),
                ..Default::default()
            });
        
        provider_metrics.record_failure(error, response_time);
        
        self.update_efficiency_rate();
    }
    
    /// Update average response time with new measurement
    fn update_average_response_time(&mut self, new_response_time: f64) {
        if self.total_requests == 1 {
            self.avg_response_time_ms = new_response_time;
        } else {
            // Use exponential moving average
            let alpha = 0.1; // Smoothing factor
            self.avg_response_time_ms = (alpha * new_response_time) + ((1.0 - alpha) * self.avg_response_time_ms);
        }
    }
    
    /// Get provider metrics for a specific provider
    pub fn get_provider_metrics(&self, provider_name: &str) -> Option<&ProviderMetrics> {
        self.provider_metrics.get(provider_name)
    }
    
    /// Generate performance report
    pub fn generate_performance_report(&self) -> PerformanceReport {
        PerformanceReport {
            summary: self.generate_summary(),
            provider_breakdown: self.provider_metrics.clone(),
            recommendations: self.generate_recommendations(),
            generated_at: Instant::now(),
        }
    }
    
    /// Generate metrics summary
    fn generate_summary(&self) -> MetricsSummary {
        MetricsSummary {
            total_requests: self.total_requests,
            success_rate: self.efficiency_rate,
            avg_response_time_ms: self.avg_response_time_ms,
            active_connections: self.active_connections,
            uptime_seconds: self.uptime_seconds,
            provider_count: self.provider_metrics.len() as u32,
        }
    }
    
    /// Generate optimization recommendations
    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Check efficiency rate
        if self.efficiency_rate < 0.95 && self.total_requests > 100 {
            recommendations.push("Consider investigating high error rates affecting efficiency".to_string());
        }
        
        // Check response times
        if self.avg_response_time_ms > 5000.0 {
            recommendations.push("Average response time is high - consider connection pool optimization".to_string());
        }
        
        // Check provider health
        for (provider_name, metrics) in &self.provider_metrics {
            if metrics.error_rate > 0.1 {
                recommendations.push(format!("Provider '{}' has elevated error rate: {:.2}%", 
                    provider_name, metrics.error_rate * 100.0));
            }
            
            if metrics.avg_response_time_ms > 10000.0 {
                recommendations.push(format!("Provider '{}' has slow response times: {:.2}ms", 
                    provider_name, metrics.avg_response_time_ms));
            }
        }
        
        recommendations
    }
}

impl ProviderMetrics {
    /// Record a successful request for this provider
    pub fn record_success(&mut self, response_time: Duration, bytes_sent: u64, bytes_received: u64) {
        self.total_requests += 1;
        self.successful_requests += 1;
        self.bytes_sent += bytes_sent;
        self.bytes_received += bytes_received;
        self.last_success = Some(Instant::now());
        
        let response_time_ms = response_time.as_millis() as f64;
        self.update_response_times(response_time_ms);
        self.update_success_rate();
        self.is_healthy = true;
    }
    
    /// Record a failed request for this provider
    pub fn record_failure(&mut self, error: &str, response_time: Option<Duration>) {
        self.total_requests += 1;
        self.failed_requests += 1;
        self.last_error = Some(error.to_string());
        self.last_error_at = Some(Instant::now());
        
        // Categorize error types
        if error.contains("timeout") {
            self.timeout_errors += 1;
        } else if error.contains("rate limit") || error.contains("429") {
            self.rate_limit_errors += 1;
        } else {
            self.connection_errors += 1;
        }
        
        if let Some(duration) = response_time {
            let response_time_ms = duration.as_millis() as f64;
            self.update_response_times(response_time_ms);
        }
        
        self.update_success_rate();
        
        // Update health status based on recent error rate
        self.is_healthy = self.error_rate < 0.2; // Consider unhealthy if >20% error rate
    }
    
    /// Update response time statistics
    fn update_response_times(&mut self, response_time_ms: f64) {
        // Update min/max
        if self.min_response_time_ms == 0.0 || response_time_ms < self.min_response_time_ms {
            self.min_response_time_ms = response_time_ms;
        }
        if response_time_ms > self.max_response_time_ms {
            self.max_response_time_ms = response_time_ms;
        }
        
        // Update average (exponential moving average)
        if self.total_requests == 1 {
            self.avg_response_time_ms = response_time_ms;
        } else {
            let alpha = 0.1;
            self.avg_response_time_ms = (alpha * response_time_ms) + ((1.0 - alpha) * self.avg_response_time_ms);
        }
    }
    
    /// Update success and error rates
    fn update_success_rate(&mut self) {
        if self.total_requests > 0 {
            self.success_rate = self.successful_requests as f64 / self.total_requests as f64;
            self.error_rate = self.failed_requests as f64 / self.total_requests as f64;
        }
    }
}

/// Performance report for analysis and optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// High-level metrics summary
    pub summary: MetricsSummary,
    
    /// Detailed provider breakdown
    pub provider_breakdown: HashMap<String, ProviderMetrics>,
    
    /// Optimization recommendations
    pub recommendations: Vec<String>,
    
    /// Report generation timestamp
    pub generated_at: Instant,
}

/// High-level metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    /// Total requests processed
    pub total_requests: u64,
    
    /// Overall success rate
    pub success_rate: f64,
    
    /// Average response time
    pub avg_response_time_ms: f64,
    
    /// Active connections
    pub active_connections: u64,
    
    /// Pool uptime in seconds
    pub uptime_seconds: u64,
    
    /// Number of registered providers
    pub provider_count: u32,
}

impl PerformanceReport {
    /// Print a formatted report to logs
    pub fn log_report(&self) {
        info!("=== Connection Pool Performance Report ===");
        info!("Total Requests: {}", self.summary.total_requests);
        info!("Success Rate: {:.2}%", self.summary.success_rate * 100.0);
        info!("Avg Response Time: {:.2}ms", self.summary.avg_response_time_ms);
        info!("Active Connections: {}", self.summary.active_connections);
        info!("Pool Uptime: {}s", self.summary.uptime_seconds);
        info!("Provider Count: {}", self.summary.provider_count);
        
        if !self.recommendations.is_empty() {
            info!("Recommendations:");
            for (i, rec) in self.recommendations.iter().enumerate() {
                info!("  {}. {}", i + 1, rec);
            }
        }
        info!("==========================================");
    }
} 