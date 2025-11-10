use std::future::Future;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument, warn};

use crate::tool::management::types::{Tool, ToolError, ToolLifecycleHook};
use super::resource_manager::ResourceManager;
use super::{ResourceLimits};

/// A hook for cleaning up tool resources
pub trait CleanupHook: Send + Sync {
    /// Clean up a tool's resources
    fn cleanup_tool(&self, tool_id: &str) -> impl Future<Output = Result<(), ToolError>> + Send;

    /// Register a new tool for cleanup tracking
    fn register_tool(&self, tool: &Tool) -> impl Future<Output = Result<(), ToolError>> + Send;

    /// Reset a tool's resources
    fn reset_tool(&self, tool_id: &str) -> impl Future<Output = Result<(), ToolError>> + Send;
}

/// Simple implementation of tool cleanup hook
#[derive(Debug)]
pub struct BasicCleanupHook {
    /// Track when tools were registered
    registration_times: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    /// Track when tools were last cleaned up
    cleanup_times: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    /// Resource manager
    resource_manager: Arc<dyn ResourceManager>,
}

impl BasicCleanupHook {
    /// Create a new basic cleanup hook
    pub fn new(resource_manager: Arc<dyn ResourceManager>) -> Self {
        Self {
            registration_times: Arc::new(RwLock::new(HashMap::new())),
            cleanup_times: Arc::new(RwLock::new(HashMap::new())),
            resource_manager,
        }
    }
}

impl CleanupHook for BasicCleanupHook {
    #[instrument(skip(self))]
    fn register_tool(&self, tool: &Tool) -> impl Future<Output = Result<(), ToolError>> + Send {
        let tool_id = tool.id.clone();
        let registration_times = self.registration_times.clone();
        let resource_manager = self.resource_manager.clone();
        
        async move {
            // Check if the tool is already registered
            {
                let registrations = registration_times.read().await;
                if registrations.contains_key(&tool_id) {
                    return Err(ToolError::RegistrationFailed(format!(
                        "Tool '{tool_id}' is already registered"
                    )));
                }
            }

            // Record registration time
            {
                let mut registrations = registration_times.write().await;
                registrations.insert(tool_id.clone(), Utc::now());
            }

            // Initialize resources
            resource_manager
                .initialize_tool(
                    &tool_id,
                    ResourceLimits::default(),
                    ResourceLimits::default(),
                )
                .await?;

            info!("Registered tool {} with cleanup hook", tool_id);
            Ok(())
        }
    }

    #[instrument(skip(self))]
    fn cleanup_tool(&self, tool_id: &str) -> impl Future<Output = Result<(), ToolError>> + Send {
        let tool_id = tool_id.to_string();
        let registration_times = self.registration_times.clone();
        let cleanup_times = self.cleanup_times.clone();
        let resource_manager = self.resource_manager.clone();
        
        async move {
            // Check if tool is registered
            {
                let registrations = registration_times.read().await;
                if !registrations.contains_key(&tool_id) {
                    return Err(ToolError::ToolNotFound(tool_id.to_string()));
                }
            }

            // Clean up resources
            resource_manager.cleanup_tool(&tool_id).await?;

            // Record cleanup time
            {
                let mut cleanups = cleanup_times.write().await;
                cleanups.insert(tool_id.clone(), Utc::now());
            }

            // Remove registration
            {
                let mut registrations = registration_times.write().await;
                registrations.remove(&tool_id);
            }

            info!("Cleaned up resources for tool {}", tool_id);
            Ok(())
        }
    }

    #[instrument(skip(self))]
    fn reset_tool(&self, tool_id: &str) -> impl Future<Output = Result<(), ToolError>> + Send {
        let tool_id = tool_id.to_string();
        let registration_times = self.registration_times.clone();
        let resource_manager = self.resource_manager.clone();
        
        async move {
            // Check if tool is registered
            {
                let registrations = registration_times.read().await;
                if !registrations.contains_key(&tool_id) {
                    return Err(ToolError::ToolNotFound(tool_id.to_string()));
                }
            }

            // Reset resources
            resource_manager.reset_tool(&tool_id).await?;

            info!("Reset resources for tool {}", tool_id);
            Ok(())
        }
    }
}

#[async_trait]
impl ToolLifecycleHook for BasicCleanupHook {
    async fn on_register(&self, tool: &Tool) -> Result<(), ToolError> {
        <Self as CleanupHook>::register_tool(self, tool).await
    }

    async fn on_unregister(&self, tool_id: &str) -> Result<(), ToolError> {
        <Self as CleanupHook>::cleanup_tool(self, tool_id).await
    }

    async fn on_activate(&self, _tool_id: &str) -> Result<(), ToolError> {
        // Default implementation does nothing
        Ok(())
    }

    async fn on_deactivate(&self, _tool_id: &str) -> Result<(), ToolError> {
        // Default implementation does nothing
        Ok(())
    }

    async fn on_error(&self, tool_id: &str, error: &ToolError) -> Result<(), ToolError> {
        // Log the error but don't take action
        warn!("Tool {} encountered error: {:?}", tool_id, error);
        Ok(())
    }

    async fn on_cleanup(&self, tool_id: &str) -> Result<(), ToolError> {
        <Self as CleanupHook>::cleanup_tool(self, tool_id).await
    }

    async fn pre_start(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn post_start(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn pre_stop(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn post_stop(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn on_pause(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn on_resume(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn on_update(&self, _tool: &Tool) -> Result<(), ToolError> {
        Ok(())
    }

    async fn initialize_tool(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn pre_execute(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    async fn post_execute(
        &self,
        _tool_id: &str,
        result: Result<(), ToolError>,
    ) -> Result<(), ToolError> {
        result
    }

    async fn reset_tool(&self, _tool_id: &str) -> Result<(), ToolError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
