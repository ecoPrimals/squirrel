//! Connection pool for efficient HTTP client management
//!
//! This module provides a connection pool that manages HTTP clients per endpoint,
//! tracks connection statistics, and performs automatic cleanup of stale connections.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::types::{ConnectionPoolHealthMetrics, ConnectionStats};

/// Connection pool for efficient HTTP client management
#[derive(Debug)]
pub struct ServiceConnectionPool {
    /// Per-endpoint HTTP clients
    clients: Arc<RwLock<HashMap<String, reqwest::Client>>>,
    /// Connection statistics
    stats: Arc<RwLock<HashMap<String, ConnectionStats>>>,
}

impl Default for ServiceConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceConnectionPool {
    /// Creates a new service connection pool
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create an HTTP client for the given endpoint with resource management
    pub async fn get_client(&self, endpoint: &str) -> reqwest::Client {
        let mut clients = self.clients.write().await;

        if let Some(client) = clients.get(endpoint) {
            // Update last used time in stats
            if self.stats.read().await.get(endpoint).is_some() {
                // Client exists and was recently used, return it
                client.clone()
            } else {
                // Client exists but no recent stats, still return it but log
                debug!(
                    "Returning cached client for {} without recent stats",
                    endpoint
                );
                client.clone()
            }
        } else {
            // Create new client with production-safe configuration
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(30)) // Request timeout
                .connect_timeout(Duration::from_secs(10)) // Connection timeout
                .pool_max_idle_per_host(5) // Reduced from 10 to prevent resource exhaustion
                .pool_idle_timeout(Duration::from_secs(30)) // Reduced from 90s for faster cleanup
                .tcp_keepalive(Duration::from_secs(60)) // TCP keepalive for connection health
                .tcp_nodelay(true) // Reduce latency
                .http2_keep_alive_interval(Duration::from_secs(30)) // HTTP/2 keepalive
                .http2_keep_alive_timeout(Duration::from_secs(10)) // HTTP/2 keepalive timeout
                .build()
                .unwrap_or_else(|e| {
                    warn!(
                        "Failed to create optimized client for {}: {}, using default",
                        endpoint, e
                    );
                    reqwest::Client::new()
                });

            info!(
                endpoint = %endpoint,
                operation = "http_client_created",
                "Created new HTTP client with resource management settings"
            );

            clients.insert(endpoint.to_string(), client.clone());
            client
        }
    }

    /// Record request statistics with connection health tracking
    pub async fn record_request(&self, endpoint: &str, response_time_ms: f64, success: bool) {
        let mut stats = self.stats.write().await;
        let entry = stats
            .entry(endpoint.to_string())
            .or_insert_with(|| ConnectionStats {
                request_count: 0,
                avg_response_time_ms: 0.0,
                last_success: None,
                failure_count: 0,
            });

        entry.request_count += 1;

        if success {
            entry.last_success = Some(Instant::now());
            // Update running average with numerical stability
            let weight = (entry.request_count - 1) as f64 / entry.request_count as f64;
            entry.avg_response_time_ms =
                entry.avg_response_time_ms * weight + response_time_ms / entry.request_count as f64;
        } else {
            entry.failure_count += 1;

            // Log connection health issues
            let failure_rate = entry.failure_count as f64 / entry.request_count as f64;
            if failure_rate > 0.1 {
                // More than 10% failure rate
                warn!(
                    endpoint = %endpoint,
                    failure_count = entry.failure_count,
                    total_requests = entry.request_count,
                    failure_rate = %format!("{:.1}%", failure_rate * 100.0),
                    operation = "connection_health_warning",
                    "High failure rate detected for endpoint"
                );
            }
        }
    }

    /// Get connection statistics for debugging/monitoring
    pub async fn get_stats(&self) -> HashMap<String, ConnectionStats> {
        self.stats.read().await.clone()
    }

    /// Cleanup stale connections and perform resource maintenance
    pub async fn cleanup_stale_connections(&self) {
        let now = Instant::now();
        let stale_threshold = Duration::from_secs(300); // 5 minutes without successful requests

        let mut clients_to_remove = Vec::new();
        let mut total_cleaned = 0;

        // Identify stale connections
        {
            let stats = self.stats.read().await;
            for (endpoint, stat) in stats.iter() {
                if let Some(last_success) = stat.last_success {
                    if now.duration_since(last_success) > stale_threshold {
                        clients_to_remove.push(endpoint.clone());
                    }
                } else if stat.request_count > 0 {
                    // No successful requests at all, but some attempts - remove
                    clients_to_remove.push(endpoint.clone());
                }
            }
        }

        // Remove stale clients and their stats
        if !clients_to_remove.is_empty() {
            let mut clients = self.clients.write().await;
            let mut stats = self.stats.write().await;

            for endpoint in &clients_to_remove {
                if clients.remove(endpoint).is_some() {
                    total_cleaned += 1;
                }
                stats.remove(endpoint);

                // Use simple string formatting to avoid span data capture
                info!(
                    "Removed stale HTTP client: endpoint={}, operation=stale_connection_cleanup",
                    endpoint
                );
            }
        }

        if total_cleaned > 0 {
            let remaining = self.clients.read().await.len();
            // Use simple string formatting to avoid span data capture
            info!(
                "Connection pool cleanup complete: cleaned={}, remaining={}, operation=connection_pool_maintenance",
                total_cleaned, remaining
            );
        }
    }

    /// Get connection pool health metrics
    pub async fn get_health_metrics(&self) -> ConnectionPoolHealthMetrics {
        let clients = self.clients.read().await;
        let stats = self.stats.read().await;

        let mut total_requests = 0;
        let mut total_failures = 0;
        let mut healthy_connections = 0;
        let mut unhealthy_connections = 0;
        let mut avg_response_time = 0.0;

        for (_endpoint, stat) in stats.iter() {
            total_requests += stat.request_count;
            total_failures += stat.failure_count;

            let failure_rate = if stat.request_count > 0 {
                stat.failure_count as f64 / stat.request_count as f64
            } else {
                0.0
            };

            if failure_rate < 0.05 && stat.avg_response_time_ms < 5000.0 {
                healthy_connections += 1;
            } else {
                unhealthy_connections += 1;
            }

            avg_response_time += stat.avg_response_time_ms;
        }

        if !stats.is_empty() {
            avg_response_time /= stats.len() as f64;
        }

        ConnectionPoolHealthMetrics {
            total_connections: clients.len(),
            healthy_connections,
            unhealthy_connections,
            total_requests,
            total_failures,
            overall_failure_rate: if total_requests > 0 {
                total_failures as f64 / total_requests as f64
            } else {
                0.0
            },
            avg_response_time_ms: avg_response_time,
        }
    }

    /// Graceful shutdown - close all connections and cleanup resources
    pub async fn shutdown(&self) {
        info!(
            operation = "connection_pool_shutdown",
            "Starting graceful shutdown of connection pool"
        );

        let start_time = Instant::now();

        // Clear all clients (reqwest clients will be dropped and connections closed)
        {
            let mut clients = self.clients.write().await;
            let connection_count = clients.len();
            clients.clear();

            info!(
                closed_connections = connection_count,
                operation = "connections_closed",
                "Closed all HTTP client connections"
            );
        }

        // Clear all stats
        {
            let mut stats = self.stats.write().await;
            stats.clear();
        }

        let shutdown_duration = start_time.elapsed();
        info!(
            shutdown_duration_ms = shutdown_duration.as_millis(),
            operation = "connection_pool_shutdown_complete",
            "Connection pool shutdown completed"
        );
    }
}
