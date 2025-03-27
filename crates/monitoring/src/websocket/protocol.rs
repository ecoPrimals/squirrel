use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::websocket::messages::{ServerMessage, ClientAction, ClientMessage, error_codes};
use squirrel_core::error::{Result, SquirrelError};

/// WebSocket protocol handler for processing client messages
#[derive(Debug)]
pub struct WebSocketProtocol {
    /// Component data store
    component_data: Arc<RwLock<HashMap<String, Value>>>,
}

impl WebSocketProtocol {
    /// Create a new protocol handler
    pub fn new(component_data: Arc<RwLock<HashMap<String, Value>>>) -> Self {
        Self {
            component_data,
        }
    }
    
    /// Process a client message and generate a response
    pub async fn process_message(&self, message: &str) -> Result<Option<ServerMessage>> {
        let client_message: ClientMessage = match serde_json::from_str(message) {
            Ok(msg) => msg,
            Err(e) => {
                return Ok(Some(ServerMessage::error(
                    &format!("Invalid message format: {}", e),
                    error_codes::INVALID_REQUEST,
                    None,
                )));
            }
        };
        
        match client_message.action {
            // Subscribe/Unsubscribe actions don't produce immediate responses
            ClientAction::Subscribe | ClientAction::Unsubscribe => Ok(None),
            
            // Get data for a specific component
            ClientAction::GetData => {
                match client_message.topic {
                    Some(topic) => {
                        let data = self.component_data.read().await;
                        match data.get(&topic) {
                            Some(value) => Ok(Some(ServerMessage::component_update(&topic, value.clone()))),
                            None => Ok(Some(ServerMessage::error(
                                &format!("Component not found: {}", topic),
                                error_codes::NOT_FOUND,
                                client_message.request_id,
                            ))),
                        }
                    },
                    None => Ok(Some(ServerMessage::error(
                        "Missing topic parameter",
                        error_codes::INVALID_REQUEST,
                        client_message.request_id,
                    ))),
                }
            },
            
            // Get list of available components
            ClientAction::GetComponents => {
                let data = self.component_data.read().await;
                let components = data.keys().cloned().collect();
                Ok(Some(ServerMessage::ComponentsList {
                    components,
                    request_id: client_message.request_id,
                }))
            },
            
            // Get health status
            ClientAction::GetHealth => {
                // In a real implementation, this would check actual health metrics
                let status = serde_json::json!({
                    "status": "healthy",
                    "connections": 0, // Would be actual connection count
                    "uptime": 0,      // Would be actual uptime
                });
                
                Ok(Some(ServerMessage::HealthStatus {
                    status,
                    request_id: client_message.request_id,
                }))
            },
        }
    }
}

/// Documentation for dashboard and UI teams
/// 
/// ## WebSocket Connection Protocol
/// 
/// ### Connection URL
/// 
/// ```
/// ws://host:port
/// ```
/// 
/// ### Client Messages (to server)
/// 
/// #### Subscribe to Component Updates
/// 
/// ```json
/// {
///   "action": "subscribe",
///   "topic": "component_name",
///   "request_id": "optional-correlation-id"
/// }
/// ```
/// 
/// #### Unsubscribe from Component Updates
/// 
/// ```json
/// {
///   "action": "unsubscribe",
///   "topic": "component_name",
///   "request_id": "optional-correlation-id"
/// }
/// ```
/// 
/// #### Request Component Data
/// 
/// ```json
/// {
///   "action": "get_data",
///   "topic": "component_name",
///   "request_id": "optional-correlation-id"
/// }
/// ```
/// 
/// #### Request Available Components
/// 
/// ```json
/// {
///   "action": "get_components",
///   "request_id": "optional-correlation-id"
/// }
/// ```
/// 
/// #### Request Health Status
/// 
/// ```json
/// {
///   "action": "get_health",
///   "request_id": "optional-correlation-id"
/// }
/// ```
/// 
/// ### Server Messages (to client)
/// 
/// #### Component Update
/// 
/// ```json
/// {
///   "type": "component_update",
///   "component_id": "component_name",
///   "data": {
///     /* component-specific data */
///   }
/// }
/// ```
/// 
/// #### Components List
/// 
/// ```json
/// {
///   "type": "components_list",
///   "components": ["component1", "component2", "component3"],
///   "request_id": "correlation-id-from-request"
/// }
/// ```
/// 
/// #### Health Status
/// 
/// ```json
/// {
///   "type": "health_status",
///   "status": {
///     "status": "healthy",
///     "connections": 42,
///     "uptime": 3600
///   },
///   "request_id": "correlation-id-from-request"
/// }
/// ```
/// 
/// #### Error Response
/// 
/// ```json
/// {
///   "type": "error",
///   "message": "Error message",
///   "code": 404,
///   "request_id": "correlation-id-from-request"
/// }
/// ```
/// 
/// ### Error Codes
/// 
/// - 400: Invalid request format
/// - 401: Authentication required
/// - 404: Component not found
/// - 500: Server error
pub struct ProtocolDocumentation; 