//! # Squirrel Plugin SDK
//!
//! The Squirrel Plugin SDK provides a comprehensive set of tools and APIs for developing
//! plugins that integrate with the Squirrel MCP (Model Context Protocol) platform.
//!
//! ## Features
//!
//! - **Plugin Framework**: Complete plugin lifecycle management
//! - **MCP Integration**: Direct access to the Model Context Protocol
//! - **Command Registry**: Register and handle custom commands
//! - **Cross-Platform**: WASM-based plugins run anywhere
//! - **Web UI Support**: Build rich user interfaces for plugins
//! - **Secure Execution**: Sandbox security handled by ToadStool compute platform
//!
//! ## Quick Start
//!
//! ```rust
//! use squirrel_sdk::prelude::*;
//! use wasm_bindgen::prelude::*;
//!
//! #[wasm_bindgen]
//! pub struct MyPlugin {
//!     config: PluginConfig,
//! }
//!
//! #[wasm_bindgen]
//! impl MyPlugin {
//!     #[wasm_bindgen(constructor)]
//!     pub fn new() -> Result<MyPlugin, JsValue> {
//!         utils::set_panic_hook();
//!         Ok(MyPlugin {
//!             config: PluginConfig::default(),
//!         })
//!     }
//!
//!     #[wasm_bindgen]
//!     pub async fn handle_command(&self, command: &str, params: JsValue) -> Result<JsValue, JsValue> {
//!         // Handle plugin commands
//!         Ok(JsValue::NULL)
//!     }
//! }
//! ```

#![warn(missing_docs)]

use wasm_bindgen::prelude::*;

// Re-exports for convenience
pub use serde;
pub use serde_json::{self, json};

/// The version of the Squirrel SDK, extracted from the package version at compile time.
pub const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");
pub use wasm_bindgen;
pub use wasm_bindgen_futures;

// Core module system
pub mod client;
pub mod communication;
pub mod core;
pub mod infrastructure;

// Prelude for easy imports
pub mod prelude {
    //! Common imports for plugin development

    // Core framework
    pub use crate::core::*;

    // Communication systems
    pub use crate::communication::*;

    // Client APIs
    #[cfg(feature = "http")]
    pub use crate::client::http::*;

    #[cfg(feature = "fs")]
    pub use crate::client::fs::*;

    // Infrastructure
    pub use crate::infrastructure::*;

    // External crates
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::{json, Value as JsonValue};

    pub const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");
    pub use wasm_bindgen::prelude::*;
    pub use wasm_bindgen_futures::JsFuture;

    // Common types
    pub use js_sys::{Array, Date, Error as JsError, Function, Object, Promise};
    pub use web_sys::{console, window, Document};
}

// Re-export all modules for direct access
pub use client::*;
pub use communication::*;
pub use core::*;
pub use infrastructure::*;

// Utility functions
/// Set panic hook for better error messages in WASM
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Initialize the SDK with default configuration
pub fn init() -> Result<(), JsValue> {
    set_panic_hook();
    Ok(())
}

/// Get SDK version information
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Get SDK build information
/// Get enabled features as a Vec<String>
fn get_enabled_features() -> Vec<String> {
    let mut features = Vec::new();
    #[cfg(feature = "http")]
    features.push("http".to_string());
    #[cfg(feature = "fs")]
    features.push("fs".to_string());
    #[cfg(feature = "mcp")]
    features.push("mcp".to_string());
    #[cfg(feature = "console_error_panic_hook")]
    features.push("console_error_panic_hook".to_string());
    features
}

/// Retrieves comprehensive build information for the SDK.
///
/// This function returns a JSON object containing detailed information about the SDK build,
/// including version, build timestamp, git hash, target platform, build profile, and enabled features.
///
/// # Returns
/// A JSON object with the following structure:
/// - `version`: The SDK version string
/// - `build_timestamp`: When the SDK was built (if available)
/// - `git_hash`: The git commit hash (if available)
/// - `target`: The target platform
/// - `profile`: The build profile (debug/release)
/// - `features`: List of enabled cargo features
///
/// # Examples
/// ```
/// use squirrel_sdk::build_info;
///
/// let info = build_info();
/// println!("SDK Version: {}", info["version"]);
/// ```
pub fn build_info() -> serde_json::Value {
    json!({
        "version": version(),
        "build_timestamp": std::env::var("BUILD_TIMESTAMP").unwrap_or_else(|_| "unknown".to_string()),
        "git_hash": std::env::var("GIT_HASH").unwrap_or_else(|_| "unknown".to_string()),
        "target": std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string()),
        "profile": std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string()),
        "features": get_enabled_features()
    })
}

