//! Toadstool client implementation

use crate::{
    errors::{ToadstoolError, ToadstoolResult},
    ExecutionEnvironment, ExecutionInfo, ExecutionResult, ExecutionStatus, PluginExecutor,
    ToadstoolConfig,
};
use async_trait::async_trait;
use base64::Engine;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tracing::{debug, error, info};
use uuid::Uuid;

/// Client for communicating with Toadstool compute platform
#[derive(Debug)]
pub struct ToadstoolClient {
    config: ToadstoolConfig,
    http_client: Client,
    endpoint: String,
}

impl ToadstoolClient {
    /// Create a new Toadstool client
    pub async fn new(config: ToadstoolConfig) -> ToadstoolResult<Self> {
        let timeout = Duration::from_millis(config.timeout);
        let http_client = Client::builder().timeout(timeout).build().map_err(|e| {
            ToadstoolError::configuration(format!("Failed to create HTTP client: {e}"))
        })?;

        let endpoint = config.endpoint.clone();

        let client = Self {
            config,
            http_client,
            endpoint: endpoint.clone(),
        };

        // Test connection
        client.health_check().await?;

        info!("Connected to Toadstool at {}", endpoint);
        Ok(client)
    }

    /// Check if Toadstool service is healthy
    pub async fn health_check(&self) -> ToadstoolResult<()> {
        let url = format!("{}/health", self.endpoint);

        debug!("Checking Toadstool health at {}", url);

        let response = self.http_client.get(&url).send().await?;

        if response.status().is_success() {
            debug!("Toadstool health check passed");
            Ok(())
        } else {
            Err(ToadstoolError::connection(format!(
                "Health check failed with status: {}",
                response.status()
            )))
        }
    }

    /// Get Toadstool service information
    pub async fn get_service_info(&self) -> ToadstoolResult<serde_json::Value> {
        let url = format!("{}/info", self.endpoint);

        let response = self.http_client.get(&url).send().await?;

        if response.status().is_success() {
            let info = response.json().await?;
            Ok(info)
        } else {
            Err(ToadstoolError::connection(format!(
                "Failed to get service info: {}",
                response.status()
            )))
        }
    }

    /// Create authenticated request builder
    fn authenticated_request(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        let mut request = self.http_client.request(method, url);

        if let Some(ref token) = self.config.auth_token {
            request = request.bearer_auth(token);
        }

        request.header("Content-Type", "application/json")
    }
}

#[async_trait]
impl PluginExecutor for ToadstoolClient {
    async fn execute_plugin(
        &self,
        plugin_id: &str,
        code: &[u8],
        environment: ExecutionEnvironment,
    ) -> ToadstoolResult<ExecutionResult> {
        let execution_id = Uuid::new_v4();
        let url = format!("{}/v1/execute", self.endpoint);

        debug!(
            "Executing plugin {} with execution ID {}",
            plugin_id, execution_id
        );

        // Encode code as base64 for JSON transport
        let code_b64 = base64::engine::general_purpose::STANDARD.encode(code);

        let request_body = json!({
            "execution_id": execution_id,
            "plugin_id": plugin_id,
            "code": code_b64,
            "environment": environment
        });

        let response = self
            .authenticated_request(reqwest::Method::POST, &url)
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let result: ExecutionResult = response.json().await?;
            info!("Plugin {} execution completed successfully", plugin_id);
            Ok(result)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Plugin {} execution failed: {}", plugin_id, error_text);
            Err(ToadstoolError::execution(error_text))
        }
    }

    async fn get_execution_status(&self, execution_id: &Uuid) -> ToadstoolResult<ExecutionStatus> {
        let url = format!("{}/v1/executions/{}/status", self.endpoint, execution_id);

        let response = self
            .authenticated_request(reqwest::Method::GET, &url)
            .send()
            .await?;

        if response.status().is_success() {
            let status: ExecutionStatus = response.json().await?;
            Ok(status)
        } else if response.status() == 404 {
            Err(ToadstoolError::execution_not_found(
                execution_id.to_string(),
            ))
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(ToadstoolError::execution(error_text))
        }
    }

    async fn cancel_execution(&self, execution_id: &Uuid) -> ToadstoolResult<()> {
        let url = format!("{}/v1/executions/{}/cancel", self.endpoint, execution_id);

        debug!("Cancelling execution {}", execution_id);

        let response = self
            .authenticated_request(reqwest::Method::POST, &url)
            .send()
            .await?;

        if response.status().is_success() {
            info!("Execution {} cancelled successfully", execution_id);
            Ok(())
        } else if response.status() == 404 {
            Err(ToadstoolError::execution_not_found(
                execution_id.to_string(),
            ))
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(ToadstoolError::execution(error_text))
        }
    }

    async fn list_executions(&self) -> ToadstoolResult<Vec<ExecutionInfo>> {
        let url = format!("{}/v1/executions", self.endpoint);

        let response = self
            .authenticated_request(reqwest::Method::GET, &url)
            .send()
            .await?;

        if response.status().is_success() {
            let executions: Vec<ExecutionInfo> = response.json().await?;
            Ok(executions)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(ToadstoolError::execution(error_text))
        }
    }
}
