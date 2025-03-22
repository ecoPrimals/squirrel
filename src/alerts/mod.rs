/// Module for monitoring alert functionality
///
/// This module provides alert generation, management, and notification capabilities
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use squirrel_core::error::{Result, SquirrelError};
use std::collections::HashMap;
use std::fmt::{Debug};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing;

/// Module for alert configuration
pub mod config;

/// Module for alert manager implementations
pub mod manager;

/// Module for alert status tracking
pub mod status;

/// Module for alert adapters
pub mod adapter;

/// Module for notification management
pub mod notify;

use std::time::Duration;
use thiserror::Error;
use tracing::{info, warn, error, debug}; 