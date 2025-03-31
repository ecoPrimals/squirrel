use crate::plugins::interfaces::{Plugin, PluginMetadata};
use crate::security::{RBACManager, SecurityConfig, SecurityError};
use crate::error::Result;
use std::path::Path;
use tracing::{debug, info};

// ... existing code ... 