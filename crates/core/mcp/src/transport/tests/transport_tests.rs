// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::sync::Arc;
use tokio::test;
use tokio::sync::RwLock;
use tokio::net::TcpStream;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use serde_json::json;

use crate::mcp::transport::{Transport, TransportConfig};
// BearDog handles security: // use crate::mcp::security::{SecurityManager, SecurityConfig, Credentials};
use crate::mcp::types::{
    MCPMessage,
    MessageType,
    ProtocolVersion,
    SecurityLevel,
    CompressionFormat,
    EncryptionFormat,
};
use crate::test_utils::{TestData, MockSecurityManager};

// Mock security manager for testing
struct MockTransportEnvironment {
    transport: Arc<Transport>,
    security_manager: Arc<RwLock<MockSecurityManager>>,
    config: TransportConfig,
}

impl MockTransportEnvironment {
    async fn new() -> Self {
        // Create mock security manager
        let security_manager = Arc::new(RwLock::new(MockSecurityManager::new()));
        
        // Create config with test values
        let config = TransportConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 0, // Use port 0 to let OS assign a free port
            max_connections: 10,
            max_message_size: 1024 * 10, // 10KB
            protocol_version: ProtocolVersion::new(1, 0),
            security_level: SecurityLevel::None,
            compression: CompressionFormat::None,
            encryption: EncryptionFormat::None,
        };
        
        // Create transport with mocked security manager
        let transport = Arc::new(Transport::with_security_manager(
            config.clone(),
            security_manager.clone()
        ).await.expect("should succeed"));
        
        Self {
            transport,
            security_manager,
            config,
        }
    }
    
    async fn with_security(security_level: SecurityLevel, encryption: EncryptionFormat) -> Self {
        // Create mock security manager
        let security_manager = Arc::new(RwLock::new(MockSecurityManager::new()));
        
        // Initialize security manager for the test
        {
            let mut manager = security_manager.write().await;
            manager.set_authentication_response(true);
            manager.set_encryption_supported(true);
        }
        
        // Create config with security enabled
        let config = TransportConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 0, // Use port 0 to let OS assign a free port
            max_connections: 10,
            max_message_size: 1024 * 10, // 10KB
            protocol_version: ProtocolVersion::new(1, 0),
            security_level,
            compression: CompressionFormat::None,
            encryption,
        };
        
        // Create transport with mocked security manager
        let transport = Arc::new(Transport::with_security_manager(
            config.clone(),
            security_manager.clone()
        ).await.expect("should succeed"));
        
        Self {
            transport,
            security_manager,
            config,
        }
    }
    
    async fn get_port(&self) -> u16 {
        // Read the actual port assigned by the OS
        self.transport.get_port().await.expect("should succeed")
    }
}

#[test]
async fn test_transport_creation() {
    // Create transport with default config
    let config = TransportConfig::default();
    let transport = Transport::new(config.clone()).await.expect("should succeed");
    
    // Verify transport was created with the correct settings
    assert!(!transport.is_running().await);
    assert_eq!(transport.get_config().await.expect("should succeed").port, config.port);
}

#[test]
async fn test_transport_with_injected_security_manager() {
    // Create mock security manager
    let security_config = SecurityConfig {
        min_security_level: SecurityLevel::None,
        encryption_format: EncryptionFormat::None,
        token_validity: 3600,
        max_auth_attempts: 3,
    };
    let security_manager = Arc::new(SecurityManager::new(security_config).expect("should succeed"));
    
    // Create transport with injected security manager
    let config = TransportConfig::default();
    let transport = Transport::with_security_manager(
        config.clone(),
        security_manager.clone()
    ).await.expect("should succeed");
    
    // Verify transport was created with the injected security manager
    assert!(!transport.is_running().await);
    assert_eq!(transport.get_config().await.expect("should succeed").port, config.port);
}

#[test]
async fn test_transport_start_and_stop() {
    // Create test environment
    let env = MockTransportEnvironment::new().await;
    
    // Start transport in background
    let transport_clone = env.transport.clone();
    let handle = tokio::spawn(async move {
        transport_clone.start().await.expect("should succeed");
    });
    
    // Wait a bit for transport to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Check transport is running
    assert!(env.transport.is_running().await);
    
    // Stop transport
    env.transport.shutdown().await.expect("should succeed");
    
    // Wait for transport to shut down
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify transport is stopped
    assert!(!env.transport.is_running().await);
    
    // Abort the spawned task
    handle.abort();
}

