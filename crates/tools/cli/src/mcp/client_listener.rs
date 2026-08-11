// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! TCP notification listener loop for [`super::MCPClient`](crate::mcp::MCPClient).

use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{debug, error};
use uuid::Uuid;

use crate::mcp::client_types::NotificationCallback;
use crate::mcp::protocol::{MCPMessage, MCPMessageType};

/// Runs until `running` is false or the stream ends / errors.
pub fn run_notification_listener(
    stream: TcpStream,
    subscriptions: Arc<Mutex<HashMap<String, HashSet<Uuid>>>>,
    notification_handlers: Arc<Mutex<HashMap<Uuid, NotificationCallback>>>,
    running: Arc<AtomicBool>,
) {
    let mut reader = BufReader::new(stream);

    while running.load(Ordering::Acquire) {
        let mut line = String::new();

        // Read a line from the server
        match reader.read_line(&mut line) {
            Ok(0) => {
                // End of stream, server disconnected
                debug!("Server disconnected (EOF)");
                break;
            }
            Ok(_) => {
                // Process message
                match MCPMessage::from_json(line.trim()) {
                    Ok(message) => {
                        // Only process notifications
                        if message.message_type == MCPMessageType::Notification {
                            let topic = message.command.clone();

                            // Find handlers for this topic
                            let Ok(subs) = subscriptions.lock() else {
                                error!("Subscriptions mutex poisoned, skipping notification");
                                continue;
                            };
                            let handlers = subs
                                .get(&topic)
                                .cloned()
                                .unwrap_or_default();
                            drop(subs);

                            for handler_id in handlers {
                                if let Ok(handlers) = notification_handlers.lock() {
                                    if let Some(handler) = handlers.get(&handler_id)
                                        && let Err(e) = handler(&topic, &message)
                                    {
                                        error!("Error in notification handler: {}", e);
                                    }
                                } else {
                                    error!(
                                        "Notification handlers mutex poisoned, skipping handler"
                                    );
                                }
                            }
                        } else {
                            debug!(
                                "Ignored non-notification message: {:?}",
                                message.message_type
                            );
                        }
                    }
                    Err(e) => {
                        error!("Error parsing notification: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Error reading from server: {}", e);
                break;
            }
        }
    }

    debug!("Notification listener thread exiting");
}
