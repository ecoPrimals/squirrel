//! Dashboard component interface
//!
//! This module defines the interface for dashboard components.

use std::fmt::Debug;
use async_trait::async_trait;
use serde_json::Value;
use chrono::{DateTime, Utc};
use squirrel_core::error::Result;

/// Update type for dashboard components
#[derive(Debug, Clone)]
pub struct Update {
    /// Component ID
    pub component_id: String,
    /// Updated data
    pub data: Value,
    /// Timestamp of the update
    pub timestamp: DateTime<Utc>,
}

/// Dashboard component interface
///
/// This trait defines the interface for components that can be displayed
/// on the dashboard.
#[async_trait]
pub trait DashboardComponent: Debug + Send + Sync {
    /// Get the component ID
    fn id(&self) -> &str;
    
    /// Start the component
    ///
    /// This method is called when the component is registered with
    /// the dashboard manager.
    async fn start(&self) -> Result<()>;
    
    /// Get the component data
    ///
    /// This method returns the current data for the component.
    async fn get_data(&self) -> Result<Value>;
    
    /// Get the last update timestamp
    ///
    /// This method returns the timestamp of the last update.
    async fn last_update(&self) -> Option<DateTime<Utc>>;
    
    /// Get an update for the component
    ///
    /// This method is called to get an update for the component.
    /// If no update is available, it returns None.
    async fn get_update(&self) -> Result<Update>;
    
    /// Handle an event for this component.
    ///
    /// By default, this method does nothing. Override to implement event handling.
    async fn handle_event(&self, _event: Value) -> Result<()> {
        Ok(())
    }
    
    /// Stop the component
    ///
    /// This method is called when the component is stopped.
    async fn stop(&self) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }
} 