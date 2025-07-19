use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::tool::management::types::{Tool, ToolError, ToolLifecycleHook, ToolState};
use crate::tool::management::ToolManager; 