// Platform-specific initialization
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_init() {
    if let Err(e) = init() {
        web_sys::console::error_1(&format!("SDK initialization failed: {:?}", e).into());
    }
}

// SDK capabilities
/// Check if a feature is enabled
pub fn has_feature(feature: &str) -> bool {
    match feature {
        #[cfg(feature = "http")]
        "http" => true,
        #[cfg(feature = "fs")]
        "fs" => true,
        #[cfg(feature = "mcp")]
        "mcp" => true,
        #[cfg(feature = "console_error_panic_hook")]
        "console_error_panic_hook" => true,
        _ => false,
    }
}

/// Get list of enabled features
pub fn enabled_features() -> Vec<&'static str> {
    let mut features = Vec::new();

    #[cfg(feature = "http")]
    features.push("http");

    #[cfg(feature = "fs")]
    features.push("fs");

    #[cfg(feature = "mcp")]
    features.push("mcp");

    #[cfg(feature = "console_error_panic_hook")]
    features.push("console_error_panic_hook");

    features
}

/// Get SDK configuration
pub fn get_sdk_config() -> Result<PluginSdkConfig, PluginError> {
    let config = PluginSdkConfig::default();
    config
        .validate()
        .map_err(|e| PluginError::InitializationError {
            reason: format!("Failed to get SDK config: {}", e),
        })?;
    Ok(config)
}

/// Validate SDK environment
pub fn validate_environment() -> Result<(), PluginError> {
    // Check WASM environment
    #[cfg(target_arch = "wasm32")]
    {
        if web_sys::window().is_none() {
            return Err(PluginError::InitializationError {
                message: "SDK requires browser environment".to_string(),
            });
        }
    }

    // Check required features
    if !has_feature("mcp") {
        return Err(PluginError::InitializationError {
            reason: "MCP feature is required".to_string(),
        });
    }

    Ok(())
}

/// Create a new plugin instance
pub fn create_plugin<T: Plugin>(config: PluginConfig) -> Result<T, PluginError> {
    validate_environment()?;
    T::new(config)
}

/// Plugin trait for SDK compatibility
pub trait Plugin: Sized {
    /// Create a new plugin instance
    fn new(config: PluginConfig) -> Result<Self, PluginError>;

    /// Get plugin information
    fn info(&self) -> &PluginInfo;

    /// Initialize the plugin
    fn init(&mut self) -> Result<(), PluginError>;

    /// Start the plugin
    fn start(&mut self) -> Result<(), PluginError>;

    /// Stop the plugin
    fn stop(&mut self) -> Result<(), PluginError>;

    /// Handle a command
    fn handle_command(
        &mut self,
        command: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, PluginError>;

    /// Handle an event
    fn handle_event(&mut self, event: &Event) -> Result<(), PluginError>;

    /// Get plugin state
    fn get_state(&self) -> serde_json::Value;

    /// Set plugin state
    fn set_state(&mut self, state: serde_json::Value) -> Result<(), PluginError>;

    /// Cleanup resources
    fn cleanup(&mut self) -> Result<(), PluginError>;
}

// Re-export key types for convenience
pub use crate::communication::{CommandRegistry, EventBus, McpClient};
pub use crate::core::{BasePlugin, PluginInfo, PluginStats, WasmPlugin};
pub use crate::infrastructure::error::PluginResult;
pub use crate::infrastructure::{Logger, PluginConfig, PluginError};

#[cfg(feature = "http")]
pub use crate::client::http::HttpClient;

#[cfg(feature = "fs")]
pub use crate::client::fs::FileSystem;

// SDK metadata
/// SDK metadata information
pub struct SdkMetadata {
    /// SDK version
    pub version: &'static str,
    /// Build timestamp
    pub build_timestamp: String,
    /// Git hash
    pub git_hash: String,
    /// Target platform
    pub target: String,
    /// Build profile
    pub profile: String,
    /// Enabled features
    pub features: Vec<&'static str>,
}

/// Get SDK metadata
pub fn metadata() -> SdkMetadata {
    SdkMetadata {
        version: version(),
        build_timestamp: std::env::var("BUILD_TIMESTAMP").unwrap_or_else(|_| "unknown".to_string()),
        git_hash: std::env::var("GIT_HASH").unwrap_or_else(|_| "unknown".to_string()),
        target: std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string()),
        profile: std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string()),
        features: enabled_features(),
    }
}
/// Internal SDK utilities
pub mod internal {
    use crate::PluginResult;

    /// Initialize the plugin environment
    pub fn init_plugin_environment(_plugin_id: &str) -> PluginResult<()> {
        // Basic initialization - can be expanded later
        Ok(())
    }
}
