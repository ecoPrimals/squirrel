use super::*;
use tokio::net::TcpStream;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use crate::mcp::security::Credentials;

#[tokio::test]
async fn test_transport_connection() {
    // Create transport with custom config
    let config = TransportConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 9001,
        max_connections: 1,
        max_message_size: 1024,
        protocol_version: ProtocolVersion::new(1, 0),
        security_level: SecurityLevel::None,
        compression: CompressionFormat::None,
        encryption: EncryptionFormat::None,
    };

    let transport = Transport::new(config.clone()).await.unwrap();
    
    // Start transport in background
    tokio::spawn(async move {
        transport.start().await.unwrap();
    });

    // Wait for transport to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect client
    let addr = SocketAddr::from_str("127.0.0.1:9001").unwrap();
    let stream = TcpStream::connect(addr).await.unwrap();
    
    // Create client codec
    let codec = MessageCodec::new();
    let (read_half, write_half) = stream.split();
    let mut reader = FrameReader::new(read_half);
    let mut writer = FrameWriter::new(write_half);

    // Expect handshake message
    let frame = reader.read_frame().await.unwrap().unwrap();
    let message = codec.decode_message(frame).await.unwrap();
    assert_eq!(message.message_type, MessageType::Handshake);
    
    // Send handshake response
    let response = MCPMessage::new(
        MessageType::HandshakeResponse,
        config.protocol_version,
        config.security_level,
        Vec::new(),
    );
    
    let frame = codec.encode_message(&response).await.unwrap();
    writer.write_frame(frame).await.unwrap();
}

#[tokio::test]
async fn test_message_exchange() {
    // Create transport
    let config = TransportConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 9002,
        max_connections: 1,
        max_message_size: 1024,
        protocol_version: ProtocolVersion::new(1, 0),
        security_level: SecurityLevel::None,
        compression: CompressionFormat::None,
        encryption: EncryptionFormat::None,
    };

    let mut transport = Transport::new(config.clone()).await.unwrap();
    
    // Start transport in background
    let transport_handle = transport.clone();
    tokio::spawn(async move {
        transport_handle.start().await.unwrap();
    });

    // Wait for transport to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect client
    let addr = SocketAddr::from_str("127.0.0.1:9002").unwrap();
    let stream = TcpStream::connect(addr).await.unwrap();
    
    // Setup client message handling
    let codec = MessageCodec::new();
    let (read_half, write_half) = stream.split();
    let mut reader = FrameReader::new(read_half);
    let mut writer = FrameWriter::new(write_half);

    // Complete handshake
    let frame = reader.read_frame().await.unwrap().unwrap();
    let message = codec.decode_message(frame).await.unwrap();
    assert_eq!(message.message_type, MessageType::Handshake);
    
    let response = MCPMessage::new(
        MessageType::HandshakeResponse,
        config.protocol_version,
        config.security_level,
        Vec::new(),
    );
    
    let frame = codec.encode_message(&response).await.unwrap();
    writer.write_frame(frame).await.unwrap();

    // Send test message
    let test_message = MCPMessage::new(
        MessageType::Command,
        config.protocol_version,
        config.security_level,
        b"test payload".to_vec(),
    );
    
    let frame = codec.encode_message(&test_message).await.unwrap();
    writer.write_frame(frame).await.unwrap();

    // Verify message received by transport
    let received = transport.receive_message().await.unwrap().unwrap();
    assert_eq!(received.message_type, MessageType::Command);
    assert_eq!(received.payload, b"test payload".to_vec());
}

