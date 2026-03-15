// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Dashboard integration for observability framework
//!
//! This module provides the dashboard client for sending observability data
//! to external dashboards and monitoring systems.

use std::sync::Arc;
use serde_json;
use reqwest;

use crate::observability::{ObservabilityError, ObservabilityResult};
use crate::observability::metrics::MetricSnapshot;
use crate::observability::tracing::SpanSnapshot;
use crate::observability::health::SystemHealthReport;
use crate::observability::alerting::Alert;

/// Client for communicating with external dashboards
pub struct DashboardClient {
    url: String,
    auth_token: Option<String>,
    client: reqwest::Client,
}

impl DashboardClient {
    /// Create a new dashboard client
    pub async fn new(url: &str, auth_token: Option<String>) -> ObservabilityResult<Self> {
        Ok(Self {
            url: url.to_string(),
            auth_token,
            client: reqwest::Client::new(),
        })
    }

    /// Send metrics to the dashboard
    pub async fn send_metrics(&self, metrics: Vec<MetricSnapshot>) -> ObservabilityResult<()> {
        let payload = serde_json::to_value(metrics)?;
        self.send_data("metrics", payload).await
    }

    /// Send traces to the dashboard
    pub async fn send_traces(&self, traces: Vec<SpanSnapshot>) -> ObservabilityResult<()> {
        let payload = serde_json::to_value(traces)?;
        self.send_data("traces", payload).await
    }

    /// Send health reports to the dashboard
    pub async fn send_health_report(&self, health_report: SystemHealthReport) -> ObservabilityResult<()> {
        let payload = serde_json::to_value(health_report)?;
        self.send_data("health", payload).await
    }

    /// Send alerts to the dashboard
    pub async fn send_alerts(&self, alerts: Vec<Alert>) -> ObservabilityResult<()> {
        let payload = serde_json::to_value(alerts)?;
        self.send_data("alerts", payload).await
    }

    /// Send data to dashboard endpoint
    async fn send_data(&self, endpoint: &str, payload: serde_json::Value) -> ObservabilityResult<()> {
        let url = format!("{}/api/v1/{}", self.url, endpoint);
        
        let mut request = self.client.post(&url).json(&payload);
        
        if let Some(ref token) = self.auth_token {
            request = request.bearer_auth(token);
        }
        
        let response = request.send().await?;
        
        if !response.status().is_success() {
            return Err(ObservabilityError::DashboardError(
                format!("Dashboard request failed: {}", response.status())
            ));
        }
        
        Ok(())
    }

    /// Shutdown the dashboard client
    pub async fn shutdown(&self) -> ObservabilityResult<()> {
        // Graceful shutdown - send any remaining data
        Ok(())
    }
}

impl Clone for DashboardClient {
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            auth_token: self.auth_token.clone(),
            client: self.client.clone(),
        }
    }
} 