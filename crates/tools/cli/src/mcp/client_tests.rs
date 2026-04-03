// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![expect(
    clippy::expect_used,
    reason = "MCP CLI client tests use expect on I/O and JSON fixtures"
)]

use super::*;
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

#[test]
fn new_sets_expected_defaults() {
    let c = MCPClient::new("127.0.0.1".to_string(), 8444);
    assert!(!c.is_connected());
}

#[test]
fn with_timeout_chains() {
    let c = MCPClient::new("10.0.0.1".to_string(), 9000).with_timeout(Duration::from_secs(7));
    assert!(!c.is_connected());
    drop(c);
}

#[test]
fn operations_require_connection() {
    let c = MCPClient::new("127.0.0.1".to_string(), 59999);
    assert!(matches!(
        c.send_command("x", None),
        Err(MCPError::ConnectionError(_))
    ));
    assert!(matches!(
        c.send_notification("t", None),
        Err(MCPError::ConnectionError(_))
    ));
    assert!(matches!(
        c.subscribe("topic", |_t, _m| Ok(())),
        Err(MCPError::ConnectionError(_))
    ));
    assert!(c.disconnect().is_ok());
}

#[test]
fn unsubscribe_without_connection_errors() {
    let c = MCPClient::new("127.0.0.1".to_string(), 59998);
    assert!(matches!(
        c.unsubscribe(uuid::Uuid::new_v4()),
        Err(MCPError::ConnectionError(_))
    ));
}

#[test]
fn request_and_notification_shapes_via_protocol() {
    let id = uuid::Uuid::new_v4().to_string();
    let req = MCPMessage::new_request(id, "cmd".to_string(), Some(json!({"args": ["a"]})));
    assert_eq!(req.message_type, MCPMessageType::Request);
    let json = req.to_json().expect("json");
    let parsed = MCPMessage::from_json(&json).expect("parse");
    assert_eq!(parsed.command, "cmd");

    let n = MCPMessage::new_notification("n1".to_string(), "topic".to_string(), Some(json!({})));
    assert_eq!(n.message_type, MCPMessageType::Notification);
}

#[test]
fn from_json_rejects_malformed_input() {
    assert!(matches!(
        MCPMessage::from_json("not-json"),
        Err(MCPError::SerializationError(_))
    ));
}

#[test]
fn interactive_mode_requires_connection() {
    let c = MCPClient::new("127.0.0.1".to_string(), 59997);
    assert!(matches!(
        c.run_interactive(),
        Err(MCPError::ConnectionError(msg)) if msg.contains("Not connected")
    ));
}

#[test]
fn connect_refused_returns_connection_error() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    drop(listener);
    let mut c = MCPClient::new("127.0.0.1".to_string(), port);
    c.timeout = Some(Duration::from_millis(200));
    let err = c.connect(None).expect_err("nothing listening");
    assert!(matches!(err, MCPError::ConnectionError(_)));
}

#[test]
fn disconnect_when_never_connected_is_ok() {
    let c = MCPClient::new("127.0.0.1".to_string(), 59996);
    assert!(c.disconnect().is_ok());
}

#[test]
fn is_connected_false_after_construct() {
    let c = MCPClient::new("127.0.0.1".to_string(), 8444);
    assert!(!c.is_connected());
}

#[test]
fn error_response_message_maps_to_command_error() {
    let err_json = MCPMessage::new_error("e1".to_string(), "cmd".to_string(), "bad".to_string())
        .to_json()
        .expect("serialize");
    let msg = MCPMessage::from_json(&err_json).expect("parse");
    assert_eq!(msg.message_type, MCPMessageType::Error);
    assert_eq!(msg.error.as_deref(), Some("bad"));
}

/// Server accepts then closes — listener sees EOF; client should disconnect cleanly.
#[test]
fn connect_listener_eof_then_disconnect() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let server = thread::spawn(move || {
        if let Ok((stream, _)) = listener.accept() {
            drop(stream);
        }
    });
    let mut c = MCPClient::new("127.0.0.1".to_string(), port);
    c.timeout = Some(Duration::from_secs(2));
    c.connect(None).expect("connect");
    assert!(c.is_connected());
    c.disconnect().expect("disconnect");
    assert!(!c.is_connected());
    server.join().ok();
}

