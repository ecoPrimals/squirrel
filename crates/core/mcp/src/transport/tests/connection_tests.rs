// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::sync::Arc;
use tokio::test;
use tokio::sync::RwLock;
use tokio::net::TcpStream;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

use crate::mcp::transport::{Transport, TransportConfig, Connection, ConnectionState};
use crate::mcp::types::{
    SecurityLevel,
    CompressionFormat,
    EncryptionFormat,
    ProtocolVersion,
};
use crate::test_utils::MockSecurityManager;

async fn await_transport_running(transport: &Transport) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while !transport.is_running().await {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("transport should start within timeout");
}

async fn await_connections(transport: &Transport, expected: usize) {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if transport.get_active_connections().await.expect("count") == expected {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("expected connection count within timeout");
}

// Helper function to create a TCP socket pair (server and client)
async fn create_socket_pair() -> (TcpStream, TcpStream) {
    // Create a TCP listener on a random port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("should succeed");
    let addr = listener.local_addr().expect("should succeed");
    
    // Connect client to server
    let client_stream = TcpStream::connect(addr).await.expect("should succeed");
    
    // Accept connection on server side
    let (server_stream, _) = listener.accept().await.expect("should succeed");
    
    (server_stream, client_stream)
}

#[test]
async fn test_connection_creation() {
    // Create server and client streams
    let (server_stream, _client_stream) = create_socket_pair().await;
    
    // Create connection
    let connection = Connection::new(
        server_stream,
        server_stream.peer_addr().expect("should succeed"),
    );
    
    // Verify connection properties
    assert!(!connection.id.is_empty());
    assert_eq!(connection.state, ConnectionState::New);
    assert_eq!(connection.remote_addr, server_stream.peer_addr().expect("should succeed"));
    
    // Check timestamps
    let now = chrono::Utc::now();
    assert!(connection.created_at <= now);
    assert!(connection.last_activity <= now);
}

#[test]
async fn test_connection_state_transitions() {
    // Create server and client streams
    let (server_stream, _client_stream) = create_socket_pair().await;
    
    // Create connection
    let mut connection = Connection::new(
        server_stream,
        server_stream.peer_addr().expect("should succeed"),
    );
    
    // Check initial state
    assert_eq!(connection.state, ConnectionState::New);
    
    // Test state transitions
    connection.set_state(ConnectionState::Handshaking);
    assert_eq!(connection.state, ConnectionState::Handshaking);
    
    connection.set_state(ConnectionState::Active);
    assert_eq!(connection.state, ConnectionState::Active);
    
    connection.set_state(ConnectionState::Closing);
    assert_eq!(connection.state, ConnectionState::Closing);
    
    connection.set_state(ConnectionState::Closed);
    assert_eq!(connection.state, ConnectionState::Closed);
}

#[test]
async fn test_connection_activity_tracking() {
    // Create server and client streams
    let (server_stream, _client_stream) = create_socket_pair().await;
    
    // Create connection
    let mut connection = Connection::new(
        server_stream,
        server_stream.peer_addr().expect("should succeed"),
    );
    
    // Get initial last_activity time
    let initial_activity = connection.last_activity;
    
    tokio::task::yield_now().await;
    
    // Update activity timestamp
    connection.update_activity();
    
    // Verify last_activity was updated
    assert!(connection.last_activity > initial_activity);
}

#[test]
async fn test_connection_in_transport() {
    // Create mock security manager
    let security_manager = Arc::new(RwLock::new(MockSecurityManager::new()));
    
    // Create transport with DI
    let config = TransportConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0, // Let OS assign port
        max_connections: 5,
        max_message_size: 1024,
        protocol_version: ProtocolVersion::new(1, 0),
        security_level: SecurityLevel::None,
        compression: CompressionFormat::None,
        encryption: EncryptionFormat::None,
    };
    
    let transport = Arc::new(Transport::with_security_manager(
        config.clone(),
        security_manager.clone()
    ).await.expect("should succeed"));
    
    // Start transport in background
    let transport_clone = transport.clone();
    let handle = tokio::spawn(async move {
        transport_clone.start().await.expect("should succeed");
    });
    
    await_transport_running(&transport).await;
    
    let port = transport.get_port().await.expect("should succeed");
    let addr = SocketAddr::from_str(&format!("127.0.0.1:{}", port)).expect("should succeed");
    let client1 = TcpStream::connect(addr).await.expect("should succeed");
    let client2 = TcpStream::connect(addr).await.expect("should succeed");
    
    await_connections(&transport, 2).await;
    
    // Get connection details
    let connections = transport.get_connection_states().await.expect("should succeed");
    assert_eq!(connections.len(), 2);
    
    // Verify connection states
    for conn in connections {
        assert_eq!(conn.state, ConnectionState::Active);
        assert!(conn.id.len() > 0);
    }
    
    drop(client1);
    
    await_connections(&transport, 1).await;
    
    // Cleanup
    drop(client2);
    transport.shutdown().await.expect("should succeed");
    handle.abort();
}

#[tokio::test(start_paused = true)]
async fn test_connection_idle_timeout() {
    // Create mock security manager
    let security_manager = Arc::new(RwLock::new(MockSecurityManager::new()));
    
    // Create transport with short idle timeout
    let config = TransportConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0, // Let OS assign port
        max_connections: 5,
        max_message_size: 1024,
        protocol_version: ProtocolVersion::new(1, 0),
        security_level: SecurityLevel::None,
        compression: CompressionFormat::None,
        encryption: EncryptionFormat::None,
        idle_timeout_seconds: 1, // Short timeout for testing
    };
    
    let transport = Arc::new(Transport::with_security_manager(
        config.clone(),
        security_manager.clone()
    ).await.expect("should succeed"));
    
    // Start transport in background
    let transport_clone = transport.clone();
    let handle = tokio::spawn(async move {
        transport_clone.start().await.expect("should succeed");
    });
    
    await_transport_running(&transport).await;
    
    let port = transport.get_port().await.expect("should succeed");
    let addr = SocketAddr::from_str(&format!("127.0.0.1:{}", port)).expect("should succeed");
    let _client = TcpStream::connect(addr).await.expect("should succeed");
    
    await_connections(&transport, 1).await;
    
    // Advance paused time past the idle timeout instead of real sleeping
    tokio::time::advance(Duration::from_secs(2)).await;
    tokio::task::yield_now().await;
    
    await_connections(&transport, 0).await;
    
    // Cleanup
    transport.shutdown().await.expect("should succeed");
    handle.abort();
} 