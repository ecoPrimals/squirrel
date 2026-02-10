// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Connection Pool Manager
//!
//! This module provides the high-level manager for HTTP connection pools,
//! integrating with AI coordinators and handling request routing, load balancing,
//! and connection lifecycle management.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex, Semaphore};
use tracing::{info, warn, error, debug, instrument};

use super::{ConnectionPool, ConnectionPoolConfig, ProviderConnectionConfig, TlsConfig, RateLimitConfig};
use super::types::*;
use super::metrics::*;
use crate::error::{Result, types::MCPError};
use crate::enhanced::coordinator::UniversalAIRequest;

/// High-level connection pool manager
#[derive(Debug)]
pub struct ConnectionPoolManager {
    /// Connection pools by provider
    pools: Arc<RwLock<HashMap<String, Arc<ConnectionPool>>>>,
    
    /// Global pool configuration
    config: ConnectionPoolConfig,
    
    /// Provider-specific configurations
    provider_configs: Arc<RwLock<HashMap<String, ProviderConnectionConfig>>>,
    
    /// Rate limiters by provider
    rate_limiters: Arc<RwLock<HashMap<String, Arc<RateLimiter>>>>,
    
    /// Global metrics aggregator
    global_metrics: Arc<RwLock<GlobalPoolMetrics>>,
    
    /// Request router for load balancing
    request_router: Arc<RequestRouter>,
    
    /// Connection health monitor
    health_monitor: Arc<HealthMonitor>,
    
    /// Background task handles
    background_tasks: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}

/// Global metrics across all connection pools
#[derive(Debug, Clone, Default)]
pub struct GlobalPoolMetrics {
    /// Total pools managed
    pub total_pools: usize,
    
    /// Total connections across all pools
    pub total_connections: usize,
    
    /// Total active connections
    pub total_active_connections: usize,
    
    /// Total requests processed
    pub total_requests: u64,
    
    /// Global success rate
    pub global_success_rate: f64,
    
    /// Average response time across all providers
    pub global_avg_response_time_ms: f64,
    
    /// Pool with highest load
    pub highest_load_provider: Option<String>,
    
    /// Pool with best performance
    pub best_performance_provider: Option<String>,
    
    /// Metrics collection timestamp
    pub last_updated: Instant,
}

/// Request router for intelligent load balancing
#[derive(Debug)]
pub struct RequestRouter {
    /// Routing strategy
    strategy: RoutingStrategy,
    
    /// Provider weights for weighted round-robin
    provider_weights: Arc<RwLock<HashMap<String, f64>>>,
    
    /// Round-robin counter
    round_robin_counter: Arc<Mutex<usize>>,
    
    /// Request history for performance-based routing
    performance_history: Arc<RwLock<HashMap<String, PerformanceMetrics>>>,
}

/// Routing strategies for load balancing
#[derive(Debug, Clone)]
pub enum RoutingStrategy {
    /// Round-robin across providers
    RoundRobin,
    
    /// Route to provider with least active connections
    LeastConnections,
    
    /// Route based on provider response times
    FastestResponse,
    
    /// Weighted round-robin based on provider capacity
    WeightedRoundRobin,
    
    /// Route based on provider health and performance
    PerformanceBased,
    
    /// Route to specific provider (no load balancing)
    Direct(String),
}

/// Performance metrics for routing decisions
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    /// Average response time
    pub avg_response_time: Duration,
    
    /// Current success rate
    pub success_rate: f64,
    
    /// Current load (active connections / max connections)
    pub load: f64,
    
    /// Health score (0.0 to 1.0)
    pub health_score: f64,
    
    /// Last update timestamp
    pub last_updated: Instant,
}

/// Health monitor for connection pools
#[derive(Debug)]
pub struct HealthMonitor {
    /// Health check interval
    check_interval: Duration,
    
    /// Health status by provider
    provider_health: Arc<RwLock<HashMap<String, ProviderHealthStatus>>>,
    
    /// Unhealthy provider quarantine
    quarantine: Arc<RwLock<HashMap<String, Instant>>>,
    
    /// Quarantine duration
    quarantine_duration: Duration,
}

/// Detailed health status for providers
#[derive(Debug, Clone)]
pub struct ProviderHealthStatus {
    /// Is provider currently healthy?
    pub is_healthy: bool,
    
    /// Consecutive failed health checks
    pub consecutive_failures: u32,
    
    /// Last successful health check
    pub last_success: Option<Instant>,
    
    /// Last health check error
    pub last_error: Option<String>,
    
    /// Health check response time
    pub response_time: Duration,
    
