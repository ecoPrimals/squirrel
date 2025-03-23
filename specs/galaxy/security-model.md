---
title: "Galaxy MCP Adapter Security Model"
description: "Security approach for the Galaxy MCP adapter crate"
version: "0.1.0"
last_updated: "2025-03-28"
status: "draft"
owners:
  primary: ["DataScienceBioLab", "mcp-team"]
  reviewers: ["core-team", "security-team"]
---

# Galaxy MCP Adapter Security Model

## 1. Overview

This specification defines the security approach for the Galaxy MCP adapter crate. It covers authentication handling, credential management, secure communication, and integration with the existing MCP security model.

## 2. Security Principles

The Galaxy MCP adapter adheres to the following security principles:

1. **Defense in Depth**: Multiple layers of security controls
2. **Principle of Least Privilege**: Minimal access rights for each operation
3. **Zero Trust**: All requests must be authenticated and authorized
4. **Data Minimization**: Limit sensitive data exposure
5. **Secure by Default**: Conservative default security settings

## 3. Authentication

### 3.1 Authentication Methods

The adapter supports the following authentication methods for Galaxy API access:

| Method | Description | Recommended Use |
|--------|-------------|----------------|
| API Key | Galaxy API key for direct authentication | Development and personal use |
| User Credentials | Username/password authentication | Discouraged, use API key instead |
| OAuth2 | OAuth2-based authentication flow | Production integrations |

### 3.2 API Key Management

```rust
pub struct GalaxyAdapterConfig {
    // Other fields...
    pub api_key: Option<String>,
    pub api_key_provider: Option<Box<dyn ApiKeyProvider>>,
}

pub trait ApiKeyProvider: Send + Sync {
    fn get_api_key(&self) -> Result<String, Error>;
}

// Example environment variable provider
pub struct EnvApiKeyProvider {
    env_var_name: String,
}

impl ApiKeyProvider for EnvApiKeyProvider {
    fn get_api_key(&self) -> Result<String, Error> {
        std::env::var(&self.env_var_name)
            .map_err(|_| Error::MissingApiKey(format!("Environment variable {} not set", self.env_var_name)))
    }
}

// Example from the secure context store
pub struct ContextApiKeyProvider<'a> {
    context: &'a context::Manager,
    credential_key: String,
}

impl<'a> ApiKeyProvider for ContextApiKeyProvider<'a> {
    fn get_api_key(&self) -> Result<String, Error> {
        self.context.get_credential(&self.credential_key)
            .map_err(|_| Error::MissingApiKey(format!("Credential {} not found in context", self.credential_key)))
    }
}
```

### 3.3 Authenticating Galaxy Requests

The adapter authenticates Galaxy API requests by adding the appropriate authentication header to each request:

```rust
impl GalaxyAdapter {
    async fn make_authenticated_request(&self, endpoint: &str, method: Method) -> Result<Response, Error> {
        let api_key = self.get_api_key().await?;
        
        let request = self.client
            .request(method, format!("{}/{}", self.config.galaxy_url, endpoint))
            .header("X-API-Key", api_key);
            
        let response = request.send().await?;
        if !response.status().is_success() {
            return Err(Error::GalaxyApiError(response.status()));
        }
        
        Ok(response)
    }
    
    async fn get_api_key(&self) -> Result<String, Error> {
        if let Some(api_key) = &self.config.api_key {
            return Ok(api_key.clone());
        }
        
        if let Some(provider) = &self.config.api_key_provider {
            return provider.get_api_key();
        }
        
        Err(Error::MissingApiKey("No API key or provider configured".into()))
    }
}
```

## 4. Credential Management

### 4.1 Secure Credential Storage

The adapter leverages the existing context crate's secure credential storage:

```rust
impl GalaxyAdapter {
    pub fn store_credentials(&self, api_key: &str) -> Result<(), Error> {
        self.context.store_credential("galaxy_api_key", api_key)
            .map_err(|e| Error::CredentialStorageError(e.to_string()))
    }
    
    pub fn clear_credentials(&self) -> Result<(), Error> {
        self.context.remove_credential("galaxy_api_key")
            .map_err(|e| Error::CredentialStorageError(e.to_string()))
    }
}
```

### 4.2 Environment Variable Handling

If using environment variables for credentials, follow these practices:

1. Use process-specific environment variables when possible
2. Clear environment variables after reading them
3. Use a prefix like `GALAXY_MCP_` for all environment variables
4. Document the specific environment variables used

```rust
// Clear sensitive environment variables after use
fn clear_env_var(var_name: &str) {
    if std::env::var(var_name).is_ok() {
        // Note: This is not completely secure as the value may be cached elsewhere
        std::env::remove_var(var_name);
    }
}

// Usage
let api_key = std::env::var("GALAXY_MCP_API_KEY").ok();
clear_env_var("GALAXY_MCP_API_KEY");
```

## 5. Integration with MCP Security

### 5.1 MCP Security Context

The adapter integrates with the MCP security model:

```rust
impl GalaxyAdapter {
    async fn handle_tool_execution(&self, message: &mcp::Message) -> Result<mcp::Message, Error> {
        // Extract security context from MCP message
        let security_context = self.mcp.extract_security_context(message)?;
        
        // Validate permissions
        if !security_context.has_permission("galaxy:execute_tool") {
            return Err(Error::PermissionDenied("Not authorized to execute Galaxy tools".into()));
        }
        
        // Extract tool execution parameters
        let params = message.payload().get_tool_execution_params()?;
        
        // Execute tool with authenticated Galaxy API
        let job_id = self.execute_tool(params.tool_id, &params.inputs).await?;
        
        // Create MCP response
        Ok(mcp::Message::new_tool_execution_response(job_id))
    }
}
```

