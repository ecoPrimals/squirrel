use std::sync::Arc;
use crate::config::McpConfig as MCPConfig;
use crate::adapter::{MCPAdapter, MCPInterface};

/// Factory for creating MCP instances.
///
/// The `MCPFactory` provides a structured way to create MCP (Machine Context Protocol) instances
/// with consistent configuration. This factory pattern centralizes instance creation,
/// simplifying dependency management and configuration throughout the application.
///
/// # Factory Pattern Benefits
///
/// Using this factory provides several advantages:
/// - Standardized instance creation with consistent configuration
/// - Separation of configuration from usage
/// - Support for dependency injection during testing
/// - Simplified creation of multiple instances with the same settings
///
/// # Threading
///
/// The factory produces instances wrapped in `Arc` (Atomic Reference Counting),
/// making them safe to share across multiple threads.
///
/// # Examples
///
/// Basic usage with default configuration:
///
/// ```rust,no_run
/// use squirrel_mcp::factory::create_mcp;
///
/// // Create an MCP instance with default configuration
/// let mcp = create_mcp();
///
/// // Use the MCP instance
/// let result = mcp.initialize();
/// ```
///
/// Using custom configuration:
///
/// ```no_run
/// use squirrel_mcp::{MCPConfig, factory::MCPFactory};
///
/// let mut config = MCPConfig::default();
/// config.timeout = 10000; // 10 seconds
///
/// let factory = MCPFactory::with_config(config);
/// ```
#[derive(Debug)]
pub struct MCPFactory {
    /// Configuration for creating MCP instances
    config: MCPConfig,
}

impl MCPFactory {
    /// Creates a new `MCPFactory` with default configuration.
    ///
    /// This method initializes a factory with the system's default
    /// MCP configuration settings. Use this when you want to create
    /// instances with standard settings.
    ///
    /// # Returns
    ///
    /// A new factory instance with default configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_mcp::factory::MCPFactory;
    ///
    /// let factory = MCPFactory::new();
    /// let mcp = factory.create_mcp();
    /// ```
    #[must_use] pub fn new() -> Self {
        Self {
            config: MCPConfig::default(),
        }
    }
    
    /// Creates a new `MCPFactory` with custom configuration.
    ///
    /// This method allows you to initialize a factory with specific
    /// configuration settings that differ from the defaults. Use this
    /// when you need to customize the behavior of created MCP instances.
    ///
    /// # Arguments
    ///
    /// * `config` - Custom configuration for MCP instances
    ///
    /// # Returns
    ///
    /// A new factory instance with the specified configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_mcp::{MCPConfig, factory::MCPFactory};
    ///
    /// let mut config = MCPConfig::default();
    /// config.timeout = 10000; // 10 seconds
    ///
    /// let factory = MCPFactory::with_config(config);
    /// ```
    #[must_use] pub const fn with_config(config: MCPConfig) -> Self {
        Self { config }
    }
    
    /// Creates a new MCP instance using this factory's configuration.
    ///
    /// This method instantiates a new MCP adapter object configured according to
    /// the factory's settings. The instance is thread-safe and can be
    /// shared across multiple threads.
    ///
    /// # Returns
    ///
    /// A new thread-safe MCP adapter instance wrapped in an `Arc`.
    ///
    /// # Thread Safety
    ///
    /// The returned MCP instance is wrapped in an `Arc`, making it
    /// safe to clone and share across multiple threads.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use squirrel_mcp::factory::MCPFactory;
    /// use std::sync::Arc;
    ///
    /// let factory = MCPFactory::new();
    /// let mcp = factory.create_mcp();
    ///
    /// // The MCP instance can be cloned and shared
    /// let mcp_clone = Arc::clone(&mcp);
    /// ```
    #[must_use] pub fn create_mcp(&self) -> Arc<dyn MCPInterface + Send + Sync> {
        Arc::new(MCPAdapter::new(self.config.clone()))
    }
}

impl Default for MCPFactory {
    /// Creates a new `MCPFactory` with default settings.
    ///
    /// This implementation of the `Default` trait simply calls `MCPFactory::new()`.
    /// It provides a convenient way to create a factory with default configuration.
    ///
    /// # Returns
    ///
    /// A new factory instance with default configuration.
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new MCP factory with default configuration.
///
/// This is a convenience function that creates a new `MCPFactory` with
/// the default settings. It provides a simple way to obtain a factory
/// without explicitly constructing it.
///
/// # Returns
///
/// A new `MCPFactory` instance with default configuration.
///
/// # Examples
///
/// ```
/// use squirrel_mcp::factory::create_mcp_factory;
///
/// let factory = create_mcp_factory();
/// let mcp = factory.create_mcp();
/// ```
#[must_use] pub fn create_mcp_factory() -> MCPFactory {
    MCPFactory::new()
}

/// Creates a new MCP instance with default configuration.
///
/// This is a convenience function that creates a new `MCPAdapter` instance with
/// the default settings. It handles the creation of the factory and the
/// instance in a single call, simplifying the most common use case.
///
/// # Returns
///
/// A new thread-safe MCP instance wrapped in an `Arc`.
///
/// # Thread Safety
///
/// The returned MCP instance is wrapped in an `Arc`, making it
/// safe to clone and share across multiple threads.
///
/// # Examples
///
/// ```no_run
/// use squirrel_mcp::factory::create_mcp;
///
/// // Get an MCP instance directly without creating a factory
/// let mcp = create_mcp();
///
/// // Use the MCP instance
/// let result = mcp.initialize();
/// ```
#[must_use] pub fn create_mcp() -> Arc<dyn MCPInterface + Send + Sync> {
    let factory = create_mcp_factory();
    factory.create_mcp()
} 