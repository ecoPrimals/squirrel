//! MCP Adapter module for dependency injection.
//!
//! This module provides an adapter implementation for the Machine Context Protocol (MCP)
//! system, enabling dependency injection and flexible integration with other components.
//! The adapter pattern allows for decoupling of the MCP implementation from its consumers,
//! making it easier to swap implementations or mock the MCP system for testing.
//!
//! # Key Components
//!
//! - `MCPInterface`: Trait defining the core MCP operations
//! - `MCPAdapter`: Concrete adapter implementation with thread-safe initialization
//! - Factory functions for creating adapters with various configurations
//!
//! # Examples
//!
//! Creating and using an MCP adapter:
//!
//! ```no_run
//! use squirrel_mcp::adapter::{create_default_mcp_adapter, MCPInterface};
//!
//! let adapter = create_default_mcp_adapter();
//! adapter.initialize().expect("Failed to initialize MCP");
//! let response = adapter.send_message("Hello, MCP!").expect("Failed to send message");
//! println!("Response: {}", response);
//! ```

use crate::config::McpConfig as MCPConfig;
use squirrel_interfaces::error::SquirrelError;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Interface for the MCP system.
///
/// This trait defines the core operations that the MCP system must support,
/// providing a stable API for consumers regardless of the underlying implementation.
/// Implementations of this trait handle the details of MCP communication, configuration,
/// and message passing.
pub trait MCPInterface: Send + Sync {
    /// Initialize the MCP system.
    ///
    /// This method sets up the MCP system and prepares it for use.
    /// It should be called before any other methods.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if initialization was successful
    /// - `Err(SquirrelError)` if initialization failed
    ///
    /// # Errors
    ///
    /// Returns an error if the MCP system cannot be initialized,
    /// such as if resources are unavailable or configuration is invalid.
    fn initialize(&self) -> Result<(), SquirrelError>;

    /// Check if the MCP system is initialized.
    ///
    /// # Returns
    ///
    /// `true` if the MCP system is initialized and ready for use, `false` otherwise.
    fn is_initialized(&self) -> bool;

    /// Get the MCP configuration.
    ///
    /// # Returns
    ///
    /// - `Ok(MCPConfig)` with the current configuration
    /// - `Err(SquirrelError)` if the configuration cannot be retrieved
    ///
    /// # Errors
    ///
    /// Returns an error if the MCP system is not initialized or
    /// if the configuration cannot be accessed.
    fn get_config(&self) -> Result<MCPConfig, SquirrelError>;

    /// Send a message through the MCP system.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// - `Ok(String)` with the response message
    /// - `Err(SquirrelError)` if the message cannot be sent
    ///
    /// # Errors
    ///
    /// Returns an error if the MCP system is not initialized,
    /// if the message format is invalid, or if the message cannot be delivered.
    fn send_message(&self, message: &str) -> Result<String, SquirrelError>;
}

/// Adapter for the MCP system.
///
/// This struct provides a thread-safe implementation of the `MCPInterface` trait,
/// managing the lifecycle of an underlying MCP implementation. It uses read-write
/// locks to ensure thread safety and provides a consistent API for MCP operations.
///
/// # Examples
///
/// ```no_run
/// use squirrel_mcp::adapter::MCPAdapter;
/// use squirrel_mcp::adapter::MCPInterface;
/// use squirrel_mcp::config::McpConfig;
///
/// let config = McpConfig::default();
/// let adapter = MCPAdapter::new(config);
/// // Check if initialized but don't actually initialize (to avoid runtime requirements)
/// let initialized = adapter.is_initialized();
/// ```
pub struct MCPAdapter {
    /// The inner MCP instance
    mcp: Arc<RwLock<Option<Arc<dyn MCPInterface + Send + Sync>>>>,
    /// Mutex to ensure thread-safe initialization
    init_mutex: RwLock<()>,
}

