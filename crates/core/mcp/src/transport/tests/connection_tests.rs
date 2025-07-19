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

// Helper function to create a TCP socket pair (server and client)
async fn create_socket_pair() -> (TcpStream, TcpStream) {
    // Create a TCP listener on a random port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    // Connect client to server
    let client_stream = TcpStream::connect(addr).await.unwrap();
    
    // Accept connection on server side
    let (server_stream, _) = listener.accept().await.unwrap();
    
    (server_stream, client_stream)
}

#[test]
async fn test_connection_creation() {
    // Create server and client streams
    let (server_stream, _client_stream) = create_socket_pair().await;
    
    // Create connection
    let connection = Connection::new(
        server_stream,
        server_stream.peer_addr().unwrap(),
    );
    
    // Verify connection properties
    assert!(!connection.id.is_empty());
    assert_eq!(connection.state, ConnectionState::New);
    assert_eq!(connection.remote_addr, server_stream.peer_addr().unwrap());
    
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
        server_stream.peer_addr().unwrap(),
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
        server_stream.peer_addr().unwrap(),
    );
    
    // Get initial last_activity time
    let initial_activity = connection.last_activity;
    
    // Wait a moment
    tokio::time::sleep(Duration::from_millis(5)).await;
    
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
    ).await.unwrap());
    
    // Start transport in background
    let transport_clone = transport.clone();
    let handle = tokio::spawn(async move {
        transport_clone.start().await.unwrap();
    });
    
    // Wait a bit for transport to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Get the assigned port
    let port = transport.get_port().await.unwrap();
    
    // Connect multiple clients
    let addr = SocketAddr::from_str(&format!("127.0.0.1:{}", port)).unwrap();
    let client1 = TcpStream::connect(addr).await.unwrap();
    let client2 = TcpStream::connect(addr).await.unwrap();
    
    // Wait for connections to be established
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify connections were created and tracked
    let active_connections = transport.get_active_connections().await.unwrap();
    assert_eq!(active_connections, 2);
    
    // Get connection details
    let connections = transport.get_connection_states().await.unwrap();
    assert_eq!(connections.len(), 2);
    
    // Verify connection states
    for conn in connections {
        assert_eq!(conn.state, ConnectionState::Active);
        assert!(conn.id.len() > 0);
    }
    
    // Close one client
    drop(client1);
    
    // Wait for transport to detect closed connection
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Verify connection count decreased
    let active_connections = transport.get_active_connections().await.unwrap();
    assert_eq!(active_connections, 1);
    
    // Cleanup
    drop(client2);
    transport.shutdown().await.unwrap();
    handle.abort();
}

#[test]
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
    ).await.unwrap());
    
    // Start transport in background
    let transport_clone = transport.clone();
    let handle = tokio::spawn(async move {
        transport_clone.start().await.unwrap();
    });
    
    // Wait a bit for transport to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Get the assigned port
    let port = transport.get_port().await.unwrap();
    
    // Connect client
    let addr = SocketAddr::from_str(&format!("127.0.0.1:{}", port)).unwrap();
    let _client = TcpStream::connect(addr).await.unwrap();
    
    // Wait for connection to be established
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify connection was created
    let active_connections = transport.get_active_connections().await.unwrap();
    assert_eq!(active_connections, 1);
    
    // Wait for connection to time out (a bit more than the idle timeout)
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Verify connection was closed due to idle timeout
    let active_connections = transport.get_active_connections().await.unwrap();
    assert_eq!(active_connections, 0);
    
    // Cleanup
    transport.shutdown().await.unwrap();
    handle.abort();
} 