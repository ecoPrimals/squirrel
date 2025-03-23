use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use tokio::sync::RwLock;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use tracing::{info, warn, error, debug};
use urlencoding;

use squirrel_core::error::{Result, SquirrelError};

use crate::dashboard::{DashboardComponent, Update, DashboardError};

/// Configuration for the Prometheus metrics component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusConfig {
    /// URL of the Prometheus server
    pub url: String,
    /// Query interval in seconds
    #[serde(default = "default_interval")]
    pub interval: u64,
    /// Queries to execute
    pub queries: HashMap<String, String>,
    /// Optional basic auth credentials
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<PrometheusAuth>,
    /// Optional timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

/// Authentication configuration for Prometheus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrometheusAuth {
    /// Basic authentication with username and password
    Basic {
        username: String,
        password: Option<String>,
    },
    /// Bearer token authentication
    Bearer {
        token: String,
    },
}

/// Prometheus metrics component for the dashboard
#[derive(Debug)]
pub struct PrometheusMetrics {
    /// Component ID
    id: String,
    /// Component configuration
    config: PrometheusConfig,
    /// HTTP client
    client: Client,
    /// Cached metrics data
    data: Arc<RwLock<Option<Value>>>,
    /// Last update timestamp
    last_update: Arc<RwLock<Option<DateTime<Utc>>>>,
}

/// Prometheus query response format
#[derive(Debug, Deserialize)]
struct PrometheusResponse {
    status: String,
    data: PrometheusData,
}

#[derive(Debug, Deserialize)]
struct PrometheusData {
    result_type: String,
    result: Vec<PrometheusResult>,
}

#[derive(Debug, Deserialize)]
struct PrometheusResult {
    metric: HashMap<String, String>,
    value: Option<(u64, String)>,
    values: Option<Vec<(u64, String)>>,
}

// Default interval for queries: 60 seconds
fn default_interval() -> u64 {
    60
}

// Default timeout for queries: 10 seconds
fn default_timeout() -> u64 {
    10
}

impl PrometheusMetrics {
    /// Create a new Prometheus metrics component
    pub fn new(id: impl Into<String>, config: PrometheusConfig) -> Self {
        // Create client with configured timeout
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .build()
            .unwrap_or_default();
        
        Self {
            id: id.into(),
            config,
            client,
            data: Arc::new(RwLock::new(None)),
            last_update: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Query Prometheus server with the given query
    async fn query_prometheus(&self, query: &str) -> Result<Value> {
        let mut url = format!("{}/api/v1/query", self.config.url.trim_end_matches('/'));
        
        // Add query parameter
        url = format!("{}?query={}", url, urlencoding::encode(query));
        
        let mut request = self.client.get(&url);
        
        // Add authentication if provided
        if let Some(auth) = &self.config.auth {
            match auth {
                PrometheusAuth::Basic { username, password } => {
                    request = request.basic_auth(username, password.as_ref());
                },
                PrometheusAuth::Bearer { token } => {
                    request = request.bearer_auth(token);
                },
            }
        }
        
        // Make the request
        let response = request.send().await
            .map_err(|e| SquirrelError::other(format!("Failed to query Prometheus: {}", e)))?;
        
        // Check status code
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| String::from("Could not get error response text"));
                
            return Err(SquirrelError::other(
                format!("Prometheus query failed with status {}: {}", status, text)
            ));
        }
        
        // Parse response
        let body = response.text().await
            .map_err(|e| SquirrelError::other(format!("Failed to read response body: {}", e)))?;
            
        // Parse JSON
        let json: Value = serde_json::from_str(&body)
            .map_err(|e| SquirrelError::other(format!("Failed to parse JSON response: {}", e)))?;
        
        // Check for Prometheus errors
        if json["status"] != "success" {
            let error_type = json["errorType"].as_str().unwrap_or("unknown");
            let error_message = json["error"].as_str().unwrap_or("No error message provided");
            
            return Err(SquirrelError::other(
                format!("Prometheus error ({}): {}", error_type, error_message)
            ));
        }
        
        Ok(json)
    }
    
