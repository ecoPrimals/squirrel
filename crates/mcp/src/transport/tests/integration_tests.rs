use std::sync::Arc;
use tokio::test;
use tokio::sync::RwLock;
use tokio::net::TcpStream;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use serde_json::json;

use crate::mcp::transport::{Transport, TransportConfig};
use crate::mcp::protocol::{MCPProtocolAdapter, MCPProtocolBase, ProtocolConfig};
use crate::mcp::types::{
    MCPMessage,
    MessageType,
    ProtocolVersion,
    SecurityLevel,
    CompressionFormat,
    EncryptionFormat,
    MessageId,
    MessageMetadata,
    ResponseStatus,
};
use crate::mcp::security::{SecurityManager, SecurityConfig, Credentials};
use crate::test_utils::{TestData, MockSecurityManager, MockProtocolAdapter};

struct IntegrationTestEnvironment {
    transport: Arc<Transport>,
    protocol: Arc<MCPProtocolAdapter>,
    security_manager: Arc<RwLock<MockSecurityManager>>,
    config: TransportConfig,
}

impl IntegrationTestEnvironment {
    async fn new() -> Self {
        // Create mock security manager
        let security_manager = Arc::new(RwLock::new(MockSecurityManager::new()));
        
        // Configure security manager
        {
            let mut manager = security_manager.write().await;
            manager.set_authentication_response(true);
            manager.set_encryption_supported(true);
        }
        
        // Create protocol
        let protocol_config = ProtocolConfig::default();
        let protocol_base = MCPProtocolBase::new(protocol_config);
        let protocol = Arc::new(MCPProtocolAdapter::with_protocol(protocol_base));
        
        // Create transport config
        let config = TransportConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 0, // Let OS assign port
            max_connections: 10,
            max_message_size: 1024 * 10, // 10KB
            protocol_version: ProtocolVersion::new(1, 0),
            security_level: SecurityLevel::None,
            compression: CompressionFormat::None,
            encryption: EncryptionFormat::None,
        };
        
        // Create transport with injected dependencies
        let transport = Arc::new(Transport::with_components(
            config.clone(),
            security_manager.clone(),
            protocol.clone()
        ).await.unwrap());
        
        Self {
            transport,
            protocol,
            security_manager,
            config,
        }
    }
    
    async fn get_port(&self) -> u16 {
        self.transport.get_port().await.unwrap()
    }
}

#[test]
async fn test_transport_protocol_integration() {
    // Create test environment
    let env = IntegrationTestEnvironment::new().await;
    
    // Register command handler on protocol
    let handler = Box::new(TestCommandHandler::new("integration test response"));
    env.protocol.register_handler(MessageType::Command, handler).await.unwrap();
    
    // Start transport in background
    let transport_clone = env.transport.clone();
    let handle = tokio::spawn(async move {
        transport_clone.start().await.unwrap();
    });
    
    // Wait a bit for transport to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Get the assigned port
    let port = env.get_port().await;
    
    // Connect client
    let addr = SocketAddr::from_str(&format!("127.0.0.1:{}", port)).unwrap();
    let mut stream = TcpStream::connect(addr).await.unwrap();
    
    // Create test message
    let message_id = MessageId(format!("test-{}", uuid::Uuid::new_v4()));
    let test_message = MCPMessage {
        protocol_version: "1.0".to_string(),
        id: message_id.clone(),
        message_type: MessageType::Command,
        payload: serde_json::to_vec(&json!({"test": "integration"})).unwrap(),
        metadata: MessageMetadata::default(),
    };
    
    // Send message
    use crate::mcp::transport::frame::{Frame, MessageCodec, FrameReader, FrameWriter};
    let codec = MessageCodec::new();
    let (read_half, write_half) = stream.split();
    let mut reader = FrameReader::new(read_half);
    let mut writer = FrameWriter::new(write_half);
    
    // Encode and send message
    let frame = codec.encode_message(&test_message).await.unwrap();
    writer.write_frame(frame).await.unwrap();
    
    // Wait for response
    let response_frame = reader.read_frame().await.unwrap().unwrap();
    let response: MCPMessage = codec.decode_message(response_frame).await.unwrap();
    
    // Verify response
    assert_eq!(response.message_type, MessageType::CommandResponse);
    assert_eq!(response.id, message_id);
    
    // Decode response payload
    let response_data: serde_json::Value = serde_json::from_slice(&response.payload).unwrap();
    assert_eq!(response_data["response"], "integration test response");
    
    // Cleanup
    env.transport.shutdown().await.unwrap();
    handle.abort();
}

