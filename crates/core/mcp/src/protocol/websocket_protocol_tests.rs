// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive tests for WebSocket protocol types and structures

#[cfg(test)]
mod tests {
    use super::super::types::{MCPMessage, MessageType};
    use super::super::websocket::*;
    use std::time::{Duration, Instant};

    // ========== WebSocketConfig Tests ==========

    #[test]
    fn test_websocket_config_default() {
        let config = WebSocketConfig::default();

        assert_eq!(config.bind_address, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.buffer_size, 1024);
        assert_eq!(config.connection_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_websocket_config_custom() {
        let config = WebSocketConfig {
            bind_address: "0.0.0.0".to_string(),
            port: 9090,
            timeout_seconds: 60,
            max_connections: 500,
            buffer_size: 2048,
            connection_timeout: Duration::from_secs(60),
        };

        assert_eq!(config.bind_address, "0.0.0.0");
        assert_eq!(config.port, 9090);
        assert_eq!(config.timeout_seconds, 60);
        assert_eq!(config.max_connections, 500);
        assert_eq!(config.buffer_size, 2048);
    }

    #[test]
    fn test_websocket_config_clone() {
        let config = WebSocketConfig::default();
        let cloned = config.clone();

        assert_eq!(config.port, cloned.port);
        assert_eq!(config.bind_address, cloned.bind_address);
    }

    #[test]
    fn test_websocket_config_debug() {
        let config = WebSocketConfig::default();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("WebSocketConfig"));
        assert!(debug_str.contains("8080"));
    }

    // ========== ConnectionState Tests ==========

    #[test]
    fn test_connection_state_variants() {
        let _ = ConnectionState::Connecting;
        let _ = ConnectionState::Connected;
        let _ = ConnectionState::Disconnecting;
        let _ = ConnectionState::Disconnected;
        let _ = ConnectionState::Failed;
    }

    #[test]
    fn test_connection_state_equality() {
        assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);
    }

    #[test]
    fn test_connection_state_clone() {
        let state = ConnectionState::Connected;
        let cloned = state.clone();

        assert_eq!(state, cloned);
    }

    #[test]
    fn test_connection_state_serialization() {
        let state = ConnectionState::Connected;
        let json = serde_json::to_string(&state).expect("test: should succeed");
        let deserialized: ConnectionState =
            serde_json::from_str(&json).expect("test: should succeed");

        assert_eq!(state, deserialized);
    }

    #[test]
    fn test_connection_state_debug() {
        let state = ConnectionState::Connecting;
        let debug_str = format!("{:?}", state);

        assert!(debug_str.contains("Connecting"));
    }

    #[test]
    fn test_connection_state_all_states() {
        let states = vec![
            ConnectionState::Connecting,
            ConnectionState::Connected,
            ConnectionState::Disconnecting,
            ConnectionState::Disconnected,
            ConnectionState::Failed,
        ];

        for state in states {
            let json = serde_json::to_string(&state).expect("test: should succeed");
            let _: ConnectionState = serde_json::from_str(&json).expect("test: should succeed");
        }
    }

    // ========== ConnectionInfo Tests ==========

    #[test]
    fn test_connection_info_creation() {
        let now = Instant::now();
        let info = ConnectionInfo {
            id: "conn-123".to_string(),
            remote_address: "127.0.0.1:8080".to_string(),
            state: ConnectionState::Connected,
            connected_at: now,
            last_message_at: None,
            message_count: 0,
            last_ping: None,
            last_pong: None,
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
        };

        assert_eq!(info.id, "conn-123");
        assert_eq!(info.remote_address, "127.0.0.1:8080");
        assert_eq!(info.state, ConnectionState::Connected);
        assert_eq!(info.message_count, 0);
    }

