//! Plugin system for the web interface
//!
//! This module provides the plugin system for the web interface, allowing
//! extensions to be loaded and integrated with the web application.

pub mod core;
pub mod registry;
pub mod model;
pub mod adapter;
pub mod example;

pub use self::core::{Plugin, PluginMetadata, PluginStatus};
pub use self::model::{WebPlugin, WebEndpoint, WebComponent, WebRequest, WebResponse, HttpMethod, ComponentType, HttpStatus};
pub use self::registry::WebPluginRegistry;
pub use self::adapter::{LegacyWebPluginAdapter, NewWebPluginAdapter}; 