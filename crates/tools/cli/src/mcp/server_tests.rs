// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Comprehensive tests for MCP server

use super::protocol::{MCPMessage, MCPMessageType};
use super::server::*;
use std::sync::Arc;

#[test]
fn test_server_creation_with_defaults() {
    let server = MCPServer::new(None, None);
    assert!(!server.is_running());
    assert!(!server.has_command_registry());
}

#[test]
fn test_server_creation_with_custom_host_port() {
    let server = MCPServer::new(Some("192.168.1.1"), Some(9000));
    assert!(!server.is_running());
}

#[test]
fn test_default_host_function() {
    let host = default_host();
    assert!(!host.is_empty());
}

#[test]
fn test_default_port_function() {
    let port = default_port();
    assert!(port > 0);
}

#[test]
fn test_has_command_registry_false_by_default() {
    let server = MCPServer::new(None, None);
    assert!(!server.has_command_registry());
}

#[test]
fn test_with_command_registry() {
    use crate::commands::registry::CommandRegistry;

    let registry = Arc::new(CommandRegistry::new());
    let server = MCPServer::new(None, None).with_command_registry(registry);

    assert!(server.has_command_registry());
}

#[test]
fn test_register_handler() {
    let server = MCPServer::new(None, None);

    let handler = |msg: MCPMessage| {
        Ok(MCPMessage {
            id: msg.id,
            message_type: MCPMessageType::Response,
            command: "test".to_string(),
            payload: None,
            error: None,
        })
    };

    let server = server.register_handler("test_command", handler);
    assert!(!server.is_running());
}

#[test]
fn test_register_multiple_handlers() {
    let server = MCPServer::new(None, None);

    let handler1 = |msg: MCPMessage| {
        Ok(MCPMessage {
            id: msg.id,
            message_type: MCPMessageType::Response,
            command: "cmd1".to_string(),
            payload: None,
            error: None,
        })
    };

    let handler2 = |msg: MCPMessage| {
        Ok(MCPMessage {
            id: msg.id,
            message_type: MCPMessageType::Response,
            command: "cmd2".to_string(),
            payload: None,
            error: None,
        })
    };

    let server = server
        .register_handler("cmd1", handler1)
        .register_handler("cmd2", handler2);

    assert!(!server.is_running());
}

#[test]
fn test_is_running_initial_state() {
    let server = MCPServer::new(None, None);
    assert!(!server.is_running());
}

#[test]
fn test_server_clone() {
    let server1 = MCPServer::new(Some("localhost"), Some(8080));
    let server2 = server1.clone();

    assert!(!server2.is_running());
    assert!(!server2.has_command_registry());
}

#[test]
fn test_server_builder_pattern() {
    use crate::commands::registry::CommandRegistry;

    let registry = Arc::new(CommandRegistry::new());
    let handler = |msg: MCPMessage| {
        Ok(MCPMessage {
            id: msg.id,
            message_type: MCPMessageType::Response,
            command: "test".to_string(),
            payload: None,
            error: None,
        })
    };

    let server = MCPServer::new(Some("127.0.0.1"), Some(8888))
        .with_command_registry(registry)
        .register_handler("test", handler);

    assert!(server.has_command_registry());
    assert!(!server.is_running());
}
