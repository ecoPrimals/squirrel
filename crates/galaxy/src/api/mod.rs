//! Module for Galaxy API endpoints and interactions
//! 
//! This module provides structured access to the Galaxy API endpoints,
//! organized by resource type with consistent patterns.

use std::collections::HashMap;
use reqwest::{Method, Client};
use serde::de::DeserializeOwned;
use serde::{Serialize, Deserialize};
use crate::error::Result;
use std::time::Duration;

/// Re-export API modules
pub mod tool;
//pub mod workflow;
//pub mod history;
//pub mod dataset;
//pub mod library;
//pub mod job;
//pub mod user;

/// Common API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Whether the request was successful
    pub ok: bool,
    
    /// The response data
    pub data: Option<T>,
    
    /// Error message, if any
    pub error: Option<String>,
    
    /// Status code from the Galaxy API
    pub status: u16,
    
    /// Additional details about the response
    pub message: Option<String>,
}

/// Common parameters for pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    /// The page number (1-based indexing)
    pub page: Option<u32>,
    
    /// The number of items per page
    pub page_size: Option<u32>,
    
    /// The field to sort by
    pub sort_by: Option<String>,
    
    /// The sort direction (asc or desc)
    pub sort_desc: Option<bool>,
}

/// Common parameters for filtering results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterParams {
    /// Only return items matching this name
    pub name: Option<String>,
    
    /// Only return items with this ID
    pub id: Option<String>,
    
    /// Only return items with these tags
    pub tags: Option<Vec<String>>,
    
    /// Only return items created after this date
    pub created_after: Option<String>,
    
    /// Only return items created before this date
    pub created_before: Option<String>,
    
    /// Only return items with matching text in description or name
    pub query: Option<String>,
    
    /// Include deleted items
    pub include_deleted: Option<bool>,
}

/// Trait representing a Galaxy API endpoint
pub trait GalaxyEndpoint {
    /// The response type for this endpoint
    type Response: serde::de::DeserializeOwned;
    
    /// Get the path for this endpoint
    fn path(&self) -> String;
    
    /// Get the HTTP method for this endpoint
    fn method(&self) -> Method {
        Method::GET
    }
    
    /// Optional query parameters
    fn query_params(&self) -> Option<HashMap<String, String>> {
        None
    }
    
    /// Optional request body
    fn body(&self) -> Option<serde_json::Value> {
        None
    }
    
    /// Execute this endpoint against a Galaxy API
    #[allow(async_fn_in_trait)]
    async fn execute(&self, client: &Client, base_url: &str, api_key: &str) -> crate::error::Result<Self::Response> {
        let url = format!("{}{}", base_url, self.path());
        
        let mut request = client
            .request(self.method(), url)
            .header("X-API-KEY", api_key)
            .header("Content-Type", "application/json");
        
        if let Some(params) = self.query_params() {
            request = request.query(&params);
        }
        
        if let Some(body) = self.body() {
            request = request.json(&body);
        }
        
        let response = request
            .send()
            .await
            .map_err(crate::error::Error::Network)?;
        
        if response.status().is_success() {
            let data = response
                .json::<Self::Response>()
                .await
                .map_err(|e| crate::error::Error::NetworkResponseDecode(format!("Failed to parse response: {}", e)))?;
            
            Ok(data)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            Err(crate::error::Error::GalaxyApi(format!(
                "API error: {} - {}", status, error_text
            )))
        }
    }
}

/// Creates an API client with specified timeout
pub fn create_api_client(timeout_seconds: u64) -> crate::error::Result<Client> {
    Client::builder()
        .timeout(Duration::from_secs(timeout_seconds))
        .build()
        .map_err(|e| crate::error::Error::Config(format!("Failed to create HTTP client: {}", e)))
}

/// Helper function to build a query string from parameters
pub fn build_query_params<T: Serialize>(params: &T) -> HashMap<String, String> {
    let value = serde_json::to_value(params).unwrap_or(serde_json::Value::Null);
    
    if let serde_json::Value::Object(map) = value {
        map.into_iter()
            .filter_map(|(key, value)| {
                if value.is_null() {
                    None
                } else {
                    Some((key, value.to_string().trim_matches('"').to_string()))
                }
            })
            .collect()
    } else {
        HashMap::new()
    }
}

/// Helper function to fetch paginated data from a Galaxy endpoint
pub async fn paginated_get<T>(
    client: &Client,
    base_url: &str,
    api_key: &str,
    resource: &str,
    params: &HashMap<String, String>,
    page_size: u32,
    max_pages: Option<u32>,
) -> Result<Vec<T>>
where
    T: DeserializeOwned,
{
    let mut results = Vec::new();
    let mut current_page = 1;
    let max_pages = max_pages.unwrap_or(100); // Default to a reasonable limit
    
    loop {
        let mut query_params = params.clone();
        query_params.insert("page".to_string(), current_page.to_string());
        query_params.insert("page_size".to_string(), page_size.to_string());
        
        let response = get::<ApiResponse<Vec<T>>>(
            client,
            base_url,
            api_key,
            resource,
            &query_params,
        ).await?;
        
        if let Some(data) = response.data {
            let data_len = data.len();
            results.extend(data);
            
            // Check if we've reached the end of data
            if data_len < page_size as usize {
                break;
            }
        } else {
            // No data in response
            break;
        }
        
        current_page += 1;
        if current_page > max_pages {
            break;
        }
    }
    
    Ok(results)
}

/// Make a GET request to a Galaxy API endpoint
pub async fn get<T>(
    client: &Client,
    base_url: &str,
    api_key: &str,
    resource: &str,
    params: &HashMap<String, String>,
) -> Result<T>
where
    T: DeserializeOwned,
{
    let url = format!("{}/{}", base_url, resource);
    
    let mut request = client.get(&url);
    
    // Add API key to headers
    request = request.header("X-API-Key", api_key);
    
    // Add query parameters
    if !params.is_empty() {
        request = request.query(params);
    }
    
    // Send request and decode response
    let response = request
        .send()
        .await
        .map_err(crate::error::Error::Network)?;
    
    let status = response.status();
    
    if status.is_success() {
        let data = response
            .json::<T>()
            .await
            .map_err(|e| crate::error::Error::NetworkResponseDecode(format!("Failed to parse response: {}", e)))?;
        
        Ok(data)
    } else {
        let error_text = response
            .text()
            .await
            .map_err(crate::error::Error::Network)?;
        
        Err(crate::error::Error::GalaxyApi(format!(
            "API error ({}) from {}: {}",
            status, url, error_text
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Method;
    
    struct TestEndpoint;
    
    #[async_trait::async_trait]
    impl GalaxyEndpoint for TestEndpoint {
        type Response = serde_json::Value;
        
        fn method(&self) -> Method {
            Method::GET
        }
        
        fn path(&self) -> String {
            "/api/test".to_string()
        }
    }
    
    #[test]
    fn test_build_query_params() {
        #[derive(Serialize)]
        struct TestParams {
            param1: Option<String>,
            param2: Option<u32>,
            param3: Option<bool>,
        }
        
        let params = TestParams {
            param1: Some("value1".to_string()),
            param2: Some(42),
            param3: None,
        };
        
        let query_params = build_query_params(&params);
        
        assert_eq!(query_params.get("param1").unwrap(), "value1");
        assert_eq!(query_params.get("param2").unwrap(), "42");
        assert!(query_params.get("param3").is_none());
    }
} 