# Adapter Implementation Guide for Dependency Injection

This guide demonstrates how to properly implement the adapter pattern for dependency injection in our codebase. We'll use the Alert Manager and the new Galaxy MCP implementation as examples.

## 1. Basic Adapter Structure

```rust
/// Adapter for the Alert Manager to support dependency injection
#[derive(Debug)]
pub struct AlertManagerAdapter {
    /// The inner manager instance
    inner: Option<Arc<AlertManager>>,
}

impl AlertManagerAdapter {
    /// Creates a new adapter without initializing it
    pub fn new() -> Self {
        Self { inner: None }
    }
    
    /// Creates an adapter with an existing manager
    pub fn with_manager(manager: Arc<AlertManager>) -> Self {
        Self { inner: Some(manager) }
    }
    
    /// Checks if the adapter is initialized
    pub fn is_initialized(&self) -> bool {
        self.inner.is_some()
    }
}
```

### 1.1 Real-World Implementation: Galaxy MCP Adapter

The Galaxy MCP Adapter follows this pattern with a more comprehensive structure:

```rust
/// Adapter for integrating Galaxy with the MCP protocol
#[derive(Debug)]
pub struct GalaxyAdapter {
    /// The MCP protocol handler
    mcp: Option<Arc<mcp::Protocol>>,
    /// The context manager for handling state
    context: Option<Arc<context::Manager>>,
    /// The Galaxy client for API communication
    galaxy_client: Option<GalaxyClient>,
    /// Configuration for the adapter
    config: GalaxyConfig,
}

impl GalaxyAdapter {
    /// Creates a new adapter with default configuration
    pub fn new() -> Self {
        Self {
            mcp: None,
            context: None,
            galaxy_client: None,
            config: GalaxyConfig::default(),
        }
    }
    
    /// Creates a new adapter with custom configuration
    pub fn with_config(config: GalaxyConfig) -> Self {
        Self {
            mcp: None,
            context: None,
            galaxy_client: None,
            config,
        }
    }
    
    /// Creates an adapter with existing components
    pub fn with_components(
        mcp: Arc<mcp::Protocol>,
        context: Arc<context::Manager>,
        client: GalaxyClient,
        config: GalaxyConfig,
    ) -> Self {
        Self {
            mcp: Some(mcp),
            context: Some(context),
            galaxy_client: Some(client),
            config,
        }
    }
    
    /// Checks if the adapter is fully initialized
    pub fn is_initialized(&self) -> bool {
        self.mcp.is_some() && self.context.is_some() && self.galaxy_client.is_some()
    }
}
```

## 2. Initialization Methods

```rust
impl AlertManagerAdapter {
    /// Initializes the adapter with default configuration
    pub fn initialize(&mut self) -> Result<(), AlertError> {
        if self.is_initialized() {
            return Err(AlertError::AlreadyInitialized);
        }
        
        let config = AlertConfig::default();
        let manager = AlertManager::new(config);
        self.inner = Some(Arc::new(manager));
        Ok(())
    }
    
    /// Initializes the adapter with custom configuration
    pub fn initialize_with_config(&mut self, config: AlertConfig) -> Result<(), AlertError> {
        if self.is_initialized() {
            return Err(AlertError::AlreadyInitialized);
        }
        
        let manager = AlertManager::new(config);
        self.inner = Some(Arc::new(manager));
        Ok(())
    }
}
```

### 2.1 Initialization Methods: Galaxy MCP Adapter

The Galaxy MCP Adapter demonstrates more complex initialization with secure credential handling:

