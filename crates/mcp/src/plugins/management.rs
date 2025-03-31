use crate::plugins::interfaces::{Plugin, PluginMetadata, PluginState};
use crate::plugins::registry::PluginRegistry;
use crate::error::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use tracing::info;

// ... existing code ... 