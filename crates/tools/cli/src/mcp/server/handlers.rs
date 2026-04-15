// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Request/command handling for [`super::MCPServer`].

use serde_json::Value;
use std::collections::HashSet;
use std::io::Write;
use tracing::{debug, warn};

use crate::mcp::protocol::{MCPError, MCPMessage, MCPResult};

use super::MCPServer;
use super::safe_lock;

impl MCPServer {
    /// Extract the `"topic"` string from a request payload.
    fn extract_topic(request: &MCPMessage) -> MCPResult<String> {
        request
            .payload
            .as_ref()
            .and_then(|p| p.get("topic"))
            .and_then(|t| t.as_str())
            .map(String::from)
            .ok_or_else(|| {
                MCPError::ProtocolError(
                    "Missing or non-string 'topic' field in request payload".to_string(),
                )
            })
    }

    /// Handle subscribe request
    pub(super) fn handle_subscribe(
        &self,
        client_id: &str,
        request: MCPMessage,
    ) -> MCPResult<Option<String>> {
        let topic = Self::extract_topic(&request)?;
        self.subscribe_client(client_id, &topic)?;

        let response = MCPMessage::new_response(
            request.id,
            "subscribe".to_string(),
            Some(serde_json::json!({ "result": "ok", "topic": topic })),
        );
        Ok(Some(response.to_json()?))
    }

    /// Handle unsubscribe request
    pub(super) fn handle_unsubscribe(
        &self,
        client_id: &str,
        request: MCPMessage,
    ) -> MCPResult<Option<String>> {
        let topic = Self::extract_topic(&request)?;

        if topic == "*" {
            self.unsubscribe_client_all(client_id)?;
        } else {
            self.unsubscribe_client(client_id, &topic)?;
        }

        let response = MCPMessage::new_response(
            request.id,
            "unsubscribe".to_string(),
            Some(serde_json::json!({ "result": "ok", "topic": topic })),
        );
        Ok(Some(response.to_json()?))
    }

    /// Handle a command request
    pub(super) fn handle_command(&self, message: MCPMessage) -> MCPResult<MCPMessage> {
        let command_name = message.command.clone();

        // Check for custom handlers
        {
            let handlers = safe_lock(&self.command_handlers, "handle_command - get handlers")?;
            if let Some(handler) = handlers.get(&command_name) {
                return handler(message);
            }
        }

        // Use the command registry if available
        if let Some(registry) = &self.command_registry {
            // Try to find and execute the command
            let args: Vec<String> = if let Some(payload) = &message.payload {
                self.json_to_args(payload)?
            } else {
                vec![]
            };

            // Try to execute the command using the command registry execute method
            let result = registry.execute(&command_name, &args);

            match result {
                Ok(output) => {
                    // Create a success response
                    let response = MCPMessage::new_response(
                        message.id,
                        message.command,
                        Some(serde_json::json!({"result": output})),
                    );

                    Ok(response)
                }
                Err(e) => {
                    // Create an error response
                    let error = format!("Command execution failed: {}", e);
                    let response = MCPMessage::new_error(message.id, message.command, error);

                    Ok(response)
                }
            }
        } else {
            // Unknown command
            Err(MCPError::ProtocolError(format!(
                "Unknown command: {}",
                command_name
            )))
        }
    }

    /// Handle a notification message
    pub(super) fn handle_notification(
        &self,
        client_id: String,
        notification: MCPMessage,
    ) -> MCPResult<()> {
        let topic = notification.command.clone();

        debug!(
            "Received notification '{}' from client {}",
            topic, client_id
        );

        // Get subscribers for this topic (excluding sender)
        let subscribers = {
            let topic_subscribers = safe_lock(
                &self.topic_subscribers,
                "handle_notification - get subscribers",
            )?;
            topic_subscribers
                .get(&topic)
                .map(|subs| {
                    subs.iter()
                        .filter(|id| **id != client_id)
                        .cloned()
                        .collect::<HashSet<String>>()
                })
                .unwrap_or_default()
        };

        // Forward the notification to all subscribers
        if !subscribers.is_empty() {
            let json = notification.to_json()?;
            let clients = safe_lock(&self.clients, "handle_notification - get clients")?;

            for subscriber_id in subscribers {
                if let Some(stream) = clients.get(&subscriber_id) {
                    match safe_lock(stream, "handle_notification - subscriber stream") {
                        Ok(mut stream_guard) => {
                            if let Err(e) = stream_guard.write_all(format!("{}\n", json).as_bytes())
                            {
                                warn!(
                                    "Failed to forward notification to client {}: {}",
                                    subscriber_id, e
                                );
                            }
                        }
                        Err(e) => {
                            warn!("Failed to lock stream for client {}: {}", subscriber_id, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Convert JSON payload to command arguments
    pub(crate) fn json_to_args(&self, payload: &Value) -> MCPResult<Vec<String>> {
        if let Some(args) = payload.get("args") {
            if let Some(args_array) = args.as_array() {
                Ok(args_array
                    .iter()
                    .filter_map(|arg| arg.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>())
            } else {
                // If args is not an array, try to use it as a single string arg
                if let Some(arg_str) = args.as_str() {
                    Ok(vec![arg_str.to_string()])
                } else {
                    Ok(vec![])
                }
            }
        } else {
            // No args field, use empty args
            Ok(vec![])
        }
    }
}