```rust
impl GalaxyAdapter {
    /// Initializes the adapter with the current configuration
    pub async fn initialize(&mut self) -> Result<(), GalaxyError> {
        if self.is_initialized() {
            return Err(GalaxyError::AlreadyInitialized);
        }
        
        // Initialize MCP protocol
        let mcp = mcp::Protocol::new();
        
        // Initialize context manager
        let context = context::Manager::new();
        
        // Initialize Galaxy client with secure credentials
        let credentials = self.config.get_secure_credentials()?;
        let galaxy_client = GalaxyClient::new(&self.config.galaxy_url, credentials)?;
        
        // Store components
        self.mcp = Some(Arc::new(mcp));
        self.context = Some(Arc::new(context));
        self.galaxy_client = Some(galaxy_client);
        
        Ok(())
    }
    
    /// Initializes the adapter with custom components
    pub fn initialize_with_components(
        &mut self,
        mcp: mcp::Protocol,
        context: context::Manager,
        credentials: SecureCredentials,
    ) -> Result<(), GalaxyError> {
        if self.is_initialized() {
            return Err(GalaxyError::AlreadyInitialized);
        }
        
        // Initialize Galaxy client with provided credentials
        let galaxy_client = GalaxyClient::new(&self.config.galaxy_url, credentials)?;
        
        // Store components
        self.mcp = Some(Arc::new(mcp));
        self.context = Some(Arc::new(context));
        self.galaxy_client = Some(galaxy_client);
        
        Ok(())
    }
    
    /// Reinitializes with new configuration
    pub async fn reinitialize_with_config(&mut self, config: GalaxyConfig) -> Result<(), GalaxyError> {
        // Reset internal state
        self.mcp = None;
        self.context = None;
        self.galaxy_client = None;
        
        // Update configuration
        self.config = config;
        
        // Reinitialize
        self.initialize().await
    }
}
```

## 3. Operation Methods with Proper Error Handling

```rust
impl AlertManagerAdapter {
    /// Registers an alert rule
    pub fn register_rule(&self, rule: AlertRule) -> Result<Uuid, AlertError> {
        match &self.inner {
            Some(manager) => manager.register_rule(rule),
            None => Err(AlertError::NotInitialized)
        }
    }
    
    /// Processes an alert event
    pub async fn process_event(&self, event: AlertEvent) -> Result<bool, AlertError> {
        match &self.inner {
            Some(manager) => manager.process_event(event).await,
            None => Err(AlertError::NotInitialized)
        }
    }
    
    /// Gets all registered alert rules
    pub fn get_rules(&self) -> Result<Vec<AlertRule>, AlertError> {
        match &self.inner {
            Some(manager) => Ok(manager.get_rules()),
            None => Err(AlertError::NotInitialized)
        }
    }
}
```

### 3.1 Operation Methods: Galaxy MCP Adapter

The Galaxy MCP Adapter demonstrates comprehensive operation methods:

```rust
impl GalaxyAdapter {
    /// Handles an incoming MCP message
    pub async fn handle_message(&self, message: mcp::Message) -> Result<mcp::Message, GalaxyError> {
        // Check initialization
        if !self.is_initialized() {
            return Err(GalaxyError::NotInitialized);
        }
        
        // Extract message type and handle accordingly
        match message.message_type() {
            mcp::MessageType::ToolDiscovery => self.handle_tool_discovery(&message).await,
            mcp::MessageType::ToolExecution => self.handle_tool_execution(&message).await,
            mcp::MessageType::JobStatus => self.handle_job_status(&message).await,
            mcp::MessageType::WorkflowDiscovery => self.handle_workflow_discovery(&message).await,
            mcp::MessageType::WorkflowExecution => self.handle_workflow_execution(&message).await,
            _ => Err(GalaxyError::UnsupportedMessageType(message.message_type())),
        }
    }
    
    /// Discovers available Galaxy tools
    pub async fn discover_tools(&self) -> Result<Vec<mcp::Tool>, GalaxyError> {
        self.ensure_initialized()?;
        
        let galaxy_client = self.galaxy_client.as_ref().unwrap();
        let galaxy_tools = galaxy_client.list_tools().await?;
        
        // Convert Galaxy tools to MCP tools
        let mut mcp_tools = Vec::with_capacity(galaxy_tools.len());
        for tool in galaxy_tools {
            mcp_tools.push(self.convert_to_mcp_tool(tool)?);
        }
        
        Ok(mcp_tools)
    }
    
    /// Executes a Galaxy tool
    pub async fn execute_tool(
        &self, 
        tool_id: &str, 
        parameters: &HashMap<String, Value>
    ) -> Result<String, GalaxyError> {
        self.ensure_initialized()?;
        
        // Validate tool parameters
        self.validate_tool_parameters(tool_id, parameters).await?;
        
        // Execute tool
        let galaxy_client = self.galaxy_client.as_ref().unwrap();
        let job_id = galaxy_client.execute_tool(tool_id, parameters).await?;
        
        // Store job in context for tracking
        let context = self.context.as_ref().unwrap();
        context.store_job_context(job_id.clone(), tool_id, parameters)?;
        
        Ok(job_id)
    }
    
    /// Helper to ensure adapter is initialized
    fn ensure_initialized(&self) -> Result<(), GalaxyError> {
        if !self.is_initialized() {
            return Err(GalaxyError::NotInitialized);
        }
        Ok(())
    }
}
```

