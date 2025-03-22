// Dashboard module for monitoring system
//
// This module provides functionality for:
// - Real-time metrics visualization
// - Health status display
// - Alert management interface
// - Performance graphs
// - Resource usage charts
// - Custom dashboards
// - Data visualization
// - Interactive controls

use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use tokio::sync::RwLock;
use std::time::Duration;
use time::OffsetDateTime;
use crate::health::{HealthChecker, HealthConfig, status::HealthStatus};
use crate::metrics::{performance::OperationType, MetricCollector, MetricConfig, Metric};
use crate::alerts::{AlertSeverity, AlertStatus, Alert, AlertConfig, AlertManager};
use squirrel_core::error::{Result, SquirrelError};
use serde_json::{Value, json};
use tracing::{info, error, debug};

/// Module for adapter implementations of dashboard functionality 