#[tokio::test]
async fn test_connection_limits() {
    // Create transport with single connection limit
    let config = TransportConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 9003,
        max_connections: 1,
        max_message_size: 1024,
        protocol_version: ProtocolVersion::new(1, 0),
        security_level: SecurityLevel::None,
        compression: CompressionFormat::None,
        encryption: EncryptionFormat::None,
    };

    let transport = Transport::new(config).await.unwrap();
    
    // Start transport in background
    tokio::spawn(async move {
        transport.start().await.unwrap();
    });

    // Wait for transport to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect first client
    let addr = SocketAddr::from_str("127.0.0.1:9003").unwrap();
    let _stream1 = TcpStream::connect(addr).await.unwrap();

    // Second connection should fail
    tokio::time::sleep(Duration::from_millis(100)).await;
    let stream2 = TcpStream::connect(addr).await;
    assert!(stream2.is_err());
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpStream;
    use std::time::Duration;
    use crate::mcp::security::Credentials;

    #[tokio::test]
    async fn test_secure_connection() -> Result<()> {
        // Create transport with security enabled
        let config = TransportConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 9004,
            max_connections: 10,
            protocol_version: ProtocolVersion::V1_0,
            security_level: SecurityLevel::Standard,
            encryption: EncryptionFormat::AES256GCM,
            compression: CompressionFormat::None,
        };

        let transport = Transport::new(config).await?;
        transport.start().await?;

        // Connect client
        let stream = TcpStream::connect("127.0.0.1:9004").await?;
        let codec = MessageCodec::new();
        let (read_half, write_half) = stream.split();
        let mut frame_reader = FrameReader::new(read_half);
        let mut frame_writer = FrameWriter::new(write_half);

        // Receive handshake
        let frame = frame_reader.read_frame().await?.unwrap();
        let handshake = codec.decode_message(frame).await?;
        assert_eq!(handshake.message_type, MessageType::Handshake);

        // Send handshake response
        let response = MCPMessage::new(
            MessageType::HandshakeResponse,
            ProtocolVersion::V1_0,
            SecurityLevel::None,
            Vec::new(),
        );
        let frame = codec.encode_message(&response).await?;
        frame_writer.write_frame(frame).await?;

        // Send auth request
        let credentials = Credentials {
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
            security_level: SecurityLevel::Standard,
        };
        let auth_request = MCPMessage::new(
            MessageType::AuthRequest,
            ProtocolVersion::V1_0,
            SecurityLevel::Standard,
            serde_json::to_vec(&credentials)?,
        );
        let frame = codec.encode_message(&auth_request).await?;
        frame_writer.write_frame(frame).await?;

        // Receive auth response
        let frame = frame_reader.read_frame().await?.unwrap();
        let auth_response = codec.decode_message(frame).await?;
        assert_eq!(auth_response.message_type, MessageType::AuthResponse);
        
        // Extract token from response
        let token = String::from_utf8(auth_response.payload)?;
        assert!(!token.is_empty());

        // Send secure message
        let secure_payload = b"This is a secure message".to_vec();
        let secure_message = MCPMessage::new(
            MessageType::Command,
            ProtocolVersion::V1_0,
            SecurityLevel::Standard,
            secure_payload.clone(),
        );
        let frame = codec.encode_message(&secure_message).await?;
        frame_writer.write_frame(frame).await?;

        // Verify message was received and decrypted
        let received = transport.receive_message().await?;
        assert_eq!(received.payload, secure_payload);

        Ok(())
    }

    #[tokio::test]
    async fn test_security_level_enforcement() -> Result<()> {
        // Create transport with high security level
        let config = TransportConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 9005,
            max_connections: 10,
            protocol_version: ProtocolVersion::V1_0,
            security_level: SecurityLevel::High,
            encryption: EncryptionFormat::AES256GCM,
            compression: CompressionFormat::None,
        };

        let transport = Transport::new(config).await?;
        transport.start().await?;

        // Connect client
        let stream = TcpStream::connect("127.0.0.1:9005").await?;
        let codec = MessageCodec::new();
        let (read_half, write_half) = stream.split();
        let mut frame_reader = FrameReader::new(read_half);
        let mut frame_writer = FrameWriter::new(write_half);

        // Complete handshake
        let frame = frame_reader.read_frame().await?.unwrap();
        let handshake = codec.decode_message(frame).await?;
        assert_eq!(handshake.message_type, MessageType::Handshake);

        let response = MCPMessage::new(
            MessageType::HandshakeResponse,
            ProtocolVersion::V1_0,
            SecurityLevel::None,
            Vec::new(),
        );
        let frame = codec.encode_message(&response).await?;
        frame_writer.write_frame(frame).await?;

        // Try to authenticate with standard security level (should fail)
        let credentials = Credentials {
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
            security_level: SecurityLevel::Standard,
        };
        let auth_request = MCPMessage::new(
            MessageType::AuthRequest,
            ProtocolVersion::V1_0,
            SecurityLevel::Standard,
            serde_json::to_vec(&credentials)?,
        );
        let frame = codec.encode_message(&auth_request).await?;
        frame_writer.write_frame(frame).await?;

        // Receive auth response (should be error)
        let frame = frame_reader.read_frame().await?.unwrap();
        let auth_response = codec.decode_message(frame).await?;
        assert_eq!(auth_response.message_type, MessageType::Error);

        Ok(())
    }
} 