// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use std::sync::Arc;

use super::*;
use crate::commands::registry::CommandRegistry;
use clap::Command as ClapCommand;
use serde_json::json;
use squirrel_commands::Command as SqCommand;
use squirrel_commands::error::CommandError;

struct Echo;

impl SqCommand for Echo {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "echo"
    }

    fn execute(&self, args: &[String]) -> Result<String, CommandError> {
        Ok(args.join(","))
    }

    fn parser(&self) -> ClapCommand {
        ClapCommand::new("echo")
    }

    fn clone_box(&self) -> Box<dyn SqCommand> {
        Box::new(Echo)
    }
}

#[test]
fn default_host_and_port_are_usable() {
    let h = default_host();
    assert!(!h.is_empty());
    let p = default_port();
    assert!(p > 0);
    let _ = MCPServer::new(None, None);
}

#[test]
fn server_config_builder_and_registry_flag() {
    let s = MCPServer::new(Some("192.168.1.1"), Some(1234));
    assert!(!s.has_command_registry());
    let reg = Arc::new(CommandRegistry::new());
    let s2 = s.with_command_registry(reg);
    assert!(s2.has_command_registry());
}

#[test]
fn register_handler_routes_requests() {
    let server = MCPServer::new(Some("127.0.0.1"), Some(65534)).register_handler("ping", |msg| {
        Ok(MCPMessage::new_response(
            msg.id,
            "ping".to_string(),
            Some(json!({"pong": true})),
        ))
    });

    let req = MCPMessage::new_request("id-1".to_string(), "ping".to_string(), None);
    let out = server.handle_command(req).expect("handler");
    assert_eq!(out.message_type, MCPMessageType::Response);
    assert_eq!(out.payload, Some(json!({"pong": true})));
}

#[test]
fn handle_command_uses_registry_execute() {
    let registry = Arc::new(CommandRegistry::new());
    registry.register("echo", Arc::new(Echo)).expect("register");

    let server = MCPServer::new(Some("127.0.0.1"), Some(65533)).with_command_registry(registry);
    let req = MCPMessage::new_request(
        "r1".to_string(),
        "echo".to_string(),
        Some(json!({"args": ["a", "b"]})),
    );
    let resp = server.handle_command(req).expect("ok");
    assert_eq!(resp.message_type, MCPMessageType::Response);
    let payload = resp.payload.expect("payload");
    assert_eq!(payload["result"], "a,b");

    let bad = MCPMessage::new_request("r2".to_string(), "missing".to_string(), None);
    let resp = server.handle_command(bad).expect("error response");
    assert_eq!(resp.message_type, MCPMessageType::Error);
    assert!(resp.error.as_deref().unwrap_or("").contains("not found"));
}

#[test]
fn json_to_args_edge_cases() {
    let server = MCPServer::new(None, None);
    assert!(server.json_to_args(&json!({})).unwrap().is_empty());
    assert_eq!(
        server.json_to_args(&json!({"args": "single"})).unwrap(),
        vec!["single".to_string()]
    );
    assert_eq!(
        server.json_to_args(&json!({"args": ["x"]})).unwrap(),
        vec!["x".to_string()]
    );
}

#[test]
fn process_message_request_routing_subscribe_unsubscribe_and_errors() {
    let server = MCPServer::new(Some("127.0.0.1"), Some(65532));

    let sub = MCPMessage::new_request(
        "s1".to_string(),
        "subscribe".to_string(),
        Some(json!({"topic": "news"})),
    );
    let line = sub.to_json().unwrap();
    let out = server
        .process_message("c1", &line)
        .expect("subscribe")
        .expect("response");
    assert!(out.contains("news"));

    let unsub = MCPMessage::new_request(
        "u1".to_string(),
        "unsubscribe".to_string(),
        Some(json!({"topic": "*"})),
    );
    let out = server
        .process_message("c1", &unsub.to_json().unwrap())
        .expect("unsub")
        .expect("resp");
    assert!(out.contains("ok"));

    let bad_sub = MCPMessage::new_request(
        "b1".to_string(),
        "subscribe".to_string(),
        Some(json!({"topic": 1})),
    );
    assert!(
        server
            .process_message("c1", &bad_sub.to_json().unwrap())
            .is_err()
    );

    let resp_msg = MCPMessage::new_response("x".to_string(), "cmd".to_string(), None);
    assert!(
        server
            .process_message("c1", &resp_msg.to_json().unwrap())
            .is_err()
    );
}

