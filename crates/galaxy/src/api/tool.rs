//! Module for Galaxy tool API endpoints
//! 
//! This module provides endpoints for interacting with Galaxy tools,
//! including listing tools, getting tool details, and executing tools.

use crate::api::{GalaxyEndpoint, ApiResponse, build_query_params};
use crate::models::tool::{GalaxyTool, ToolExecutionRequest, ToolExecutionResponse};
use reqwest::Method;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Endpoint to list all available tools
#[derive(Default)]
pub struct ListTools {
    /// Whether to include details for each tool
    pub include_details: bool,
    
    /// Filter tools by name/text
    pub query: Option<String>,
    
    /// Filter tools by a particular section
    pub section: Option<String>,
    
    /// Filter tools by a tool shed repository
    pub tool_shed: Option<String>,
}

/// Endpoint to get details for a specific tool
pub struct GetTool {
    /// The ID of the tool to retrieve
    pub id: String,
    
    /// Whether to include input details
    pub include_inputs: bool,
    
    /// Whether to include output details
    pub include_outputs: bool,
}

/// Endpoint to execute a tool
pub struct ExecuteTool {
    /// The tool execution request
    pub request: ToolExecutionRequest,
}

/// Endpoint to get available datatypes
pub struct GetDatatypes {}

/// Tool search parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchParams {
    /// Text to search for in tool names and descriptions
    pub query: Option<String>,
    
    /// Filter tools by section
    pub section: Option<String>,
    
    /// Include inputs in the response
    pub include_inputs: Option<bool>,
    
    /// Include outputs in the response
    pub include_outputs: Option<bool>,
    
    /// Include tool version information
    pub include_versions: Option<bool>,
    
    /// Filter tools by tool shed
    pub tool_shed: Option<String>,
}


#[async_trait::async_trait]
impl GalaxyEndpoint for ListTools {
    type Response = ApiResponse<Vec<GalaxyTool>>;
    
    fn method(&self) -> Method {
        Method::GET
    }
    
    fn path(&self) -> String {
        "/api/tools".to_string()
    }
    
    fn query_params(&self) -> Option<HashMap<String, String>> {
        let params = ToolSearchParams {
            query: self.query.clone(),
            section: self.section.clone(),
            include_inputs: Some(self.include_details),
            include_outputs: Some(self.include_details),
            include_versions: Some(true),
            tool_shed: self.tool_shed.clone(),
        };
        
        Some(build_query_params(&params))
    }
}

#[async_trait::async_trait]
impl GalaxyEndpoint for GetTool {
    type Response = ApiResponse<GalaxyTool>;
    
    fn method(&self) -> Method {
        Method::GET
    }
    
    fn path(&self) -> String {
        format!("/api/tools/{}", self.id)
    }
    
    fn query_params(&self) -> Option<HashMap<String, String>> {
        let mut params = HashMap::new();
        
        if self.include_inputs {
            params.insert("include_inputs".to_string(), "true".to_string());
        }
        
        if self.include_outputs {
            params.insert("include_outputs".to_string(), "true".to_string());
        }
        
        Some(params)
    }
}

#[async_trait::async_trait]
impl GalaxyEndpoint for ExecuteTool {
    type Response = ApiResponse<ToolExecutionResponse>;
    
    fn method(&self) -> Method {
        Method::POST
    }
    
    fn path(&self) -> String {
        "/api/tools".to_string()
    }
    
    fn body(&self) -> Option<serde_json::Value> {
        serde_json::to_value(&self.request).ok()
    }
}

#[async_trait::async_trait]
impl GalaxyEndpoint for GetDatatypes {
    type Response = ApiResponse<HashMap<String, String>>;
    
    fn method(&self) -> Method {
        Method::GET
    }
    
    fn path(&self) -> String {
        "/api/datatypes".to_string()
    }
}

