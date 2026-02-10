// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Dashboard UI structures and theming

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Dashboard theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTheme {
    /// Primary color
    pub primary_color: String,
    
    /// Secondary color
    pub secondary_color: String,
    
    /// Background color
    pub background_color: String,
    
    /// Text color
    pub text_color: String,
    
    /// Font family
    pub font_family: String,
    
    /// Font size
    pub font_size: u16,
    
    /// Dark mode enabled
    pub dark_mode: bool,
}

impl Default for DashboardTheme {
    fn default() -> Self {
        Self {
            primary_color: "#007bff".to_string(),
            secondary_color: "#6c757d".to_string(),
            background_color: "#ffffff".to_string(),
            text_color: "#212529".to_string(),
            font_family: "Arial, sans-serif".to_string(),
            font_size: 14,
            dark_mode: false,
        }
    }
}

/// Dashboard state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardState {
    /// Current status
    pub status: DashboardStatus,
    
    /// Last update timestamp
    pub last_update: Option<DateTime<Utc>>,
    
    /// Active connections count
    pub active_connections: usize,
    
    /// Uptime
    pub uptime: std::time::Duration,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            status: DashboardStatus::Starting,
            last_update: None,
            active_connections: 0,
            uptime: std::time::Duration::from_secs(0),
        }
    }
}

/// Dashboard status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashboardStatus {
    /// Starting up
    Starting,
    /// Active and operational
    Active,
    /// Paused
    Paused,
    /// Stopped
    Stopped,
    /// Error state
    Error,
}

impl Default for DashboardStatus {
    fn default() -> Self {
        Self::Starting
    }
} 