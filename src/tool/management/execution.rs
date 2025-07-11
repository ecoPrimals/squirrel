use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use async_trait::async_trait;
use chrono::Utc;
use tracing::{info, error, warn};

use super::types::*;
use super::ToolManager; // Import from parent module
use crate::tool::cleanup::RecoveryHook; 

impl ToolManager {
    async fn execute_recovery_hook(
        &self,
        tool_id: &str,
        hook: RecoveryHook,
    ) -> Result<(), ToolError> {
        match hook.execute() {
            Ok(()) => {
                info!("Recovery hook succeeded for tool {}", tool_id);
                // Recovery successful, activate the tool
                self.update_tool_state(tool_id, ToolState::Active).await
            }
            Err(e) => {
                error!("Recovery hook failed for tool {}: {:?}", tool_id, e);
                Err(ToolError::ExecutionFailed {
                    tool_id: tool_id.to_string(),
                    reason: "Recovery failed".to_string(),
                })
            }
        }
    }
} 