    /// Health trend (improving/degrading/stable)
    pub trend: HealthTrend,
}

/// Health trend analysis
#[derive(Debug, Clone)]
pub enum HealthTrend {
    /// Health is improving
    Improving,
    /// Health is degrading
    Degrading,
    /// Health is stable
    Stable,
    /// Not enough data
    Unknown,
}

impl ConnectionPoolManager {
    /// Create a new connection pool manager
    pub fn new(config: ConnectionPoolConfig) -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            config,
            provider_configs: Arc::new(RwLock::new(HashMap::new())),
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
            global_metrics: Arc::new(RwLock::new(GlobalPoolMetrics::default())),
            request_router: Arc::new(RequestRouter::new(RoutingStrategy::PerformanceBased)),
            health_monitor: Arc::new(HealthMonitor::new(Duration::from_secs(60))),
            background_tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Register a provider with the pool manager
    #[instrument(skip(self, provider_config))]
    pub async fn register_provider(&self, provider_config: ProviderConnectionConfig) -> Result<()> {
        let provider_name = provider_config.name.clone();
        info!("Registering provider '{}' with connection pool manager", provider_name);
        
        // Create connection pool for provider
        let pool = Arc::new(ConnectionPool::new(self.config.clone()));
        pool.register_provider(provider_config.clone()).await?;
        
        // Store pool
        {
            let mut pools = self.pools.write().await;
            pools.insert(provider_name.clone(), pool);
        }
        
        // Store provider config
        {
            let mut configs = self.provider_configs.write().await;
            configs.insert(provider_name.clone(), provider_config.clone());
        }
        
        // Create rate limiter for provider
        let rate_limiter = Arc::new(RateLimiter::new(
            provider_config.rate_limit.max_requests_per_second,
            provider_config.rate_limit.burst_capacity,
        ));
        
        {
            let mut rate_limiters = self.rate_limiters.write().await;
            rate_limiters.insert(provider_name.clone(), rate_limiter);
        }
        
        // Initialize provider in request router
        self.request_router.add_provider(&provider_name, 1.0).await;
        
        // Initialize health monitoring
        self.health_monitor.add_provider(&provider_name).await;
        
        info!("Successfully registered provider '{}'", provider_name);
        Ok(())
    }
    
    /// Get optimal provider for a request
    #[instrument(skip(self, request))]
    pub async fn route_request(&self, request: &UniversalAIRequest) -> Result<String> {
        // Check if request specifies a particular provider
        if let Some(preferred_provider) = request.parameters.get("provider") {
            if let Some(provider_name) = preferred_provider.as_str() {
                if self.is_provider_healthy(provider_name).await {
                    return Ok(provider_name.to_string());
                } else {
                    warn!("Preferred provider '{}' is unhealthy, routing to alternative", provider_name);
                }
            }
        }
        
        // Use router to select optimal provider
        self.request_router.route_request().await
    }
    
    /// Execute a request using the optimal provider
    #[instrument(skip(self, request))]
    pub async fn execute_request(&self, request: UniversalAIRequest) -> Result<crate::enhanced::coordinator::UniversalAIResponse> {
        let provider_name = self.route_request(&request).await?;
        
        // Acquire rate limit token
        {
            let rate_limiters = self.rate_limiters.read().await;
            if let Some(rate_limiter) = rate_limiters.get(&provider_name) {
                rate_limiter.acquire().await?;
            }
        }
        
        // Get connection pool for provider
        let pool = {
            let pools = self.pools.read().await;
            pools.get(&provider_name)
                .ok_or_else(|| MCPError::Configuration(
                    format!("No pool found for provider '{}'", provider_name)
                ))?
                .clone()
        };
        
        // Get pooled client
        let client = pool.get_client(&provider_name).await?;
        
        // Execute request (delegate to AI provider implementation)
        self.execute_with_client(&request, &client).await
    }
    
