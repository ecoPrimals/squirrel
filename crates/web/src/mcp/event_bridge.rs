use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use serde_json::json;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::mcp::client::McpClient;
use crate::mcp::context::ContextManager;
use crate::mcp::error::McpError;
use crate::mcp::types::McpMessage;
use crate::websocket::ConnectionManager;

/// MCP Event Bridge - Bridges events from MCP to WebSocket
pub struct McpEventBridge {
    /// MCP client
    mcp_client: Arc<dyn McpClient>,
    
    /// WebSocket manager
    ws_manager: Arc<ConnectionManager>,
    
    /// Event handlers by event type
    event_handlers: Arc<RwLock<HashMap<String, Vec<Box<dyn Fn(serde_json::Value) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> + Send + Sync + 'static>>>>>,
    
    /// Running flag
    running: Arc<RwLock<bool>>,
    
    /// Event subscriptions
    subscriptions: Arc<RwLock<HashMap<String, String>>>,
    
    /// Context manager
    context_manager: Arc<ContextManager>,
}

impl McpEventBridge {
    /// Create a new MCP event bridge
    pub fn new(
        mcp_client: Arc<dyn McpClient>,
        ws_manager: Arc<ConnectionManager>,
        context_manager: Arc<ContextManager>,
    ) -> Self {
        Self {
            mcp_client,
            ws_manager,
            event_handlers: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            context_manager,
        }
    }
    
    /// Start the event bridge
    pub async fn start(&self) -> Result<(), McpError> {
        // Set running flag
        {
            let mut running = self.running.write().await;
            *running = true;
        }
        
        // Clone references for the task
        let ws_manager = self.ws_manager.clone();
        let event_handlers = self.event_handlers.clone();
        let running = self.running.clone();
        let mcp_client = self.mcp_client.clone();
        let context_manager = self.context_manager.clone();
        
        // Create subscription for common event types
        self.subscribe_to_events(vec![
            "command.status".to_string(),
            "command.progress".to_string(),
            "command.result".to_string(),
            "command.error".to_string(),
            "system.status".to_string(),
        ]).await?;
        
        // Spawn a task to handle events
        tokio::spawn(async move {
            info!("MCP event bridge started");
            
            // Create a channel for receiving events
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);
            
            // Create event handler
            let tx_clone = tx.clone();
            let _event_handler = move |event: serde_json::Value| {
                let tx = tx_clone.clone();
                async move {
                    if let Err(e) = tx.send(event).await {
                        error!("Failed to send event to channel: {}", e);
                    }
                }
            };
            
            // Register handler with MCP client (implementation depends on the MCP client interface)
            // This is a placeholder - actual implementation would depend on how the MCP client handles subscriptions
            if let Ok(mcp_message) = serde_json::to_string(&McpMessage {
                type_: "subscribe".to_string(),
                payload: json!({
                    "events": [
                        "command.status",
                        "command.progress",
                        "command.result",
                        "command.error",
                        "system.status"
                    ]
                }),
                context: context_manager.create_context(
                    "system".to_string(),
                    Some("event_bridge".to_string()),
                    Some("web".to_string()),
                    None,
                    None,
                ).await,
            }) {
                // Send subscription message to MCP
                if let Err(e) = mcp_client.send_message(&mcp_message).await {
                    error!("Failed to subscribe to MCP events: {}", e);
                }
            }
            
            // Process events from the channel
            while *running.read().await {
                if let Some(event_data) = rx.recv().await {
                    // Extract event type and data
                    let event_type = event_data.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
                    
                    // Forward to WebSocket manager
                    if let Err(e) = ws_manager.handle_mcp_event(event_type, event_data.clone()).await {
                        error!("Failed to handle MCP event: {}", e);
                    }
                    
                    // Call registered event handlers
                    if let Some(handlers) = event_handlers.read().await.get(event_type) {
                        for handler in handlers {
                            handler(event_data.clone()).await;
                        }
                    }
                }
            }
            
            info!("MCP event bridge stopped");
        });
        
        Ok(())
    }
    
    /// Stop the event bridge
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        
        // Unsubscribe from events
        self.unsubscribe_from_all_events().await.ok();
    }
    
    /// Register an event handler
    pub async fn register_event_handler<F, Fut>(
        &self,
        event_type: &str,
        handler: F,
    ) where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let mut event_handlers = self.event_handlers.write().await;
        let handlers = event_handlers.entry(event_type.to_string()).or_insert_with(Vec::new);
        
        // Create wrapper function
        let wrapper = move |data: serde_json::Value| -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
            Box::pin(handler(data))
        };
        
        handlers.push(Box::new(wrapper));
    }
    
    /// Subscribe to MCP events
    pub async fn subscribe_to_events(&self, event_types: Vec<String>) -> Result<(), McpError> {
        let mut subscriptions = self.subscriptions.write().await;
        let context = self.context_manager.create_context(
            "system".to_string(),
            Some("event_bridge".to_string()),
            Some("web".to_string()),
            None,
            None,
        ).await;
        
        let mcp_message = McpMessage {
            type_: "subscribe".to_string(),
            payload: json!({
                "events": event_types
            }),
            context,
        };
        
        // Send subscription message to MCP
        let message_str = serde_json::to_string(&mcp_message)?;
        let response = self.mcp_client.send_message(&message_str).await?;
        
        // Parse response
        let response_json: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| McpError::InvalidResponse(format!("Failed to parse response: {}", e)))?;
        
        // Check for success
        let success = response_json.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
        if !success {
            return Err(McpError::CommandError("Failed to subscribe to events".to_string()));
        }
        
        // Extract subscription ID
        let subscription_id = response_json.get("subscription_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidResponse("Missing subscription_id".to_string()))?;
        
        // Store subscription
        for event_type in event_types {
            subscriptions.insert(event_type, subscription_id.to_string());
        }
        
        Ok(())
    }
    
    /// Unsubscribe from all events
    pub async fn unsubscribe_from_all_events(&self) -> Result<(), McpError> {
        let subscriptions = self.subscriptions.read().await;
        let mut unique_subscription_ids = HashSet::new();
        
        // Collect unique subscription IDs
        for (_, subscription_id) in subscriptions.iter() {
            unique_subscription_ids.insert(subscription_id.clone());
        }
        
        // Unsubscribe from each subscription
        for subscription_id in unique_subscription_ids {
            let context = self.context_manager.create_context(
                "system".to_string(),
                Some("event_bridge".to_string()),
                Some("web".to_string()),
                None,
                None,
            ).await;
            
            let mcp_message = McpMessage {
                type_: "unsubscribe".to_string(),
                payload: json!({
                    "subscription_id": subscription_id
                }),
                context,
            };
            
            // Send unsubscription message to MCP
            let message_str = serde_json::to_string(&mcp_message)?;
            self.mcp_client.send_message(&message_str).await?;
        }
        
        // Clear subscriptions
        self.subscriptions.write().await.clear();
        
        Ok(())
    }
} 