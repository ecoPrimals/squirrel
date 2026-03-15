// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Composite Lifecycle Hook Implementation
//!
//! This module provides a composite implementation that can combine multiple
//! lifecycle hooks and execute them in sequence.

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, error};

use crate::tool::{Tool, ToolError, ToolLifecycleHook};

/// Composite lifecycle hook that executes multiple hooks in sequence
#[derive(Debug)]
pub struct CompositeLifecycleHook {
    /// The hooks to execute
    hooks: Vec<Arc<dyn ToolLifecycleHook + Send + Sync>>,
}

impl CompositeLifecycleHook {
    /// Create a new composite lifecycle hook
    #[must_use] 
    pub fn new() -> Self {
        Self {
            hooks: Vec::new(),
        }
    }

    /// Add a hook to the composite
    pub fn add_hook<H>(&mut self, hook: H)
    where
        H: ToolLifecycleHook + Send + Sync + 'static,
    {
        self.hooks.push(Arc::new(hook));
    }

    /// Create a composite hook with the given hooks
    pub fn with_hooks<I, H>(hooks: I) -> Self
    where
        I: IntoIterator<Item = H>,
        H: ToolLifecycleHook + Send + Sync + 'static,
    {
        let mut composite = Self::new();
        for hook in hooks {
            composite.add_hook(hook);
        }
        composite
    }

    /// Execute all hooks for a given operation, stopping on first error
    async fn execute_hooks<F, Fut>(&self, operation_name: &str, operation: F) -> Result<(), ToolError>
    where
        F: Fn(Arc<dyn ToolLifecycleHook + Send + Sync>) -> Fut,
        Fut: std::future::Future<Output = Result<(), ToolError>>,
    {
        for (index, hook) in self.hooks.iter().enumerate() {
            if let Err(e) = operation(hook.clone()).await {
                error!(
                    "Hook {} failed during {}: {}",
                    index, operation_name, e
                );
                return Err(e);
            }
        }
        Ok(())
    }