    /// Execute request with a specific pooled client
    async fn execute_with_client(
        &self, 
        request: &UniversalAIRequest, 
        client: &super::PooledClient
    ) -> Result<crate::enhanced::coordinator::UniversalAIResponse> {
        let start_time = Instant::now();
        
        // Create pooled request
        let pooled_request = PooledRequest::post(
            client.provider_name.clone(),
            self.build_api_url(&client.provider_name, request).await?,
            self.build_request_body(request).await?,
        )
        .with_header("Content-Type".to_string(), "application/json".to_string())
        .with_header("Authorization".to_string(), self.build_auth_header(&client.provider_name).await?);
        
        // Execute HTTP request
        let request_builder = pooled_request.to_request_builder(&client.client);
        let response = request_builder.send().await.map_err(|e| {
            MCPError::ProviderError(format!("Request failed: {}", e))
        })?;
        
        let request_duration = start_time.elapsed();
        let status = response.status();
        
        // Update metrics
        self.update_request_metrics(&client.provider_name, request_duration, status.is_success()).await;
        
        // Handle response
        if status.is_success() {
            let response_body = response.text().await.map_err(|e| {
                MCPError::ProviderError(format!("Failed to read response body: {}", e))
            })?;
            
            self.parse_ai_response(&client.provider_name, request, &response_body, request_duration).await
        } else {
            Err(MCPError::ProviderError(
                format!("Provider returned error status: {}", status)
            ))
        }
    }
    
    /// Build API URL for provider request
    async fn build_api_url(&self, provider_name: &str, request: &UniversalAIRequest) -> Result<String> {
        let provider_configs = self.provider_configs.read().await;
        let config = provider_configs.get(provider_name)
            .ok_or_else(|| MCPError::Configuration(
                format!("Provider config not found: {}", provider_name)
            ))?;
        
        // Build provider-specific URL
        let endpoint = match provider_name {
            "openai" => "/v1/chat/completions",
            "anthropic" => "/v1/messages", 
            "gemini" => "/v1beta/models/gemini-pro:generateContent",
            _ => "/api/generate", // Generic endpoint
        };
        
        Ok(format!("{}{}", config.base_url.trim_end_matches('/'), endpoint))
    }
    
    /// Build request body for AI provider
    async fn build_request_body(&self, request: &UniversalAIRequest) -> Result<Vec<u8>> {
        let body = serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "max_tokens": request.parameters.get("max_tokens").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(1000))),
            "temperature": request.parameters.get("temperature").unwrap_or(&serde_json::Value::Number(serde_json::Number::from_f64(0.7).expect("0.7 is a valid f64"))),
        });
        
        serde_json::to_vec(&body).map_err(|e| {
            MCPError::Internal(format!("Failed to serialize request body: {}", e))
        })
    }
    
    /// Build authorization header for provider
    async fn build_auth_header(&self, provider_name: &str) -> Result<String> {
        // This would typically get API keys from secure configuration
        // For now, return a placeholder
        Ok(format!("Bearer {}_api_key_placeholder", provider_name))
    }
    
    /// Parse AI provider response
    async fn parse_ai_response(
        &self,
        provider_name: &str,
        request: &UniversalAIRequest,
        response_body: &str,
        duration: Duration,
    ) -> Result<crate::enhanced::coordinator::UniversalAIResponse> {
        // Parse provider-specific response format
        let parsed_response: serde_json::Value = serde_json::from_str(response_body)
            .map_err(|e| MCPError::ProviderError(format!("Invalid JSON response: {}", e)))?;
        
        // Extract content based on provider format
        let content = match provider_name {
            "openai" => {
                parsed_response
                    .get("choices")
                    .and_then(|c| c.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|choice| choice.get("message"))
                    .and_then(|msg| msg.get("content"))
                    .and_then(|content| content.as_str())
                    .unwrap_or("No response content")
                    .to_string()
            }
            "anthropic" => {
                parsed_response
                    .get("content")
                    .and_then(|c| c.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|item| item.get("text"))
                    .and_then(|text| text.as_str())
                    .unwrap_or("No response content")
                    .to_string()
            }
            _ => response_body.to_string(), // Return raw response for unknown providers
        };
        
        Ok(crate::enhanced::coordinator::UniversalAIResponse {
            id: uuid::Uuid::new_v4().to_string(),
            provider: provider_name.to_string(),
            model: request.model.clone(),
            response_type: request.request_type.clone(),
            content,
            cost: 0.001, // Placeholder cost calculation
            duration,
            metadata: HashMap::new(),
        })
    }
    
    /// Update request metrics
    async fn update_request_metrics(&self, provider_name: &str, duration: Duration, success: bool) {
        // Update global metrics
        {
            let mut global_metrics = self.global_metrics.write().await;
            global_metrics.total_requests += 1;
            
            if success {
                // Update success rate (exponential moving average)
                global_metrics.global_success_rate = 
                    (global_metrics.global_success_rate * 0.9) + (1.0 * 0.1);
            } else {
                global_metrics.global_success_rate = 
                    (global_metrics.global_success_rate * 0.9) + (0.0 * 0.1);
            }
            
            // Update average response time
            let response_time_ms = duration.as_millis() as f64;
            global_metrics.global_avg_response_time_ms = 
                (global_metrics.global_avg_response_time_ms * 0.9) + (response_time_ms * 0.1);
            
            global_metrics.last_updated = Instant::now();
        }
        
        // Update router performance metrics
        self.request_router.update_performance(provider_name, duration, success).await;
    }
    
    /// Check if a provider is healthy
    async fn is_provider_healthy(&self, provider_name: &str) -> bool {
        self.health_monitor.is_provider_healthy(provider_name).await
    }
    
    /// Get global metrics
    pub async fn get_global_metrics(&self) -> GlobalPoolMetrics {
        let mut metrics = self.global_metrics.read().await.clone();
        
        // Update pool counts
        let pools = self.pools.read().await;
        metrics.total_pools = pools.len();
        
        // Calculate total connections
        let mut total_connections = 0;
        let mut total_active = 0;
        
        for pool in pools.values() {
            let pool_metrics = pool.get_metrics().await;
            total_connections += pool_metrics.active_connections as usize;
            // Note: We'd need to add total_connections to ConnectionPoolMetrics
        }
        
        metrics.total_connections = total_connections;
        metrics.total_active_connections = total_active;
        
        metrics
    }
    
    /// Start background maintenance tasks
    pub async fn start_background_tasks(&self) -> Result<()> {
        let mut handles = self.background_tasks.lock().await;
        
        // Start health monitoring
        {
            let health_monitor = Arc::clone(&self.health_monitor);
            let pools = Arc::clone(&self.pools);
            let handle = tokio::spawn(async move {
                health_monitor.start_monitoring(pools).await;
            });
            handles.push(handle);
        }
        
        // Start metrics collection
        {
            let global_metrics = Arc::clone(&self.global_metrics);
            let pools = Arc::clone(&self.pools);
            let handle = tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(30));
                loop {
                    interval.tick().await;
                    
                    let mut metrics = global_metrics.write().await;
                    metrics.last_updated = Instant::now();
                    
                    // Collect metrics from all pools
                    let pools_guard = pools.read().await;
                    metrics.total_pools = pools_guard.len();
                    
                    drop(pools_guard);
                    drop(metrics);
                }
            });
            handles.push(handle);
        }
        
        // Start individual pool background tasks
        {
            let pools = self.pools.read().await;
            for pool in pools.values() {
                pool.start_background_tasks().await?;
            }
        }
        
        info!("Started {} background tasks for connection pool manager", handles.len());
        Ok(())
    }
    
    /// Shutdown the pool manager
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down connection pool manager");
        
        // Cancel all background tasks
        let mut handles = self.background_tasks.lock().await;
        for handle in handles.drain(..) {
            handle.abort();
        }
        
        Ok(())
    }
}