#[test]
async fn test_transport_security_protocol_integration() {
    // Create security manager
    let security_config = SecurityConfig {
        min_security_level: SecurityLevel::Standard,
        encryption_format: EncryptionFormat::AES256GCM,
        token_validity: 3600,
        max_auth_attempts: 3,
    };
    let security_manager = Arc::new(SecurityManager::new(security_config).unwrap());
    
    // Create protocol
    let protocol_config = ProtocolConfig::default();
    let protocol_base = MCPProtocolBase::new(protocol_config);
    let protocol = Arc::new(MCPProtocolAdapter::with_protocol(protocol_base));
    
    // Register command handler on protocol
    let handler = Box::new(TestCommandHandler::new("secure integration test"));
    protocol.register_handler(MessageType::Command, handler).await.unwrap();
    
    // Create transport config with security
    let config = TransportConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0, // Let OS assign port
        max_connections: 10,
        max_message_size: 1024 * 10, // 10KB
        protocol_version: ProtocolVersion::new(1, 0),
        security_level: SecurityLevel::Standard,
        compression: CompressionFormat::None,
        encryption: EncryptionFormat::AES256GCM,
    };
    
    // Create transport with injected dependencies
    let transport = Arc::new(Transport::with_components(
        config.clone(),
        security_manager.clone(),
        protocol.clone()
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
    let mut stream = TcpStream::connect(addr).await.unwrap();
    
    // Setup codec for communication
    use crate::mcp::transport::frame::{Frame, MessageCodec, FrameReader, FrameWriter};
    let codec = MessageCodec::new();
    let (read_half, write_half) = stream.split();
    let mut reader = FrameReader::new(read_half);
    let mut writer = FrameWriter::new(write_half);
    
    // Perform handshake
    let handshake_frame = reader.read_frame().await.unwrap().unwrap();
    let handshake = codec.decode_message(handshake_frame).await.unwrap();
    assert_eq!(handshake.message_type, MessageType::Handshake);
    
    // Send handshake response
    let response = MCPMessage::new(
        MessageType::HandshakeResponse,
        ProtocolVersion::new(1, 0),
        SecurityLevel::Standard,
        Vec::new(),
    );
    let frame = codec.encode_message(&response).await.unwrap();
    writer.write_frame(frame).await.unwrap();
    
    // Authenticate
    let credentials = Credentials {
        username: "test_user".to_string(),
        password: "test_password".to_string(),
        security_level: SecurityLevel::Standard,
    };
    
    let auth_request = MCPMessage::new(
        MessageType::AuthRequest,
        ProtocolVersion::new(1, 0),
        SecurityLevel::Standard,
        serde_json::to_vec(&credentials).unwrap(),
    );
    
    let frame = codec.encode_message(&auth_request).await.unwrap();
    writer.write_frame(frame).await.unwrap();
    
    // Receive auth response
    let auth_frame = reader.read_frame().await.unwrap().unwrap();
    let auth_response = codec.decode_message(auth_frame).await.unwrap();
    assert_eq!(auth_response.message_type, MessageType::AuthResponse);
    
    // Now send a secure command
    let message_id = MessageId(format!("test-{}", uuid::Uuid::new_v4()));
    let secure_message = MCPMessage {
        protocol_version: "1.0".to_string(),
        id: message_id.clone(),
        message_type: MessageType::Command,
        payload: serde_json::to_vec(&json!({"test": "secure"})).unwrap(),
        metadata: MessageMetadata {
            security_level: SecurityLevel::Standard,
            session_id: Some("test-session".to_string()),
            ..MessageMetadata::default()
        },
    };
    
    let frame = codec.encode_message(&secure_message).await.unwrap();
    writer.write_frame(frame).await.unwrap();
    
    // Receive response
    let response_frame = reader.read_frame().await.unwrap().unwrap();
    let response = codec.decode_message(response_frame).await.unwrap();
    
    // Verify secure response
    assert_eq!(response.message_type, MessageType::CommandResponse);
    assert_eq!(response.id, message_id);
    assert_eq!(response.metadata.security_level, SecurityLevel::Standard);
    
    // Decode response payload
    let response_data: serde_json::Value = serde_json::from_slice(&response.payload).unwrap();
    assert_eq!(response_data["response"], "secure integration test");
    
    // Cleanup
    transport.shutdown().await.unwrap();
    handle.abort();
}

// Test command handler for integration tests
#[derive(Clone)]
struct TestCommandHandler {
    response_text: String,
}

impl TestCommandHandler {
    fn new(response_text: &str) -> Self {
        Self {
            response_text: response_text.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl crate::mcp::protocol::CommandHandler for TestCommandHandler {
    async fn handle(&self, message: &MCPMessage) -> crate::mcp::error::Result<crate::mcp::types::MCPResponse> {
        // Create response payload
        let request: serde_json::Value = serde_json::from_slice(&message.payload).unwrap_or(json!({}));
        let response_payload = json!({
            "response": self.response_text,
            "request": request,
        });
        
        // Create response
        Ok(crate::mcp::types::MCPResponse {
            protocol_version: message.protocol_version.clone(),
            message_id: message.id.0.clone(),
            status: ResponseStatus::Success,
            payload: serde_json::to_vec(&response_payload).unwrap(),
            error_message: None,
            metadata: message.metadata.clone(),
        })
    }
} 