/// Execute tool helper function
pub async fn execute_tool(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    tool_id: &str,
    history_id: &str,
    params: HashMap<String, serde_json::Value>,
) -> Result<ToolExecutionResponse, crate::error::Error> {
    // Convert from serde_json::Value to ParameterValue
    let param_values = params.into_iter()
        .map(|(k, v)| {
            let param_value = match v {
                serde_json::Value::String(s) => crate::models::ParameterValue::String(s),
                serde_json::Value::Number(n) => {
                    if let Some(f) = n.as_f64() {
                        crate::models::ParameterValue::Number(f)
                    } else {
                        crate::models::ParameterValue::String(n.to_string())
                    }
                },
                serde_json::Value::Bool(b) => crate::models::ParameterValue::Boolean(b),
                serde_json::Value::Null => crate::models::ParameterValue::Null,
                _ => crate::models::ParameterValue::String(v.to_string()),
            };
            (k, param_value)
        })
        .collect();

    let request = ToolExecutionRequest {
        tool_id: tool_id.to_string(),
        history_id: history_id.to_string(),
        parameters: param_values,
        create_history: false,
    };
    
    let endpoint = ExecuteTool { request };
    let response = endpoint.execute(client, base_url, api_key).await?;
    
    match response.data {
        Some(data) => Ok(data),
        None => Err(crate::error::Error::GalaxyApi(format!(
            "No data returned from tool execution for tool: {}",
            tool_id
        ))),
    }
}

/// Get tool details helper function
pub async fn get_tool_details(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    tool_id: &str,
) -> Result<GalaxyTool, crate::error::Error> {
    let endpoint = GetTool {
        id: tool_id.to_string(),
        include_inputs: true,
        include_outputs: true,
    };
    
    let response = endpoint.execute(client, base_url, api_key).await?;
    
    match response.data {
        Some(data) => Ok(data),
        None => Err(crate::error::Error::GalaxyApi(format!(
            "No data returned for tool: {}",
            tool_id
        ))),
    }
}

/// List available tools helper function
pub async fn list_tools(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    query: Option<&str>,
    section: Option<&str>,
) -> Result<Vec<GalaxyTool>, crate::error::Error> {
    let endpoint = ListTools {
        include_details: false,
        query: query.map(|s| s.to_string()),
        section: section.map(|s| s.to_string()),
        tool_shed: None,
    };
    
    let response = endpoint.execute(client, base_url, api_key).await?;
    
    match response.data {
        Some(data) => Ok(data),
        None => Ok(Vec::new()),
    }
}

/// Get available datatypes helper function
pub async fn get_datatypes(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
) -> Result<HashMap<String, String>, crate::error::Error> {
    let endpoint = GetDatatypes {};
    let response = endpoint.execute(client, base_url, api_key).await?;
    
    match response.data {
        Some(data) => Ok(data),
        None => Ok(HashMap::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Method;
    
    #[test]
    fn test_list_tools_endpoint() {
        let endpoint = ListTools {
            include_details: true,
            query: Some("sequence".to_string()),
            section: Some("Text Manipulation".to_string()),
            tool_shed: None,
        };
        
        assert_eq!(endpoint.method(), Method::GET);
        assert_eq!(endpoint.path(), "/api/tools");
        
        let params = endpoint.query_params().unwrap();
        assert_eq!(params.get("query").unwrap(), "sequence");
        assert_eq!(params.get("section").unwrap(), "Text Manipulation");
        assert_eq!(params.get("include_inputs").unwrap(), "true");
        assert_eq!(params.get("include_outputs").unwrap(), "true");
        assert_eq!(params.get("include_versions").unwrap(), "true");
    }
    
    #[test]
    fn test_get_tool_endpoint() {
        let endpoint = GetTool {
            id: "toolshed.g2.bx.psu.edu/repos/devteam/fastqc/fastqc/0.72+galaxy1".to_string(),
            include_inputs: true,
            include_outputs: false,
        };
        
        assert_eq!(endpoint.method(), Method::GET);
        assert_eq!(
            endpoint.path(),
            "/api/tools/toolshed.g2.bx.psu.edu/repos/devteam/fastqc/fastqc/0.72+galaxy1"
        );
        
        let params = endpoint.query_params().unwrap();
        assert_eq!(params.get("include_inputs").unwrap(), "true");
        assert!(params.get("include_outputs").is_none());
    }
} 