impl RequestRouter {
    /// Create a new request router
    pub fn new(strategy: RoutingStrategy) -> Self {
        Self {
            strategy,
            provider_weights: Arc::new(RwLock::new(HashMap::new())),
            round_robin_counter: Arc::new(Mutex::new(0)),
            performance_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Add a provider to the router
    pub async fn add_provider(&self, provider_name: &str, weight: f64) {
        let mut weights = self.provider_weights.write().await;
        weights.insert(provider_name.to_string(), weight);
        
        let mut performance = self.performance_history.write().await;
        performance.insert(provider_name.to_string(), PerformanceMetrics::default());
    }
    
    /// Route a request to the optimal provider
    pub async fn route_request(&self) -> Result<String> {
        let providers: Vec<String> = {
            let weights = self.provider_weights.read().await;
            weights.keys().cloned().collect()
        };
        
        if providers.is_empty() {
            return Err(MCPError::Configuration("No providers available for routing".to_string()));
        }
        
        match &self.strategy {
            RoutingStrategy::RoundRobin => {
                let mut counter = self.round_robin_counter.lock().await;
                let provider = &providers[*counter % providers.len()];
                *counter += 1;
                Ok(provider.clone())
            }
            
            RoutingStrategy::PerformanceBased => {
                self.route_by_performance(&providers).await
            }
            
            RoutingStrategy::Direct(provider_name) => {
                Ok(provider_name.clone())
            }
            
            _ => {
                // Default to round-robin for unimplemented strategies
                let mut counter = self.round_robin_counter.lock().await;
                let provider = &providers[*counter % providers.len()];
                *counter += 1;
                Ok(provider.clone())
            }
        }
    }
    
    /// Route based on provider performance
    async fn route_by_performance(&self, providers: &[String]) -> Result<String> {
        let performance_history = self.performance_history.read().await;
        
        // Calculate performance score for each provider
        let mut best_provider = providers[0].clone();
        let mut best_score = 0.0;
        
        for provider in providers {
            if let Some(metrics) = performance_history.get(provider) {
                let score = metrics.health_score * (1.0 - metrics.load) * metrics.success_rate;
                if score > best_score {
                    best_score = score;
                    best_provider = provider.clone();
                }
            }
        }
        
        Ok(best_provider)
    }
    
    /// Update provider performance metrics
    pub async fn update_performance(&self, provider_name: &str, duration: Duration, success: bool) {
        let mut performance_history = self.performance_history.write().await;
        if let Some(metrics) = performance_history.get_mut(provider_name) {
            // Update response time (exponential moving average)
            let response_time_ms = duration.as_millis() as f64;
            let current_avg = metrics.avg_response_time.as_millis() as f64;
            let new_avg = (current_avg * 0.8) + (response_time_ms * 0.2);
            metrics.avg_response_time = Duration::from_millis(new_avg as u64);
            
            // Update success rate
            metrics.success_rate = (metrics.success_rate * 0.8) + (if success { 1.0 } else { 0.0 } * 0.2);
            
            // Update health score based on success rate and response time
            let response_score = if new_avg < 1000.0 { 1.0 } else { 1000.0 / new_avg };
            metrics.health_score = (metrics.success_rate * 0.7) + (response_score * 0.3);
            
            metrics.last_updated = Instant::now();
        }
    }
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(check_interval: Duration) -> Self {
        Self {
            check_interval,
            provider_health: Arc::new(RwLock::new(HashMap::new())),
            quarantine: Arc::new(RwLock::new(HashMap::new())),
            quarantine_duration: Duration::from_secs(300), // 5 minutes
        }
    }
    
    /// Add a provider to health monitoring
    pub async fn add_provider(&self, provider_name: &str) {
        let mut health = self.provider_health.write().await;
        health.insert(provider_name.to_string(), ProviderHealthStatus {
            is_healthy: true,
            consecutive_failures: 0,
            last_success: Some(Instant::now()),
            last_error: None,
            response_time: Duration::from_millis(0),
            trend: HealthTrend::Unknown,
        });
    }
    
    /// Check if a provider is healthy
    pub async fn is_provider_healthy(&self, provider_name: &str) -> bool {
        // Check if provider is quarantined
        {
            let quarantine = self.quarantine.read().await;
            if let Some(quarantine_time) = quarantine.get(provider_name) {
                if quarantine_time.elapsed() < self.quarantine_duration {
                    return false; // Still quarantined
                }
            }
        }
        
        // Check health status
        let health = self.provider_health.read().await;
        health.get(provider_name)
            .map(|status| status.is_healthy)
            .unwrap_or(false)
    }
    
    /// Start health monitoring background task
    pub async fn start_monitoring(&self, pools: Arc<RwLock<HashMap<String, Arc<ConnectionPool>>>>) {
        let provider_health = Arc::clone(&self.provider_health);
        let quarantine = Arc::clone(&self.quarantine);
        let check_interval = self.check_interval;
        let quarantine_duration = self.quarantine_duration;
        
        let mut interval = tokio::time::interval(check_interval);
        
        loop {
            interval.tick().await;
            
            let pools_guard = pools.read().await;
            for (provider_name, pool) in pools_guard.iter() {
                match pool.health_check().await {
                    Ok(health_results) => {
                        if let Some(health) = health_results.get(provider_name) {
                            // Update health status
                            let mut provider_health_guard = provider_health.write().await;
                            if let Some(status) = provider_health_guard.get_mut(provider_name) {
                                status.is_healthy = health.is_healthy;
                                status.response_time = health.avg_response_time;
                                
                                if health.is_healthy {
                                    status.consecutive_failures = 0;
                                    status.last_success = Some(Instant::now());
                                    status.last_error = None;
                                    
                                    // Remove from quarantine if healthy
                                    let mut quarantine_guard = quarantine.write().await;
                                    quarantine_guard.remove(provider_name);
                                } else {
                                    status.consecutive_failures += 1;
                                    status.last_error = health.last_error.clone();
                                    
                                    // Quarantine if too many failures
                                    if status.consecutive_failures >= 3 {
                                        let mut quarantine_guard = quarantine.write().await;
                                        quarantine_guard.insert(provider_name.clone(), Instant::now());
                                        warn!("Provider '{}' quarantined due to health failures", provider_name);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Health check failed for provider '{}': {}", provider_name, e);
                    }
                }
            }
            
            drop(pools_guard);
        }
    }
} 