    /// Query all configured Prometheus queries
    async fn query_all(&self) -> Result<Value> {
        let mut results = HashMap::new();
        
        for (name, query) in &self.config.queries {
            match self.query_prometheus(query).await {
                Ok(data) => {
                    results.insert(name.clone(), data);
                },
                Err(e) => {
                    warn!("Failed to query Prometheus metric '{}': {}", name, e);
                    results.insert(name.clone(), json!({
                        "error": format!("{}", e),
                        "query": query,
                    }));
                }
            }
        }
        
        Ok(json!(results))
    }
}

/// Format Prometheus results into a more dashboard-friendly format
fn format_prometheus_results(results: &[PrometheusResult]) -> Result<Value> {
    let mut formatted = Vec::with_capacity(results.len());
    
    for result in results {
        // Extract labels from metric
        let labels = result.metric.clone();
        
        // Extract values based on what's available
        let values = if let Some(value) = &result.value {
            // Single value (instant query)
            vec![parse_prometheus_value(value)?]
        } else if let Some(values) = &result.values {
            // Multiple values (range query)
            let mut parsed = Vec::with_capacity(values.len());
            for v in values {
                parsed.push(parse_prometheus_value(v)?);
            }
            parsed
        } else {
            // No values
            vec![]
        };
        
        formatted.push(json!({
            "labels": labels,
            "values": values
        }));
    }
    
    Ok(json!(formatted))
}

/// Parse a Prometheus value (timestamp, value string)
fn parse_prometheus_value(value: &(u64, String)) -> Result<Value> {
    let timestamp = value.0;
    let value_str = &value.1;
    
    // Try to parse as number first
    if let Ok(num) = value_str.parse::<f64>() {
        Ok(json!({
            "timestamp": timestamp,
            "value": num
        }))
    } else {
        // Fall back to string
        Ok(json!({
            "timestamp": timestamp,
            "value": value_str
        }))
    }
}

#[async_trait]
impl DashboardComponent for PrometheusMetrics {
    /// Get the component ID
    fn id(&self) -> &str {
        &self.id
    }
    
    /// Start the component
    async fn start(&self) -> Result<()> {
        info!("Starting Prometheus metrics component: {}", self.id);
        
        // Perform initial query
        match self.query_all().await {
            Ok(data) => {
                let mut data_lock = self.data.write().await;
                *data_lock = Some(data);
                
                let mut timestamp_lock = self.last_update.write().await;
                *timestamp_lock = Some(Utc::now());
                
                info!("Prometheus metrics component {} initialized successfully", self.id);
            },
            Err(e) => {
                error!("Failed to initialize Prometheus metrics component {}: {}", self.id, e);
                return Err(e);
            }
        }
        
        // Create background task for periodic updates
        let id = self.id.clone();
        let interval = self.config.interval;
        let data = self.data.clone();
        let last_update = self.last_update.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let component = PrometheusMetrics::new(&id, config);
            let mut interval_timer = tokio::time::interval(Duration::from_secs(interval));
            
            loop {
                interval_timer.tick().await;
                
                debug!("Updating Prometheus metrics for component {}", id);
                match component.query_all().await {
                    Ok(new_data) => {
                        let mut data_lock = data.write().await;
                        *data_lock = Some(new_data);
                        
                        let mut timestamp_lock = last_update.write().await;
                        *timestamp_lock = Some(Utc::now());
                        
                        debug!("Prometheus metrics for component {} updated successfully", id);
                    },
                    Err(e) => {
                        error!("Failed to update Prometheus metrics for component {}: {}", id, e);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Get the current data for the component
    async fn get_data(&self) -> Result<Value> {
        let data_lock = self.data.read().await;
        
        match &*data_lock {
            Some(data) => Ok(data.clone()),
            None => Err(DashboardError::ComponentError("No data available yet".to_string()).into()),
        }
    }
    
    /// Get the last update timestamp
    async fn last_update(&self) -> Option<DateTime<Utc>> {
        let guard = self.last_update.read().await;
        *guard
    }
    
    /// Get an update for the component
    async fn get_update(&self) -> Result<Update> {
        let data = self.get_data().await?;
        let timestamp = self.last_update().await.unwrap_or_else(Utc::now);
        
        // Convert from chrono::DateTime<Utc> to OffsetDateTime (used by the Update struct)
        let offset_time = time::OffsetDateTime::from_unix_timestamp(timestamp.timestamp())
            .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
        
        Ok(Update {
            component_id: self.id().to_string(),
            timestamp: offset_time,
            data,
        })
    }
} 