#[test]
async fn test_client_connection() {
    // Create test environment
    let env = MockTransportEnvironment::new().await;
    
    // Start transport in background
    let transport_clone = env.transport.clone();
    let handle = tokio::spawn(async move {
        transport_clone.start().await.expect("should succeed");
    });
    
    // Wait a bit for transport to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Get the assigned port
    let port = env.get_port().await;
    
    // Connect client
    let addr = SocketAddr::from_str(&format!("127.0.0.1:{}", port)).expect("should succeed");
    let stream = TcpStream::connect(addr).await.expect("should succeed");
    
    // Wait a bit for connection to complete
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify connection was accepted
    let active_connections = env.transport.get_active_connections().await.expect("should succeed");
    assert_eq!(active_connections, 1);
    
    // Cleanup
    env.transport.shutdown().await.expect("should succeed");
    handle.abort();
}

#[test]
async fn test_connection_limits() {
    // Create environment with limited connections
    let security_manager = Arc::new(RwLock::new(MockSecurityManager::new()));
    
    let config = TransportConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0, // Let OS assign port
        max_connections: 1, // Only allow one connection
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
    
    // Wait a bit for transport to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Get the assigned port
    let port = transport.get_port().await.expect("should succeed");
    
    // Connect first client - should succeed
    let addr = SocketAddr::from_str(&format!("127.0.0.1:{}", port)).expect("should succeed");
    let _stream1 = TcpStream::connect(addr).await.expect("should succeed");
    
    // Wait for connection to be established
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify one connection is active
    let active_connections = transport.get_active_connections().await.expect("should succeed");
    assert_eq!(active_connections, 1);
    
    // Try to connect second client - should fail or be rejected by transport
    // Note: In some implementations, the connection might be accepted but immediately closed
    // Let's try to connect and then check if we still have only one active connection
    let _stream2_result = TcpStream::connect(addr).await;
    
    // Wait a bit for potential connection to be processed
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify still only one connection is active
    let active_connections = transport.get_active_connections().await.expect("should succeed");
    assert_eq!(active_connections, 1);
    
    // Cleanup
    transport.shutdown().await.expect("should succeed");
    handle.abort();
}

#[test]
async fn test_message_sending() {
    // Create test environment
    let env = MockTransportEnvironment::new().await;
    
    // Start transport in background
    let transport_clone = env.transport.clone();
    let handle = tokio::spawn(async move {
        transport_clone.start().await.expect("should succeed");
    });
    
    // Wait a bit for transport to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Get the assigned port
    let port = env.get_port().await;
    
    // Connect client
    let addr = SocketAddr::from_str(&format!("127.0.0.1:{}", port)).expect("should succeed");
    let mut stream = TcpStream::connect(addr).await.expect("should succeed");
    
    // Create and prepare test message
    let test_message = MCPMessage::new(
        MessageType::Command,
        ProtocolVersion::new(1, 0),
        SecurityLevel::None,
        TestData::create_test_payload(),
    );
    
    // Serialize and send message directly to transport
    use crate::mcp::transport::frame::{Frame, MessageCodec, FrameWriter};
    let codec = MessageCodec::new();
    let (read_half, write_half) = stream.split();
    let mut writer = FrameWriter::new(write_half);
    
    // Encode and send message
    let frame = codec.encode_message(&test_message).await.expect("should succeed");
    writer.write_frame(frame).await.expect("should succeed");
    
    // Wait a bit for message to be processed
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Try to receive message from transport
    let mut transport_receiver = env.transport.clone();
    let received = transport_receiver.receive_message().await.expect("should succeed");
    
    // Verify message was received correctly
    assert!(received.is_some());
    let received = received.expect("should succeed");
    assert_eq!(received.message_type, test_message.message_type);
    assert_eq!(received.protocol_version, test_message.protocol_version);
    assert_eq!(received.payload, test_message.payload);
    
    // Cleanup
    env.transport.shutdown().await.expect("should succeed");
    handle.abort();
} 