    /// Get this hook as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Default for CompositeLifecycleHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolLifecycleHook for CompositeLifecycleHook {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn on_register(&self, tool: &Tool) -> Result<(), ToolError> {
        debug!("Executing composite on_register for tool: {}", tool.id);
        self.execute_hooks("on_register", |hook| {
            let tool = tool.clone();
            async move { hook.on_register(&tool).await }
        }).await
    }

    async fn on_unregister(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Executing composite on_unregister for tool: {}", tool_id);
        let tool_id = tool_id.to_string();
        self.execute_hooks("on_unregister", |hook| {
            let tool_id = tool_id.clone();
            async move { hook.on_unregister(&tool_id).await }
        }).await
    }

    async fn on_activate(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Executing composite on_activate for tool: {}", tool_id);
        let tool_id = tool_id.to_string();
        self.execute_hooks("on_activate", |hook| {
            let tool_id = tool_id.clone();
            async move { hook.on_activate(&tool_id).await }
        }).await
    }

    async fn on_deactivate(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Executing composite on_deactivate for tool: {}", tool_id);
        let tool_id = tool_id.to_string();
        self.execute_hooks("on_deactivate", |hook| {
            let tool_id = tool_id.clone();
            async move { hook.on_deactivate(&tool_id).await }
        }).await
    }

    async fn on_error(&self, tool_id: &str, error: &ToolError) -> Result<(), ToolError> {
        debug!("Executing composite on_error for tool: {}", tool_id);
        let tool_id = tool_id.to_string();
        
        // For error handling, we continue even if some hooks fail
        for (index, hook) in self.hooks.iter().enumerate() {
            if let Err(e) = hook.on_error(&tool_id, error).await {
                error!(
                    "Hook {} failed during on_error for tool {}: {}",
                    index, tool_id, e
                );
                // Continue with other hooks even if one fails
            }
        }
        Ok(())
    }

    async fn pre_start(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Executing composite pre_start for tool: {}", tool_id);
        let tool_id = tool_id.to_string();
        self.execute_hooks("pre_start", |hook| {
            let tool_id = tool_id.clone();
            async move { hook.pre_start(&tool_id).await }
        }).await
    }

    async fn post_start(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Executing composite post_start for tool: {}", tool_id);
        let tool_id = tool_id.to_string();
        
        // For post operations, we try to execute all hooks even if some fail
        let mut last_error = None;
        for (index, hook) in self.hooks.iter().enumerate() {
            if let Err(e) = hook.post_start(&tool_id).await {
                error!(
                    "Hook {} failed during post_start for tool {}: {}",
                    index, tool_id, e
                );
                last_error = Some(e);
            }
        }
        
        if let Some(error) = last_error {
            Err(error)
        } else {
            Ok(())
        }
    }

    async fn pre_stop(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Executing composite pre_stop for tool: {}", tool_id);
        let tool_id = tool_id.to_string();
        self.execute_hooks("pre_stop", |hook| {
            let tool_id = tool_id.clone();
            async move { hook.pre_stop(&tool_id).await }
        }).await
    }

    async fn post_stop(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Executing composite post_stop for tool: {}", tool_id);
        let tool_id = tool_id.to_string();
        
        // For post operations, we try to execute all hooks even if some fail
        let mut last_error = None;
        for (index, hook) in self.hooks.iter().enumerate() {
            if let Err(e) = hook.post_stop(&tool_id).await {
                error!(
                    "Hook {} failed during post_stop for tool {}: {}",
                    index, tool_id, e
                );
                last_error = Some(e);
            }
        }
        
        if let Some(error) = last_error {
            Err(error)
        } else {
            Ok(())
        }
    }

    async fn on_pause(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Executing composite on_pause for tool: {}", tool_id);
        let tool_id = tool_id.to_string();
        self.execute_hooks("on_pause", |hook| {
            let tool_id = tool_id.clone();
            async move { hook.on_pause(&tool_id).await }
        }).await
    }

    async fn on_resume(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Executing composite on_resume for tool: {}", tool_id);
        let tool_id = tool_id.to_string();
        self.execute_hooks("on_resume", |hook| {
            let tool_id = tool_id.clone();
            async move { hook.on_resume(&tool_id).await }
        }).await
    }

    async fn on_update(&self, tool: &Tool) -> Result<(), ToolError> {
        debug!("Executing composite on_update for tool: {}", tool.id);
        self.execute_hooks("on_update", |hook| {
            let tool = tool.clone();
            async move { hook.on_update(&tool).await }
        }).await
    }

    async fn on_cleanup(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Executing composite on_cleanup for tool: {}", tool_id);
        let tool_id = tool_id.to_string();
        
        // For cleanup, we try to execute all hooks even if some fail
        let mut last_error = None;
        for (index, hook) in self.hooks.iter().enumerate() {
            if let Err(e) = hook.on_cleanup(&tool_id).await {
                error!(
                    "Hook {} failed during on_cleanup for tool {}: {}",
                    index, tool_id, e
                );
                last_error = Some(e);
            }
        }
        
        if let Some(error) = last_error {
            Err(error)
        } else {
            Ok(())
        }
    }

    async fn register_tool(&self, tool: &Tool) -> Result<(), ToolError> {
        self.on_register(tool).await
    }

    async fn initialize_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Executing composite initialize_tool for tool: {}", tool_id);
        let tool_id = tool_id.to_string();
        self.execute_hooks("initialize_tool", |hook| {
            let tool_id = tool_id.clone();
            async move { hook.initialize_tool(&tool_id).await }
        }).await
    }

    async fn pre_execute(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Executing composite pre_execute for tool: {}", tool_id);
        let tool_id = tool_id.to_string();
        self.execute_hooks("pre_execute", |hook| {
            let tool_id = tool_id.clone();
            async move { hook.pre_execute(&tool_id).await }
        }).await
    }

    async fn post_execute(
        &self,
        tool_id: &str,
        result: Result<(), ToolError>,
    ) -> Result<(), ToolError> {
        debug!("Executing composite post_execute for tool: {}", tool_id);
        let tool_id = tool_id.to_string();
        
        // For post operations, we try to execute all hooks even if some fail
        let mut last_error = None;
        for (index, hook) in self.hooks.iter().enumerate() {
            if let Err(e) = hook.post_execute(&tool_id, result.clone()).await {
                error!(
                    "Hook {} failed during post_execute for tool {}: {}",
                    index, tool_id, e
                );
                last_error = Some(e);
            }
        }
        
        if let Some(error) = last_error {
            Err(error)
        } else {
            Ok(())
        }
    }

    async fn reset_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Executing composite reset_tool for tool: {}", tool_id);
        let tool_id = tool_id.to_string();
        self.execute_hooks("reset_tool", |hook| {
            let tool_id = tool_id.clone();
            async move { hook.reset_tool(&tool_id).await }
        }).await
    }

    async fn cleanup_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        self.on_cleanup(tool_id).await
    }
} 