## 4. Factory Functions for Easy Creation

```rust
/// Creates and initializes an alert manager adapter with default configuration
pub fn create_initialized_alert_adapter() -> Result<AlertManagerAdapter, AlertError> {
    let mut adapter = AlertManagerAdapter::new();
    adapter.initialize()?;
    Ok(adapter)
}

/// Creates and initializes an alert manager adapter with custom configuration
pub fn create_alert_adapter_with_config(config: AlertConfig) -> Result<AlertManagerAdapter, AlertError> {
    let mut adapter = AlertManagerAdapter::new();
    adapter.initialize_with_config(config)?;
    Ok(adapter)
}
```

### 4.1 Factory Functions: Galaxy MCP Adapter

The Galaxy MCP Adapter provides more sophisticated factory functions:

```rust
/// Creates and initializes a Galaxy adapter with default configuration
pub async fn create_galaxy_adapter() -> Result<GalaxyAdapter, GalaxyError> {
    let mut adapter = GalaxyAdapter::new();
    adapter.initialize().await?;
    Ok(adapter)
}

/// Creates and initializes a Galaxy adapter with configuration from environment
pub async fn create_galaxy_adapter_from_env() -> Result<GalaxyAdapter, GalaxyError> {
    // Load configuration from environment variables with secure handling
    let config = GalaxyConfig::from_env()?;
    let mut adapter = GalaxyAdapter::with_config(config);
    adapter.initialize().await?;
    Ok(adapter)
}

/// Creates a Galaxy adapter for testing with mock components
pub fn create_test_galaxy_adapter() -> GalaxyAdapter {
    // Create mock components
    let mcp = Arc::new(mcp::Protocol::new());
    let context = Arc::new(context::Manager::new());
    let mock_client = GalaxyClient::new_mock();
    let config = GalaxyConfig::for_testing();
    
    // Create adapter with components
    GalaxyAdapter::with_components(mcp, context, mock_client, config)
}
```

## 5. Error Types

```rust
#[derive(Debug, Error)]
pub enum AlertError {
    #[error("Alert Manager not initialized")]
    NotInitialized,
    
    #[error("Alert Manager already initialized")]
    AlreadyInitialized,
    
    #[error("Invalid alert rule: {0}")]
    InvalidRule(String),
    
    #[error("Failed to process alert: {0}")]
    ProcessingFailure(String),
}
```

### 5.1 Error Types: Galaxy MCP Adapter

The Galaxy MCP Adapter demonstrates a comprehensive error hierarchy:

