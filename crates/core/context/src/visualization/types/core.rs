// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Core visualization types and structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Visualization type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VisualizationType {
    /// Context state visualization
    ContextState,
    /// Rule dependency graph
    RuleDependencyGraph,
    /// Timeline visualization
    Timeline,
    /// Metrics dashboard
    MetricsDashboard,
    /// State diff visualization
    StateDiff,
    /// Performance heatmap
    PerformanceHeatmap,
    /// Interactive graph
    InteractiveGraph,
    /// Custom visualization
    Custom(String),
}

/// Visualization request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationRequest {
    /// Type of visualization
    pub visualization_type: VisualizationType,

    /// Visualization configuration
    pub config: super::VisualizationConfig,

    /// Data to visualize
    pub data: Value,

    /// Metadata for the visualization
    pub metadata: HashMap<String, Value>,

    /// Title for the visualization
    pub title: Option<String>,

    /// Description for the visualization
    pub description: Option<String>,
}

/// Visualization response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationResponse {
    /// Unique identifier for the visualization
    pub visualization_id: String,

    /// Type of visualization
    pub visualization_type: VisualizationType,

    /// Output format
    pub format: String,

    /// Rendered content
    pub content: String,

    /// Metadata
    pub metadata: HashMap<String, Value>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}