impl MCPAdapter {
    /// Create a new `MCPAdapter`.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the MCP system
    ///
    /// # Returns
    ///
    /// A new, uninitialized `MCPAdapter` instance
    #[must_use] pub fn new(_config: MCPConfig) -> Self {
        Self {
            mcp: Arc::new(RwLock::new(None)),
            init_mutex: RwLock::new(()),
        }
    }

    /// Create a new `MCPAdapter` that is already initialized.
    ///
    /// This method creates and initializes an `MCPAdapter` in a single step.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the MCP system
    ///
    /// # Returns
    ///
    /// - `Ok(MCPAdapter)` with an initialized adapter
    /// - `Err(SquirrelError)` if initialization fails
    ///
    /// # Errors
    ///
    /// Returns an error if the MCP system cannot be initialized.
    pub fn new_initialized(config: MCPConfig) -> Result<Self, SquirrelError> {
        let adapter = Self::new(config);
        adapter.initialize()?;
        Ok(adapter)
    }
}

impl MCPInterface for MCPAdapter {
    fn initialize(&self) -> Result<(), SquirrelError> {
        // Since this needs to be synchronous, we'll use block_in_place to execute the async code
        tokio::task::block_in_place(|| {
            let runtime = tokio::runtime::Handle::current();
            runtime.block_on(async {
                let _lock = self.init_mutex.read().await;

                // Check if an implementation already exists
                let impl_exists = {
                    let reader = self.mcp.read().await;
                    reader.is_some()
                };

                if impl_exists {
                    return Ok(());
                }

                // Initialize MCP implementation code here...
                Ok(())
            })
        })
    }

    fn is_initialized(&self) -> bool {
        tokio::task::block_in_place(|| {
            let runtime = tokio::runtime::Handle::current();
            runtime.block_on(async {
                let reader = self.mcp.read().await;
                reader.is_some()
            })
        })
    }

    fn get_config(&self) -> Result<MCPConfig, SquirrelError> {
        tokio::task::block_in_place(|| {
            let runtime = tokio::runtime::Handle::current();
            runtime.block_on(async {
                let reader = self.mcp.read().await;
                (*reader).as_ref().map_or_else(
                    || Err(SquirrelError::mcp(
                        "MCP is not initialized. Call initialize() first.",
                    )),
                    |mcp| mcp.get_config()
                )
            })
        })
    }

    fn send_message(&self, message: &str) -> Result<String, SquirrelError> {
        tokio::task::block_in_place(|| {
            let runtime = tokio::runtime::Handle::current();
            runtime.block_on(async {
                let reader = self.mcp.read().await;
                (*reader).as_ref().map_or_else(
                    || Err(SquirrelError::mcp(
                        "MCP is not initialized. Call initialize() first.",
                    )),
                    |mcp| mcp.send_message(message)
                )
            })
        })
    }
}

/// Create a new `MCPAdapter`.
///
/// # Arguments
///
/// * `config` - Configuration for the MCP system
///
/// # Returns
///
/// A new, uninitialized `MCPAdapter` instance
#[must_use] pub fn create_mcp_adapter(config: MCPConfig) -> MCPAdapter {
    MCPAdapter::new(config)
}

/// Create a new `MCPAdapter` with default configuration.
///
/// # Returns
///
/// A new, uninitialized `MCPAdapter` instance with default configuration
#[must_use] pub fn create_default_mcp_adapter() -> MCPAdapter {
    MCPAdapter::new(MCPConfig::default())
}

/// Create a new `MCPAdapter` that is already initialized.
///
/// # Arguments
///
/// * `config` - Configuration for the MCP system
///
/// # Returns
///
/// - `Ok(MCPAdapter)` with an initialized adapter
/// - `Err(SquirrelError)` if initialization fails
///
/// # Errors
///
/// Returns an error if the MCP system cannot be initialized.
pub fn create_initialized_mcp_adapter(config: MCPConfig) -> Result<MCPAdapter, SquirrelError> {
    MCPAdapter::new_initialized(config)
}

/// Create a new `MCPAdapter` with default configuration that is already initialized.
///
/// # Returns
///
/// - `Ok(MCPAdapter)` with an initialized adapter using default configuration
/// - `Err(SquirrelError)` if initialization fails
///
/// # Errors
///
/// Returns an error if the MCP system cannot be initialized.
pub fn create_default_initialized_mcp_adapter() -> Result<MCPAdapter, SquirrelError> {
    MCPAdapter::new_initialized(MCPConfig::default())
}