```rust
#[derive(Debug, Error)]
pub enum GalaxyError {
    #[error("Galaxy adapter not initialized")]
    NotInitialized,
    
    #[error("Galaxy adapter already initialized")]
    AlreadyInitialized,
    
    #[error("API error: {0}")]
    ApiError(#[from] ApiError),
    
    #[error("Authorization error: {0}")]
    AuthError(#[from] AuthError),
    
    #[error("Configuration error: {0}")]
    ConfigError(#[from] ConfigError),
    
    #[error("Invalid tool ID: {0}")]
    InvalidToolId(String),
    
    #[error("Invalid parameter: {0} - {1}")]
    InvalidParameter(String, String),
    
    #[error("Unsupported message type: {0:?}")]
    UnsupportedMessageType(mcp::MessageType),
    
    #[error("Job error: {0}")]
    JobError(#[from] JobError),
    
    #[error("Context error: {0}")]
    ContextError(#[from] context::Error),
    
    #[error("Security error: {0}")]
    SecurityError(#[from] security::Error),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// API-specific errors
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("API returned error: {status} {message}")]
    ServerError { status: u16, message: String },
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Connection timeout")]
    Timeout,
    
    #[error("API response parsing error: {0}")]
    ParseError(String),
}

/// Authentication and authorization errors
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Missing API key")]
    MissingApiKey,
    
    #[error("Invalid API key")]
    InvalidApiKey,
    
    #[error("API key expired")]
    ExpiredApiKey,
    
    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
}
```

## 6. Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_uninitialized_adapter() {
        let adapter = AlertManagerAdapter::new();
        let rule = AlertRule::new("test", "test condition");
        
        // Should fail when not initialized
        let result = adapter.register_rule(rule);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, AlertError::NotInitialized));
        }
    }
    
    #[tokio::test]
    async fn test_initialized_adapter() {
        let mut adapter = AlertManagerAdapter::new();
        adapter.initialize().unwrap();
        
        let rule = AlertRule::new("test", "test condition");
        let result = adapter.register_rule(rule);
        assert!(result.is_ok());
    }
}
```

### 6.1 Testing: Galaxy MCP Adapter

The Galaxy MCP Adapter demonstrates comprehensive testing:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_uninitialized_adapter() {
        let adapter = GalaxyAdapter::new();
        let message = mcp::Message::new_tool_discovery_request();
        
        // Should fail when not initialized
        let result = adapter.handle_message(message).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, GalaxyError::NotInitialized));
        }
    }
    
    #[tokio::test]
    async fn test_tool_discovery() {
        // Use the test factory to create an adapter with mock components
        let adapter = create_test_galaxy_adapter();
        
        // Configure mock client expectations
        let mock_tools = vec![
            GalaxyTool::new("tool1", "Tool 1", "Category 1"),
            GalaxyTool::new("tool2", "Tool 2", "Category 2"),
        ];
        adapter.galaxy_client().expect_list_tools()
            .times(1)
            .returning(move || Ok(mock_tools.clone()));
        
        // Test tool discovery
        let tools = adapter.discover_tools().await.unwrap();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].id(), "tool1");
        assert_eq!(tools[1].id(), "tool2");
    }
    
    #[tokio::test]
    async fn test_secure_credentials() {
        // Test credential handling with secure storage
        let config = GalaxyConfig::new()
            .with_url("https://usegalaxy.org")
            .with_api_key(SecretString::new("test-key"));
        
        let adapter = GalaxyAdapter::with_config(config);
        
        // Verify credentials are not exposed in debug output
        let debug_str = format!("{:?}", adapter);
        assert!(!debug_str.contains("test-key"));
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        let adapter = create_test_galaxy_adapter();
        
        // Configure mock client to return an error
        adapter.galaxy_client().expect_list_tools()
            .times(1)
            .returning(|| Err(ApiError::ServerError { 
                status: 500, 
                message: "Internal server error".to_string() 
            }.into()));
        
        // Test error handling
        let result = adapter.discover_tools().await;
        assert!(result.is_err());
        if let Err(e) = result {
            if let GalaxyError::ApiError(api_err) = e {
                if let ApiError::ServerError { status, .. } = api_err {
                    assert_eq!(status, 500);
                } else {
                    panic!("Wrong error type");
                }
            } else {
                panic!("Wrong error type");
            }
        }
    }
}
```

