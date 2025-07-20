//! HTTP Connection Pool for AI Provider Clients
//!
//! This module provides an efficient connection pool manager for HTTP clients
//! connecting to various AI providers (OpenAI, Anthropic, etc.), reducing
//! connection overhead and improving performance for high-throughput scenarios.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{info, warn, error, debug, instrument};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use uuid::Uuid;

use crate::error::{Result, types::MCPError};

pub mod types;
pub mod manager;
pub mod metrics;

pub use types::*;
pub use manager::*;
pub use metrics::*;

#[cfg(test)]
mod tests;

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections per provider
    pub max_connections_per_provider: usize,
    
    /// Maximum idle connections per provider
    pub max_idle_connections: usize,
    
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    
    /// Idle connection timeout in seconds
    pub idle_timeout_seconds: u64,
    
    /// Keep-alive timeout in seconds
    pub keep_alive_timeout_seconds: u64,
    
    /// Maximum retries for failed requests
    pub max_retries: u32,
    
    /// Connection health check interval in seconds
    pub health_check_interval_seconds: u64,
    
    /// Enable HTTP/2
    pub enable_http2: bool,
    
    /// TCP keep-alive settings
    pub tcp_keep_alive: bool,
    
    /// User agent string
    pub user_agent: String,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections_per_provider: 20,
            max_idle_connections: 10,
            connection_timeout_ms: 30000,
            request_timeout_ms: 60000,
            idle_timeout_seconds: 300,
            keep_alive_timeout_seconds: 90,
            max_retries: 3,
            health_check_interval_seconds: 60,
            enable_http2: true,
            tcp_keep_alive: true,
            user_agent: "MCP-AI-Client/1.0".to_string(),
        }
    }
}

/// Provider-specific connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConnectionConfig {
    /// Provider name
    pub name: String,
    
    /// Base URL for API endpoints
    pub base_url: String,
    
    /// Provider-specific headers
    pub headers: HashMap<String, String>,
    
    /// Custom timeout overrides
    pub connection_timeout_ms: Option<u64>,
    pub request_timeout_ms: Option<u64>,
    
    /// TLS configuration
    pub tls_config: TlsConfig,
    
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
}

/// TLS configuration for secure connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Verify TLS certificates
    pub verify_certificates: bool,
    
    /// Custom CA certificate path
    pub ca_cert_path: Option<String>,
    
    /// Client certificate path (for mutual TLS)
    pub client_cert_path: Option<String>,
    
    /// Client key path (for mutual TLS)
    pub client_key_path: Option<String>,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            verify_certificates: true,
            ca_cert_path: None,
            client_cert_path: None,
            client_key_path: None,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per second
    pub max_requests_per_second: f64,
    
    /// Burst capacity
    pub burst_capacity: u32,
    
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests_per_second: 10.0,
            burst_capacity: 20,
            retry_delay_ms: 1000,
        }
    }
}

/// Connection pool for AI provider HTTP clients
#[derive(Debug)]
pub struct ConnectionPool {
    /// Pool configuration
    config: ConnectionPoolConfig,
    
    /// Provider-specific configurations
    provider_configs: Arc<RwLock<HashMap<String, ProviderConnectionConfig>>>,
    
    /// HTTP clients by provider
    clients: Arc<RwLock<HashMap<String, PooledClient>>>,
    
    /// Connection semaphores for concurrency control
    semaphores: Arc<RwLock<HashMap<String, Arc<Semaphore>>>>,
    
    /// Connection metrics
    metrics: Arc<RwLock<ConnectionPoolMetrics>>,
    
    /// Health monitoring
    health_monitor: Arc<RwLock<HashMap<String, ProviderHealth>>>,
}

/// Pooled HTTP client with connection management
#[derive(Debug, Clone)]
pub struct PooledClient {
    /// Underlying reqwest client
    pub client: Client,
    
    /// Provider name
    pub provider_name: String,
    
    /// Creation timestamp
    pub created_at: Instant,
    
    /// Last used timestamp
    pub last_used: Arc<RwLock<Instant>>,
    
    /// Connection statistics
    pub stats: Arc<RwLock<ClientStats>>,
}

/// Client connection statistics
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ClientStats {
    /// Total requests made
    pub total_requests: u64,
    
    /// Successful requests
    pub successful_requests: u64,
    
    /// Failed requests
    pub failed_requests: u64,
    
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    
    /// Total bytes sent
    pub bytes_sent: u64,
    
    /// Total bytes received
    pub bytes_received: u64,
}