    #[test]
    fn test_connection_info_with_activity() {
        let now = Instant::now();
        let info = ConnectionInfo {
            id: "conn-456".to_string(),
            remote_address: "192.168.1.1:9090".to_string(),
            state: ConnectionState::Connected,
            connected_at: now,
            last_message_at: Some(now),
            message_count: 10,
            last_ping: Some(now),
            last_pong: Some(now),
            bytes_sent: 1024,
            bytes_received: 2048,
            messages_sent: 5,
            messages_received: 5,
        };

        assert_eq!(info.message_count, 10);
        assert_eq!(info.bytes_sent, 1024);
        assert_eq!(info.bytes_received, 2048);
        assert_eq!(info.messages_sent, 5);
        assert_eq!(info.messages_received, 5);
        assert!(info.last_message_at.is_some());
    }

    #[test]
    fn test_connection_info_clone() {
        let now = Instant::now();
        let info = ConnectionInfo {
            id: "conn-789".to_string(),
            remote_address: "10.0.0.1:3000".to_string(),
            state: ConnectionState::Connecting,
            connected_at: now,
            last_message_at: None,
            message_count: 0,
            last_ping: None,
            last_pong: None,
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
        };

        let cloned = info.clone();
        assert_eq!(info.id, cloned.id);
        assert_eq!(info.state, cloned.state);
    }