### 5.2 Permission Mapping

The adapter maps MCP permissions to Galaxy permissions:

| MCP Permission | Galaxy Action | Description |
|----------------|---------------|-------------|
| galaxy:discover_tools | List tools, Get tool details | Discover available Galaxy tools |
| galaxy:execute_tool | Run job | Execute a Galaxy tool |
| galaxy:manage_data | Upload dataset, Get dataset | Manage Galaxy datasets |
| galaxy:manage_workflow | Run workflow, Get workflow | Manage Galaxy workflows |
| galaxy:admin | Admin actions | Administrative actions |

## 6. Secure Communication

### 6.1 TLS Configuration

The adapter ensures secure communication with Galaxy:

```rust
use reqwest::ClientBuilder;
use std::time::Duration;

impl GalaxyAdapter {
    fn create_secure_client(&self) -> Result<reqwest::Client, Error> {
        let mut client_builder = ClientBuilder::new()
            .use_rustls_tls() // Prefer Rustls over OpenSSL
            .min_tls_version(reqwest::tls::Version::TLS_1_2) // Minimum TLS 1.2
            .timeout(Duration::from_secs(self.config.timeout))
            .connect_timeout(Duration::from_secs(self.config.connect_timeout));
        
        // Add TLS certificate verification (can be disabled for local testing only)
        if self.config.verify_tls {
            client_builder = client_builder.danger_accept_invalid_certs(false);
        } else {
            log::warn!("TLS certificate verification disabled. Not recommended for production use!");
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }
        
        client_builder.build().map_err(|e| Error::ClientCreationError(e.to_string()))
    }
}
```

### 6.2 Request Validation

The adapter validates all requests before sending them to Galaxy:

```rust
impl GalaxyAdapter {
    fn validate_tool_inputs(&self, tool_id: &str, inputs: &HashMap<String, ToolInput>) -> Result<(), Error> {
        // Check tool_id format
        if !tool_id.contains("/") && !tool_id.contains("?") {
            return Err(Error::ValidationError("Invalid tool ID format".into()));
        }
        
        // Validate input values against known tool parameters
        let tool = self.get_tool_definition(tool_id).await?;
        for (name, _) in inputs {
            if !tool.parameters.iter().any(|p| p.name == *name) {
                return Err(Error::ValidationError(format!("Unknown parameter: {}", name)));
            }
        }
        
        Ok(())
    }
}
```

## 7. Error Handling and Logging

### 7.1 Security-Related Errors

The adapter defines specific error types for security issues:

```rust
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("API key expired or invalid")]
    InvalidApiKey,
    
    #[error("TLS error: {0}")]
    TlsError(String),
    
    #[error("Credential storage error: {0}")]
    CredentialStorageError(String),
}
```

### 7.2 Secure Logging

Security-sensitive information is never logged:

```rust
impl GalaxyAdapter {
    async fn authenticate(&self) -> Result<(), Error> {
        match self.get_api_key().await {
            Ok(_) => {
                log::debug!("Successfully authenticated with Galaxy API");
                Ok(())
            }
            Err(e) => {
                log::error!("Authentication failed: {}", e);
                // Note: Error message doesn't include the actual API key
                Err(e)
            }
        }
    }
}
```

## 8. Implementation Guidelines

### 8.1 Security Best Practices

The adapter implementation follows these security best practices:

1. **Avoid Security Sensitive Data in Memory**: Minimize time credentials are in memory
2. **Input Validation**: Validate all input before sending to Galaxy API
3. **Rate Limiting**: Implement backoff for failed authentication attempts
4. **Error Messages**: Provide minimal information in error messages
5. **Secure Dependencies**: Review and update dependencies regularly

### 8.2 Testing Security Features

Security features are tested as part of the adapter's test suite:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_authentication_failure() {
        let config = GalaxyAdapterConfig {
            galaxy_url: "https://usegalaxy.org/api".to_string(),
            api_key: Some("invalid-key".to_string()),
            ..Default::default()
        };
        
        let adapter = GalaxyAdapter::new(config);
        let result = adapter.list_tools().await;
        
        assert!(result.is_err());
        match result {
            Err(Error::SecurityError(SecurityError::AuthenticationFailed(_))) => (),
            _ => panic!("Expected authentication failure error"),
        }
    }
    
    #[tokio::test]
    async fn test_secure_credential_storage() {
        let context = context::Manager::new();
        let config = GalaxyAdapterConfig {
            galaxy_url: "https://usegalaxy.org/api".to_string(),
            api_key_provider: Some(Box::new(ContextApiKeyProvider {
                context: &context,
                credential_key: "test_key".to_string(),
            })),
            ..Default::default()
        };
        
        // Store a credential
        context.store_credential("test_key", "test-api-key").unwrap();
        
        let adapter = GalaxyAdapter::new(config);
        let api_key = adapter.get_api_key().await;
        
        assert!(api_key.is_ok());
        assert_eq!(api_key.unwrap(), "test-api-key");
    }
}
```

## 9. Related Specifications

- [Galaxy MCP Integration Plan](galaxy-mcp-integration.md)
- [Configuration Management](configuration-management.md)
- [Deployment Guide](deployment-guide.md)

<version>0.1.0</version> 