/// Provider health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    /// Is provider healthy?
    pub is_healthy: bool,
    
    /// Last health check timestamp
    pub last_check: Instant,
    
    /// Average response time
    pub avg_response_time: Duration,
    
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    
    /// Current error rate
    pub error_rate: f64,
    
    /// Last error message
    pub last_error: Option<String>,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(config: ConnectionPoolConfig) -> Self {
        Self {
            config,
            provider_configs: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
            semaphores: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(ConnectionPoolMetrics::new())),
            health_monitor: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a provider with the connection pool
    #[instrument(skip(self, provider_config))]
    pub async fn register_provider(&self, provider_config: ProviderConnectionConfig) -> Result<()> {
        let provider_name = provider_config.name.clone();
        info!("Registering provider '{}' with connection pool", provider_name);
        
        // Validate configuration
        self.validate_provider_config(&provider_config)?;
        
        // Create HTTP client for provider
        let client = self.create_client_for_provider(&provider_config).await?;
        
        // Store provider configuration
        {
            let mut configs = self.provider_configs.write().await;
            configs.insert(provider_name.clone(), provider_config);
        }
        
        // Create pooled client
        let pooled_client = PooledClient {
            client,
            provider_name: provider_name.clone(),
            created_at: Instant::now(),
            last_used: Arc::new(RwLock::new(Instant::now())),
            stats: Arc::new(RwLock::new(ClientStats::default())),
        };
        
        // Store pooled client
        {
            let mut clients = self.clients.write().await;
            clients.insert(provider_name.clone(), pooled_client);
        }
        
        // Create semaphore for concurrency control
        {
            let mut semaphores = self.semaphores.write().await;
            semaphores.insert(
                provider_name.clone(),
                Arc::new(Semaphore::new(self.config.max_connections_per_provider))
            );
        }
        
        // Initialize health monitoring
        {
            let mut health_monitor = self.health_monitor.write().await;
            health_monitor.insert(provider_name.clone(), ProviderHealth {
                is_healthy: true,
                last_check: Instant::now(),
                avg_response_time: Duration::from_millis(0),
                success_rate: 1.0,
                error_rate: 0.0,
                last_error: None,
            });
        }
        
        Ok(())
    }
    
    /// Get a client for the specified provider
    #[instrument(skip(self))]
    pub async fn get_client(&self, provider_name: &str) -> Result<PooledClient> {
        // Check if provider exists
        let clients = self.clients.read().await;
        let client = clients.get(provider_name)
            .ok_or_else(|| MCPError::Configuration(
                format!("Provider '{}' not registered in connection pool", provider_name)
            ))?
            .clone();
        drop(clients);
        
        // Update last used timestamp
        {
            let mut last_used = client.last_used.write().await;
            *last_used = Instant::now();
        }
        
        // Acquire semaphore permit for concurrency control
        let semaphores = self.semaphores.read().await;
        let semaphore = semaphores.get(provider_name)
            .ok_or_else(|| MCPError::Internal("Semaphore not found".to_string()))?
            .clone();
        drop(semaphores);
        
        let _permit = semaphore.acquire().await.map_err(|e| {
            MCPError::Internal(format!("Failed to acquire connection permit: {}", e))
        })?;
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.active_connections += 1;
            metrics.total_requests += 1;
        }
        
        Ok(client)
    }
    
    /// Create HTTP client for a provider
    async fn create_client_for_provider(&self, provider_config: &ProviderConnectionConfig) -> Result<Client> {
        let mut client_builder = Client::builder()
            .user_agent(&self.config.user_agent)
            .timeout(Duration::from_millis(
                provider_config.request_timeout_ms
                    .unwrap_or(self.config.request_timeout_ms)
            ))
            .connect_timeout(Duration::from_millis(
                provider_config.connection_timeout_ms
                    .unwrap_or(self.config.connection_timeout_ms)
            ));
        
        // Configure HTTP/2
        if self.config.enable_http2 {
            client_builder = client_builder.http2_prior_knowledge();
        }
        
        // Configure TCP keep-alive
        if self.config.tcp_keep_alive {
            client_builder = client_builder.tcp_keepalive(Duration::from_secs(
                self.config.keep_alive_timeout_seconds
            ));
        }
        
        // Configure TLS
        if !provider_config.tls_config.verify_certificates {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }
        
        // Build client
        let client = client_builder.build().map_err(|e| {
            MCPError::Configuration(format!("Failed to create HTTP client: {}", e))
        })?;
        
        Ok(client)
    }
    
    /// Validate provider configuration
    fn validate_provider_config(&self, config: &ProviderConnectionConfig) -> Result<()> {
        if config.name.is_empty() {
            return Err(MCPError::Configuration("Provider name cannot be empty".to_string()));
        }
        
        if config.base_url.is_empty() {
            return Err(MCPError::Configuration("Base URL cannot be empty".to_string()));
        }
        
        // Validate URL format
        if let Err(e) = url::Url::parse(&config.base_url) {
            return Err(MCPError::Configuration(
                format!("Invalid base URL '{}': {}", config.base_url, e)
            ));
        }
        
        Ok(())
    }
    
    /// Perform health checks on all registered providers
    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Result<HashMap<String, ProviderHealth>> {
        let mut health_results = HashMap::new();
        
        let clients = self.clients.read().await;
        for (provider_name, pooled_client) in clients.iter() {
            let health = self.check_provider_health(provider_name, pooled_client).await?;
            health_results.insert(provider_name.clone(), health);
        }
        
        Ok(health_results)
    }
    
    /// Check health of a specific provider
    async fn check_provider_health(&self, provider_name: &str, client: &PooledClient) -> Result<ProviderHealth> {
        let start_time = Instant::now();
        
        // Get provider configuration
        let provider_configs = self.provider_configs.read().await;
        let provider_config = provider_configs.get(provider_name)
            .ok_or_else(|| MCPError::Internal("Provider config not found".to_string()))?;
        
        // Perform simple HTTP health check (HEAD request to base URL)
        let health_check_url = format!("{}/health", provider_config.base_url.trim_end_matches('/'));
        let response_result = client.client
            .head(&health_check_url)
            .timeout(Duration::from_secs(10))
            .send()
            .await;
        
        let response_time = start_time.elapsed();
        let is_healthy = response_result.is_ok();
        
        let mut health = ProviderHealth {
            is_healthy,
            last_check: Instant::now(),
            avg_response_time: response_time,
            success_rate: if is_healthy { 1.0 } else { 0.0 },
            error_rate: if is_healthy { 0.0 } else { 1.0 },
            last_error: if let Err(ref e) = response_result {
                Some(e.to_string())
            } else {
                None
            },
        };
        
        // Update stored health status
        {
            let mut health_monitor = self.health_monitor.write().await;
            if let Some(existing_health) = health_monitor.get(provider_name) {
                // Calculate rolling averages
                health.avg_response_time = Duration::from_millis(
                    ((existing_health.avg_response_time.as_millis() as f64 * 0.8) +
                     (response_time.as_millis() as f64 * 0.2)) as u64
                );
                
                health.success_rate = (existing_health.success_rate * 0.8) + 
                                    (if is_healthy { 1.0 } else { 0.0 } * 0.2);
                
                health.error_rate = (existing_health.error_rate * 0.8) + 
                                  (if is_healthy { 0.0 } else { 1.0 } * 0.2);
            }
            
            health_monitor.insert(provider_name.to_string(), health.clone());
        }
        
        Ok(health)
    }
    
    /// Get connection pool metrics
    pub async fn get_metrics(&self) -> ConnectionPoolMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Clean up idle connections
    #[instrument(skip(self))]
    pub async fn cleanup_idle_connections(&self) -> Result<usize> {
        let mut cleaned_up = 0;
        let idle_timeout = Duration::from_secs(self.config.idle_timeout_seconds);
        let now = Instant::now();
        
        let clients = self.clients.read().await;
        for (provider_name, client) in clients.iter() {
            let last_used = *client.last_used.read().await;
            if now.duration_since(last_used) > idle_timeout {
                info!("Cleaning up idle connection for provider '{}'", provider_name);
                cleaned_up += 1;
            }
        }
        
        if cleaned_up > 0 {
            let mut metrics = self.metrics.write().await;
            metrics.connections_cleaned_up += cleaned_up;
        }
        
        Ok(cleaned_up)
    }
    
    /// Get provider health status
    pub async fn get_provider_health(&self, provider_name: &str) -> Option<ProviderHealth> {
        let health_monitor = self.health_monitor.read().await;
        health_monitor.get(provider_name).cloned()
    }
    
    /// List all registered providers
    pub async fn list_providers(&self) -> Vec<String> {
        let clients = self.clients.read().await;
        clients.keys().cloned().collect()
    }
    
    /// Start background tasks for pool maintenance
    pub async fn start_background_tasks(&self) -> Result<()> {
        let pool = Arc::new(self.clone());
        
        // Start health check task
        {
            let pool_clone = Arc::clone(&pool);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(
                    pool_clone.config.health_check_interval_seconds
                ));
                
                loop {
                    interval.tick().await;
                    if let Err(e) = pool_clone.health_check().await {
                        error!("Health check failed: {}", e);
                    }
                }
            });
        }
        
        // Start cleanup task
        {
            let pool_clone = Arc::clone(&pool);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(300)); // Clean up every 5 minutes
                
                loop {
                    interval.tick().await;
                    if let Ok(cleaned) = pool_clone.cleanup_idle_connections().await {
                        if cleaned > 0 {
                            debug!("Cleaned up {} idle connections", cleaned);
                        }
                    }
                }
            });
        }
        
        Ok(())
    }
}

// Make ConnectionPool cloneable for use in background tasks
impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            provider_configs: Arc::clone(&self.provider_configs),
            clients: Arc::clone(&self.clients),
            semaphores: Arc::clone(&self.semaphores),
            metrics: Arc::clone(&self.metrics),
            health_monitor: Arc::clone(&self.health_monitor),
        }
    }
} 