    #[test]
    fn test_connection_info_debug() {
        let now = Instant::now();
        let info = ConnectionInfo {
            id: "debug-test".to_string(),
            remote_address: "127.0.0.1:8080".to_string(),
            state: ConnectionState::Connected,
            connected_at: now,
            last_message_at: None,
            message_count: 0,
            last_ping: None,
            last_pong: None,
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
        };

        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("ConnectionInfo"));
        assert!(debug_str.contains("debug-test"));
    }

    // ========== WebSocketTransport Tests ==========

    #[test]
    fn test_websocket_transport_creation() {
        let now = Instant::now();
        let connection = ConnectionInfo {
            id: "transport-1".to_string(),
            remote_address: "127.0.0.1:8080".to_string(),
            state: ConnectionState::Connected,
            connected_at: now,
            last_message_at: None,
            message_count: 0,
            last_ping: None,
            last_pong: None,
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
        };

        let config = WebSocketConfig::default();
        let transport = WebSocketTransport::new(connection.clone(), config.clone());

        assert_eq!(transport.connection.id, "transport-1");
        assert_eq!(transport.config.port, 8080);
    }

    #[test]
    fn test_websocket_transport_clone() {
        let now = Instant::now();
        let connection = ConnectionInfo {
            id: "transport-2".to_string(),
            remote_address: "127.0.0.1:9090".to_string(),
            state: ConnectionState::Connected,
            connected_at: now,
            last_message_at: None,
            message_count: 0,
            last_ping: None,
            last_pong: None,
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
        };

        let config = WebSocketConfig::default();
        let transport = WebSocketTransport::new(connection, config);
        let cloned = transport.clone();

        assert_eq!(transport.connection.id, cloned.connection.id);
    }

    #[test]
    fn test_websocket_transport_debug() {
        let now = Instant::now();
        let connection = ConnectionInfo {
            id: "transport-3".to_string(),
            remote_address: "127.0.0.1:8080".to_string(),
            state: ConnectionState::Connected,
            connected_at: now,
            last_message_at: None,
            message_count: 0,
            last_ping: None,
            last_pong: None,
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
        };

        let config = WebSocketConfig::default();
        let transport = WebSocketTransport::new(connection, config);

        let debug_str = format!("{:?}", transport);
        assert!(debug_str.contains("WebSocketTransport"));
    }

    // ========== ServerEvent Tests ==========

    #[test]
    fn test_server_event_client_connected() {
        let event = ServerEvent::ClientConnected("client-123".to_string());

        match event {
            ServerEvent::ClientConnected(id) => assert_eq!(id, "client-123"),
            _ => panic!("Expected ClientConnected"),
        }
    }

    #[test]
    fn test_server_event_client_disconnected() {
        let event = ServerEvent::ClientDisconnected("client-456".to_string());

        match event {
            ServerEvent::ClientDisconnected(id) => assert_eq!(id, "client-456"),
            _ => panic!("Expected ClientDisconnected"),
        }
    }

    #[test]
    fn test_server_event_connection_error() {
        let event = ServerEvent::ConnectionError(
            "client-789".to_string(),
            "Connection timeout".to_string(),
        );

        match event {
            ServerEvent::ConnectionError(id, msg) => {
                assert_eq!(id, "client-789");
                assert_eq!(msg, "Connection timeout");
            }
            _ => panic!("Expected ConnectionError"),
        }
    }

    #[test]
    fn test_server_event_clone() {
        let event = ServerEvent::ClientConnected("client-999".to_string());
        let cloned = event.clone();

        match (event, cloned) {
            (ServerEvent::ClientConnected(id1), ServerEvent::ClientConnected(id2)) => {
                assert_eq!(id1, id2);
            }
            _ => panic!("Clone produced different variant"),
        }
    }

    #[test]
    fn test_server_event_debug() {
        let event = ServerEvent::ClientConnected("debug-client".to_string());
        let debug_str = format!("{:?}", event);

        assert!(debug_str.contains("ClientConnected"));
        assert!(debug_str.contains("debug-client"));
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_connection_lifecycle() {
        let states = vec![
            ConnectionState::Connecting,
            ConnectionState::Connected,
            ConnectionState::Disconnecting,
            ConnectionState::Disconnected,
        ];

        for (i, state) in states.iter().enumerate() {
            assert_eq!(state, &states[i]);
        }
    }

    #[test]
    fn test_connection_stats_tracking() {
        let now = Instant::now();
        let mut info = ConnectionInfo {
            id: "stats-conn".to_string(),
            remote_address: "127.0.0.1:8080".to_string(),
            state: ConnectionState::Connected,
            connected_at: now,
            last_message_at: None,
            message_count: 0,
            last_ping: None,
            last_pong: None,
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
        };

        // Simulate activity
        info.message_count += 1;
        info.bytes_sent += 512;
        info.bytes_received += 256;
        info.messages_sent += 1;
        info.messages_received += 0;

        assert_eq!(info.message_count, 1);
        assert_eq!(info.bytes_sent, 512);
        assert_eq!(info.bytes_received, 256);
    }

    #[test]
    fn test_multiple_connections() {
        let now = Instant::now();
        let connections = vec![
            ConnectionInfo {
                id: "conn-1".to_string(),
                remote_address: "127.0.0.1:8081".to_string(),
                state: ConnectionState::Connected,
                connected_at: now,
                last_message_at: None,
                message_count: 0,
                last_ping: None,
                last_pong: None,
                bytes_sent: 0,
                bytes_received: 0,
                messages_sent: 0,
                messages_received: 0,
            },
            ConnectionInfo {
                id: "conn-2".to_string(),
                remote_address: "127.0.0.1:8082".to_string(),
                state: ConnectionState::Connected,
                connected_at: now,
                last_message_at: None,
                message_count: 0,
                last_ping: None,
                last_pong: None,
                bytes_sent: 0,
                bytes_received: 0,
                messages_sent: 0,
                messages_received: 0,
            },
        ];

        assert_eq!(connections.len(), 2);
        assert_eq!(connections[0].id, "conn-1");
        assert_eq!(connections[1].id, "conn-2");
    }

    #[test]
    fn test_config_variations() {
        let configs = vec![
            WebSocketConfig {
                bind_address: "127.0.0.1".to_string(),
                port: 8080,
                timeout_seconds: 30,
                max_connections: 100,
                buffer_size: 1024,
                connection_timeout: Duration::from_secs(30),
            },
            WebSocketConfig {
                bind_address: "0.0.0.0".to_string(),
                port: 9090,
                timeout_seconds: 60,
                max_connections: 1000,
                buffer_size: 4096,
                connection_timeout: Duration::from_secs(60),
            },
        ];

        assert_eq!(configs[0].port, 8080);
        assert_eq!(configs[1].port, 9090);
    }
}
