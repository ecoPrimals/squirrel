/*!
 * Galaxy API client.
 * 
 * This module provides the HTTP client for interacting with the Galaxy API.
 */

use std::collections::HashMap;
use std::time::Duration;
use reqwest::{Client, header};
use serde_json::Value;
use tracing::debug;

use crate::error::{Error, Result};
use crate::models::{
    GalaxyTool, 
    tool::{JobState, ToolOutput},
    ParameterValue
};
use crate::security::{SecureCredentials, SecretString};

/// Galaxy API client
#[derive(Debug)]
pub struct GalaxyClient {
    /// Base URL for the Galaxy API
    base_url: String,
    
    /// Secure credentials for authentication
    credentials: SecureCredentials,
    
    /// HTTP client for making requests
    client: Client,
    
    /// Whether the client is initialized
    initialized: bool,
}

impl GalaxyClient {
    /// Creates a new Galaxy client with secure credentials
    pub fn new(
        base_url: &str,
        credentials: SecureCredentials,
        timeout: Option<Duration>,
    ) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        
        // Add common headers
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("galaxy-mcp-adapter"),
        );
        
        // Add API key if available
        if let Some(api_key) = credentials.api_key() {
            headers.insert(
                header::HeaderName::from_static("x-api-key"),
                header::HeaderValue::from_str(api_key.expose())
                    .map_err(|e| Error::Config(format!("Invalid API key: {}", e)))?,
            );
        }
        
        // Create HTTP client
        let client = Client::builder()
            .default_headers(headers)
            .timeout(timeout.unwrap_or_else(|| Duration::from_secs(30)))
            .build()
            .map_err(|e| Error::Config(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self {
            base_url: base_url.to_string(),
            credentials,
            client,
            initialized: true,
        })
    }
    
    /// Creates a new Galaxy client with an API key
    pub fn with_api_key(
        base_url: &str,
        api_key: impl Into<String>,
        timeout: Option<Duration>,
    ) -> Result<Self> {
        Self::new(
            base_url,
            SecureCredentials::with_api_key(SecretString::new(api_key.into())),
            timeout,
        )
    }
    
    /// Creates a new Galaxy client with email and password
    pub fn with_email_password(
        base_url: &str,
        email: impl Into<String>,
        password: impl Into<String>,
        timeout: Option<Duration>,
    ) -> Result<Self> {
        Self::new(
            base_url,
            SecureCredentials::with_email_password(
                email.into(),
                SecretString::new(password.into()),
            ),
            timeout,
        )
    }
    
    /// Creates a mock client for testing
    #[cfg(test)]
    pub fn new_mock() -> Self {
        Self {
            base_url: "http://localhost:8080/api".to_string(),
            credentials: SecureCredentials::with_api_key(SecretString::new("mock-api-key")),
            client: Client::new(),
            initialized: true,
        }
    }
    
    /// Checks if the client is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Returns the base URL for the Galaxy API
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
    
    /// Returns a reference to the secure credentials
    pub fn credentials(&self) -> &SecureCredentials {
        &self.credentials
    }
    
    /// Updates the credentials
    pub fn update_credentials(&mut self, credentials: SecureCredentials) -> Result<()> {
        self.credentials = credentials;
        
        // Rebuild the client with new credentials
        let mut headers = header::HeaderMap::new();
        
        // Add common headers
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("galaxy-mcp-adapter"),
        );
        
        // Add API key if available
        if let Some(api_key) = self.credentials.api_key() {
            headers.insert(
                header::HeaderName::from_static("x-api-key"),
                header::HeaderValue::from_str(api_key.expose())
                    .map_err(|e| Error::Config(format!("Invalid API key: {}", e)))?,
            );
        }
        
        // Create HTTP client
        self.client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| Error::Config(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(())
    }
    
    /// Lists available Galaxy tools
    pub async fn list_tools(&self) -> Result<Vec<GalaxyTool>> {
        if !self.initialized {
            return Err(Error::NotInitialized);
        }
        
        debug!("Fetching tools from Galaxy API");
        
        let response = self.client
            .get(format!("{}/tools", self.base_url))
            .send()
            .await
            .map_err(Error::Network)?;
        
        if !response.status().is_success() {
            return Err(Error::GalaxyApi(format!(
                "Failed to list tools: HTTP {}",
                response.status()
            )));
        }
        
        let tools: Vec<GalaxyTool> = response
            .json()
            .await
            .map_err(|e| Error::NetworkResponseDecode(e.to_string()))?;
        
        Ok(tools)
    }
    
    /// Gets a specific Galaxy tool by ID
    pub async fn get_tool(&self, tool_id: &str) -> Result<GalaxyTool> {
        debug!("Fetching tool from Galaxy API: {}", tool_id);
        
        let response = self.client
            .get(format!("{}/tools/{}", self.base_url, tool_id))
            .send()
            .await
            .map_err(Error::Network)?;
        
        if !response.status().is_success() {
            return Err(Error::GalaxyApi(format!(
                "Failed to get tool {}: HTTP {}",
                tool_id, response.status()
            )));
        }
        
        let tool: GalaxyTool = response
            .json()
            .await
            .map_err(|e| Error::NetworkResponseDecode(e.to_string()))?;
        
        Ok(tool)
    }
    
    /// Executes a Galaxy tool with the specified parameters
    pub async fn execute_tool(
        &self,
        tool_id: &str,
        parameters: &HashMap<String, ParameterValue>,
    ) -> Result<String> {
        debug!("Executing tool: {}", tool_id);
        
        let mut body = HashMap::new();
        body.insert("tool_id", Value::String(tool_id.to_string()));
        body.insert("parameters", serde_json::to_value(parameters).map_err(Error::Json)?);
        
        let response = self.client
            .post(format!("{}/tools/{}/execute", self.base_url, tool_id))
            .json(&body)
            .send()
            .await
            .map_err(Error::Network)?;
        
        if !response.status().is_success() {
            return Err(Error::GalaxyApi(format!(
                "Failed to execute tool {}: HTTP {}",
                tool_id, response.status()
            )));
        }
        
        let result: Value = response
            .json()
            .await
            .map_err(|e| Error::NetworkResponseDecode(e.to_string()))?;
        
        let job_id = result
            .get("job_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::GalaxyApi("Missing job ID in response".to_string()))?
            .to_string();
        
        Ok(job_id)
    }
    
    /// Gets the status of a job
    pub async fn get_job_status(&self, job_id: &str) -> Result<JobState> {
        debug!("Getting job status: {}", job_id);
        
        let response = self.client
            .get(format!("{}/jobs/{}", self.base_url, job_id))
            .send()
            .await
            .map_err(Error::Network)?;
        
        if !response.status().is_success() {
            return Err(Error::GalaxyApi(format!(
                "Failed to get job status for {}: HTTP {}",
                job_id, response.status()
            )));
        }
        
        let result: Value = response
            .json()
            .await
            .map_err(|e| Error::NetworkResponseDecode(e.to_string()))?;
        
        let state = result
            .get("state")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::GalaxyApi("Missing state in job status".to_string()))?;
        
        match state {
            "new" | "queued" => Ok(JobState::Queued),
            "running" => Ok(JobState::Running),
            "ok" | "done" => Ok(JobState::Completed),
            "error" | "failed" => Ok(JobState::Failed),
            "deleted" => Ok(JobState::Deleted),
            "waiting" => Ok(JobState::Waiting),
            _ => Ok(JobState::Unknown),
        }
    }
    
    /// Gets the results of a completed job
    pub async fn get_job_results(&self, job_id: &str) -> Result<Vec<ToolOutput>> {
        debug!("Getting job results: {}", job_id);
        
        // First check if job is complete
        let status = self.get_job_status(job_id).await?;
        if status != JobState::Completed {
            return Err(Error::GalaxyApi(format!(
                "Cannot get results for job that is not complete: {} (status: {:?})",
                job_id, status
            )));
        }
        
        let response = self.client
            .get(format!("{}/jobs/{}/outputs", self.base_url, job_id))
            .send()
            .await
            .map_err(Error::Network)?;
        
        if !response.status().is_success() {
            return Err(Error::GalaxyApi(format!(
                "Failed to get job results for {}: HTTP {}",
                job_id, response.status()
            )));
        }
        
        let outputs: Vec<ToolOutput> = response
            .json()
            .await
            .map_err(|e| Error::NetworkResponseDecode(e.to_string()))?;
        
        Ok(outputs)
    }
    
    /// Downloads a dataset
    pub async fn download_dataset(&self, dataset_id: &str) -> Result<Vec<u8>> {
        debug!("Downloading dataset: {}", dataset_id);
        
        let response = self.client
            .get(format!("{}/datasets/{}/download", self.base_url, dataset_id))
            .send()
            .await
            .map_err(Error::Network)?;
        
        if !response.status().is_success() {
            return Err(Error::GalaxyApi(format!(
                "Failed to download dataset {}: HTTP {}",
                dataset_id, response.status()
            )));
        }
        
        let bytes = response
            .bytes()
            .await
            .map_err(Error::Network)?;
        
        Ok(bytes.to_vec())
    }
    
    /// Uploads a dataset to Galaxy
    pub async fn upload_dataset(
        &self,
        history_id: &str,
        name: &str,
        data: Vec<u8>,
        file_type: Option<&str>,
    ) -> Result<crate::models::dataset::Dataset> {
        debug!("Uploading dataset to history: {}", history_id);
        
        let form = reqwest::multipart::Form::new()
            .text("history_id", history_id.to_string())
            .text("name", name.to_string())
            .part(
                "file", 
                reqwest::multipart::Part::bytes(data)
                    .file_name(name.to_string())
            );
        
        let form = if let Some(ft) = file_type {
            form.text("file_type", ft.to_string())
        } else {
            form
        };
        
        let response = self.client
            .post(format!("{}/tools/upload/auto", self.base_url))
            .multipart(form)
            .send()
            .await
            .map_err(Error::Network)?;
        
        if !response.status().is_success() {
            return Err(Error::GalaxyApi(format!(
                "Failed to upload dataset: HTTP {}",
                response.status()
            )));
        }
        
        let dataset: crate::models::dataset::Dataset = response
            .json()
            .await
            .map_err(|e| Error::NetworkResponseDecode(e.to_string()))?;
        
        Ok(dataset)
    }
    
    /// Creates a new history
    pub async fn create_history(&self, name: &str) -> Result<crate::models::history::History> {
        debug!("Creating history: {}", name);
        
        let mut body = HashMap::new();
        body.insert("name", Value::String(name.to_string()));
        
        let response = self.client
            .post(format!("{}/histories", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(Error::Network)?;
        
        if !response.status().is_success() {
            return Err(Error::GalaxyApi(format!(
                "Failed to create history: HTTP {}",
                response.status()
            )));
        }
        
        let history: crate::models::history::History = response
            .json()
            .await
            .map_err(|e| Error::NetworkResponseDecode(e.to_string()))?;
        
        Ok(history)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_client_initialization() {
        let client = GalaxyClient::new(
            "https://usegalaxy.org/api",
            SecureCredentials::with_api_key(SecretString::new("test_key")),
            Some(Duration::from_secs(30)),
        )
        .unwrap();
        
        assert!(client.is_initialized());
        assert_eq!(client.base_url(), "https://usegalaxy.org/api");
    }
} 