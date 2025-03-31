use crate::plugins::interfaces::{Plugin, PluginMetadata, PluginCapability, PluginSource};
use crate::error::{Result, PluginError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::plugins::repository::PluginRepository;
use crate::plugins::management::PluginManager;
use tracing::{info, warn};

// ... existing code ... 