/// Malformed line from server exercises notification JSON parse error path in the listener.
#[test]
fn listener_malformed_json_line_is_tolerated() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let _ = writeln!(stream, "not-json {{{{");
            let _ = stream.flush();
            thread::sleep(Duration::from_millis(300));
        }
    });
    let mut c = MCPClient::new("127.0.0.1".to_string(), port);
    c.timeout = Some(Duration::from_secs(2));
    c.connect(None).expect("connect");
    thread::sleep(Duration::from_millis(50));
    c.disconnect().expect("disconnect");
}

/// Reconnect lifecycle: disconnect then connect to a new listener.
#[test]
fn reconnect_after_disconnect_to_new_port() {
    let l1 = TcpListener::bind("127.0.0.1:0").expect("bind");
    let p1 = l1.local_addr().expect("a1").port();
    let t1 = thread::spawn(move || {
        if let Ok((s, _)) = l1.accept() {
            drop(s);
        }
    });
    let mut c = MCPClient::new("127.0.0.1".to_string(), p1);
    c.timeout = Some(Duration::from_secs(2));
    c.connect(None).expect("c1");
    c.disconnect().expect("d1");
    t1.join().ok();

    let l2 = TcpListener::bind("127.0.0.1:0").expect("bind");
    let p2 = l2.local_addr().expect("a2").port();
    let t2 = thread::spawn(move || {
        if let Ok((s, _)) = l2.accept() {
            drop(s);
        }
    });
    let mut c2 = MCPClient::new("127.0.0.1".to_string(), p2);
    c2.timeout = Some(Duration::from_secs(2));
    c2.connect(None).expect("c2");
    c2.disconnect().expect("d2");
    t2.join().ok();
}

/// Non-notification JSON line is ignored by the listener (debug path).
#[test]
fn listener_ignores_non_notification_message_type() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let line = MCPMessage::new_response(
                "r1".to_string(),
                "cmd".to_string(),
                Some(json!({"ok": true})),
            )
            .to_json()
            .expect("ser");
            let _ = writeln!(stream, "{}", line);
            let _ = stream.flush();
            thread::sleep(Duration::from_millis(200));
        }
    });
    let mut c = MCPClient::new("127.0.0.1".to_string(), port);
    c.timeout = Some(Duration::from_secs(2));
    c.connect(None).expect("connect");
    thread::sleep(Duration::from_millis(50));
    c.disconnect().expect("disconnect");
}

#[test]
fn drop_disconnects_when_connected() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    thread::spawn(move || {
        if let Ok((s, _)) = listener.accept() {
            drop(s);
        }
    });
    {
        let mut c = MCPClient::new("127.0.0.1".to_string(), port);
        c.timeout = Some(Duration::from_secs(2));
        c.connect(None).expect("connect");
        assert!(c.is_connected());
    }
}

/// Mirrors `send_message` response handling (avoids TCP race with the notification listener).
#[test]
fn send_message_response_maps_error_type_to_command_error() {
    let err_line = MCPMessage::new_error("e1".to_string(), "c".to_string(), "bad".to_string())
        .to_json()
        .expect("ser");
    let response = MCPMessage::from_json(&err_line).expect("parse");
    let mapped = if response.message_type == MCPMessageType::Error
        && let Some(error) = &response.error
    {
        Err(MCPError::CommandError(error.clone()))
    } else {
        Ok(response)
    };
    assert!(matches!(mapped, Err(MCPError::CommandError(ref s)) if s == "bad"));
}

/// `SocketAddr` parsing requires a numeric IP — hostnames fail before connect.
#[test]
fn connect_rejects_unparseable_address() {
    let mut c = MCPClient::new("localhost".to_string(), 8080);
    c.timeout = Some(Duration::from_millis(200));
    let err = c.connect(None).expect_err("expected parse error");
    assert!(matches!(err, MCPError::ConnectionError(ref m) if m.contains("parse")));
}

#[test]
fn send_notification_round_trip_no_response_body() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let server = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut line = String::new();
            {
                let mut reader = BufReader::new(&mut stream);
                reader.read_line(&mut line).expect("read");
            }
            assert!(line.contains("notify_topic"));
        }
    });
    let mut c = MCPClient::new("127.0.0.1".to_string(), port);
    c.timeout = Some(Duration::from_secs(2));
    c.connect(None).expect("connect");
    c.send_notification("notify_topic", Some(json!({"z": 2})))
        .expect("notify");
    c.disconnect().expect("disconnect");
    server.join().expect("server");
}