#[test]
fn stop_when_not_running_errors() {
    let server = MCPServer::new(Some("127.0.0.1"), Some(65531));
    assert!(!server.is_running());
    assert!(server.stop().is_err());
}

#[test]
fn subscription_helpers_round_trip() {
    let server = MCPServer::new(Some("127.0.0.1"), Some(65530));
    server.subscribe_client("a", "t1").expect("sub");
    server
        .broadcast_notification("t1", Some(json!({"x": 1})))
        .expect("broadcast");
    server.unsubscribe_client("a", "t1").expect("unsub");
    server.subscribe_client("b", "t2").expect("sub b");
    server.unsubscribe_client_all("b").expect("all");
    assert!(server.send_notification("nope", "t", None).is_err());
}

#[test]
fn notification_from_client_forwards_to_peer() {
    let server = MCPServer::new(Some("127.0.0.1"), Some(65529));
    server.subscribe_client("listener", "alerts").expect("sub");
    let note = MCPMessage::new_notification(
        "n1".to_string(),
        "alerts".to_string(),
        Some(json!({"level": "info"})),
    );
    server
        .handle_notification("sender".to_string(), note)
        .expect("handle");
}

#[test]
fn process_message_notification_returns_no_line() {
    let server = MCPServer::new(Some("127.0.0.1"), Some(65528));
    let note = MCPMessage::new_notification(
        "n2".to_string(),
        "topic_x".to_string(),
        Some(json!({ "x": 1 })),
    );
    let out = server
        .process_message("c99", &note.to_json().unwrap())
        .expect("ok");
    assert!(out.is_none());
}

#[test]
fn handle_command_unknown_without_registry_errors() {
    let server = MCPServer::new(Some("127.0.0.1"), Some(65527));
    let req = MCPMessage::new_request("r".to_string(), "unknown".to_string(), None);
    let err = server.handle_command(req).expect_err("no registry");
    assert!(matches!(err, MCPError::ProtocolError(_)));
}

#[test]
fn start_when_already_running_errors() {
    let Some(port) = portpicker::pick_unused_port() else {
        return;
    };
    let server = MCPServer::new(Some("127.0.0.1"), Some(port));
    server.start().expect("first start");
    let second = server.start();
    assert!(second.is_err());
    let _ = server.stop();
}

#[test]
fn broadcast_notification_empty_subscribers_ok() {
    let server = MCPServer::new(Some("127.0.0.1"), Some(65526));
    server
        .broadcast_notification("no_subscribers", Some(json!({})))
        .expect("broadcast");
}

#[test]
fn json_to_args_non_array_args_branch() {
    let server = MCPServer::new(None, None);
    let v = serde_json::json!({"args": 42});
    let args = server.json_to_args(&v).expect("args");
    assert!(args.is_empty());
}

#[test]
fn handle_unsubscribe_specific_topic() {
    let server = MCPServer::new(Some("127.0.0.1"), Some(65525));
    server.subscribe_client("c1", "news").expect("sub");
    let req = MCPMessage::new_request(
        "u2".to_string(),
        "unsubscribe".to_string(),
        Some(json!({"topic": "news"})),
    );
    let line = server
        .process_message("c1", &req.to_json().unwrap())
        .expect("unsub")
        .expect("line");
    assert!(line.contains("ok"));
}

#[test]
fn register_handler_overwrites_previous() {
    let server = MCPServer::new(Some("127.0.0.1"), Some(65524))
        .register_handler("dup", |_| {
            Ok(MCPMessage::new_response(
                "1".to_string(),
                "dup".to_string(),
                Some(json!({"v": 1})),
            ))
        })
        .register_handler("dup", |_| {
            Ok(MCPMessage::new_response(
                "2".to_string(),
                "dup".to_string(),
                Some(json!({"v": 2})),
            ))
        });
    let req = MCPMessage::new_request("x".to_string(), "dup".to_string(), None);
    let out = server.handle_command(req).expect("cmd");
    assert_eq!(out.payload, Some(json!({"v": 2})));
}
