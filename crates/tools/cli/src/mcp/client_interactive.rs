// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Interactive REPL for [`super::MCPClient`](crate::mcp::MCPClient).

use std::io::{BufRead, Write};

use tracing::{error, warn};

use crate::mcp::client::MCPClient;
use crate::mcp::protocol::{MCPError, MCPResult};

/// Run in interactive mode (stdin command loop).
#[expect(
    clippy::too_many_lines,
    reason = "REPL command loop; split later if this grows further"
)]
pub fn run_interactive(client: &MCPClient) -> MCPResult<()> {
    // Ensure connected
    if !client.is_connected() {
        return Err(MCPError::ConnectionError(
            "Not connected to MCP server".to_string(),
        ));
    }

    println!("MCP Interactive Mode");
    println!("Type 'exit' or 'quit' to exit");
    println!("Type 'help' for available commands");
    println!("Format: <command> [JSON args]");

    // Read commands from stdin
    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines();

    loop {
        print!("> ");
        std::io::stdout().flush()?;

        if let Some(line) = lines.next() {
            let line = line?;
            let line = line.trim();

            // Exit command
            if line == "exit" || line == "quit" {
                break;
            }

            // Help command
            if line == "help" {
                println!("Available commands:");
                println!("  <command> [JSON args] - Execute command");
                println!("  subscribe <topic>     - Subscribe to topic");
                println!("  unsubscribe <topic>   - Unsubscribe from topic");
                println!("  notify <topic> [JSON] - Send notification");
                println!("  help                  - Show this help");
                println!("  exit, quit            - Exit interactive mode");
                continue;
            }

            // Parse command and args
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.is_empty() {
                continue;
            }

            let command = parts[0];

            // Handle special commands
            match command {
                "subscribe" => {
                    if parts.len() > 1 {
                        let topic = parts[1];

                        // Subscribe to topic with a handler that prints notifications
                        match client.subscribe(topic, |topic, msg| {
                            println!("Notification received from topic {}: {:?}", topic, msg);
                            Ok(())
                        }) {
                            Ok(_) => println!("Subscribed to topic: {}", topic),
                            Err(e) => error!("Error subscribing to topic: {}", e),
                        }
                    } else {
                        warn!("Missing topic. Usage: subscribe <topic>");
                    }
                    continue;
                }
                "unsubscribe" => {
                    if parts.len() > 1 {
                        let topic = parts[1];

                        let subscription_ids = match client.subscription_ids_for_topic(topic) {
                            Ok(Some(ids)) if !ids.is_empty() => ids,
                            Ok(_) => {
                                warn!("Not subscribed to topic: {}", topic);
                                continue;
                            }
                            Err(_) => {
                                error!("Subscriptions mutex poisoned");
                                continue;
                            }
                        };

                        // Unsubscribe from all
                        for id in subscription_ids {
                            match client.unsubscribe(id) {
                                Ok(_) => println!("Unsubscribed from topic: {}", topic),
                                Err(e) => error!("Error unsubscribing from topic: {}", e),
                            }
                        }
                    } else {
                        warn!("Missing topic. Usage: unsubscribe <topic>");
                    }
                    continue;
                }
                "notify" => {
                    if parts.len() < 2 {
                        warn!("Missing topic. Usage: notify <topic> [JSON]");
                        continue;
                    }

                    let notify_parts: Vec<&str> = parts[1].splitn(2, ' ').collect();
                    let topic = notify_parts[0];

                    // Parse JSON payload if provided
                    let payload = if notify_parts.len() > 1 {
                        match serde_json::from_str(notify_parts[1]) {
                            Ok(json) => Some(json),
                            Err(e) => {
                                error!("Error parsing JSON payload: {}", e);
                                continue;
                            }
                        }
                    } else {
                        None
                    };

                    // Send notification
                    match client.send_notification(topic, payload) {
                        Ok(_) => println!("Notification sent to topic: {}", topic),
                        Err(e) => error!("Error sending notification: {}", e),
                    }
                    continue;
                }
                _ => {}
            }

            // Parse JSON args if provided
            let args = if parts.len() > 1 {
                match serde_json::from_str(parts[1]) {
                    Ok(json) => Some(json),
                    Err(e) => {
                        error!("Error parsing JSON args: {}", e);
                        continue;
                    }
                }
            } else {
                None
            };

            // Send command
            match client.send_command(command, args) {
                Ok(response) => {
                    // Pretty print response
                    match serde_json::to_string_pretty(&response) {
                        Ok(json) => println!("{}", json),
                        Err(e) => error!("Error formatting response: {}", e),
                    }
                }
                Err(e) => {
                    error!("Error sending command: {}", e);
                }
            }
        }
    }

    println!("Exiting interactive mode");
    Ok(())
}