## 7. Example Usage

### 7.1 Example Usage: Galaxy MCP Adapter

The Galaxy MCP Adapter is used in the following ways:

```rust
// Example 1: Using factory function for simple use cases
async fn discover_galaxy_tools() -> Result<Vec<Tool>, Error> {
    // Create and initialize adapter
    let adapter = create_galaxy_adapter().await?;
    
    // Use adapter to discover tools
    let tools = adapter.discover_tools().await?;
    
    Ok(tools)
}

// Example 2: Advanced usage with custom configuration
async fn run_galaxy_workflow(workflow_id: &str, inputs: &HashMap<String, Value>) -> Result<String, Error> {
    // Create configuration with secure credential handling
    let config = GalaxyConfig::new()
        .with_url(std::env::var("GALAXY_URL")?)
        .with_api_key(SecureString::from_env("GALAXY_API_KEY")?);
    
    // Create and initialize adapter
    let mut adapter = GalaxyAdapter::with_config(config);
    adapter.initialize().await?;
    
    // Execute workflow
    let execution_id = adapter.execute_workflow(workflow_id, inputs).await?;
    
    Ok(execution_id)
}

// Example 3: Integration with MCP system
async fn handle_mcp_message(message: mcp::Message) -> Result<mcp::Message, Error> {
    // Get or create adapter
    let adapter = get_or_create_galaxy_adapter().await?;
    
    // Process message
    let response = adapter.handle_message(message).await?;
    
    Ok(response)
}

// Example 4: Dependency injection in a larger system
struct GalaxyService {
    adapter: GalaxyAdapter,
}

impl GalaxyService {
    async fn new() -> Result<Self, Error> {
        let adapter = create_galaxy_adapter().await?;
        Ok(Self { adapter })
    }
    
    async fn execute_analysis_pipeline(&self, pipeline: &AnalysisPipeline) -> Result<PipelineResult, Error> {
        // Use the adapter for various operations
        let tools = self.adapter.discover_tools().await?;
        
        // Process pipeline
        // ...
        
        Ok(result)
    }
}
```

## 8. Best Practices

1. **Never initialize implicitly**: Always require explicit initialization.
2. **Return clear errors**: Return descriptive errors when the adapter is not initialized.
3. **Provide factory functions**: Make it easy to create and initialize adapters.
4. **Use appropriate error types**: Define specific error types for your adapter.
5. **Test initialization states**: Verify behavior for both initialized and uninitialized states.
6. **Document usage patterns**: Provide clear examples of how to use your adapter.
7. **Consider thread safety**: Use Arc for shared ownership when needed.
8. **Avoid global state**: Never use static variables or global state in adapters.
9. **Handle security properly**: Use secure types for credentials and sensitive data.
10. **Support testing**: Provide mock implementations for testing.

## 9. Implementation Checklist

### Galaxy MCP Adapter Implementation Progress

- [x] Create adapter struct with proper component references
- [x] Implement new() constructor and with_* constructors
- [x] Add is_initialized() method
- [x] Implement initialize() and specialized initialization methods
- [x] Create secure credential handling
- [x] Develop comprehensive error hierarchy
- [x] Implement tool discovery functionality
- [x] Add tool execution capabilities
- [x] Create context management integration
- [x] Implement MCP message handling
- [x] Develop secure API client
- [x] Add basic workflow functionality
- [x] Create factory functions for easy creation
- [x] Implement proper job monitoring
- [x] Add initial data management support
- [x] Create basic unit tests
- [x] Begin integration tests
- [ ] Complete security-focused tests
- [ ] Finalize documentation
- [ ] Add performance optimizations
- [ ] Implement advanced features 