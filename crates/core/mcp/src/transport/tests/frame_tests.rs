use std::sync::Arc;
use tokio::test;
use bytes::BytesMut;
use tokio::io::duplex;

use crate::mcp::transport::frame::{Frame, MessageCodec, FrameReader, FrameWriter};
use crate::mcp::types::{MCPMessage, MessageType, ProtocolVersion, SecurityLevel};
use crate::test_utils::TestData;

#[test]
async fn test_frame_serialization_and_parsing() {
    // Create test payload
    let test_payload = BytesMut::from(&b"This is a test payload"[..]);
    
    // Create a frame
    let frame = Frame::new(test_payload.clone());
    
    // Serialize the frame
    let serialized = frame.serialize();
    
    // Parse the frame back
    let mut buffer = serialized.clone();
    let parsed_frame = Frame::parse(&mut buffer).unwrap().unwrap();
    
    // Verify the parsed frame matches the original
    assert_eq!(&parsed_frame.payload[..], &test_payload[..]);
}

#[test]
async fn test_frame_check_incomplete() {
    // Create an incomplete frame (only header, no payload)
    let mut buffer = BytesMut::with_capacity(8);
    buffer.extend_from_slice(&0x4D435000u32.to_be_bytes()); // MCP\0 magic
    buffer.extend_from_slice(&10u32.to_be_bytes()); // Payload length = 10
    
    // Check should return None for incomplete frame
    let check_result = Frame::check_frame(&buffer);
    assert!(check_result.is_none());
}

#[test]
async fn test_frame_check_invalid_magic() {
    // Create frame with invalid magic
    let mut buffer = BytesMut::with_capacity(8);
    buffer.extend_from_slice(&0x12345678u32.to_be_bytes()); // Invalid magic
    buffer.extend_from_slice(&10u32.to_be_bytes()); // Payload length = 10
    
    // Check should return None for invalid magic
    let check_result = Frame::check_frame(&buffer);
    assert!(check_result.is_none());
}

#[test]
async fn test_message_codec() {
    // Create test message
    let message = MCPMessage::new(
        MessageType::Command,
        ProtocolVersion::new(1, 0),
        SecurityLevel::None,
        TestData::create_test_payload(),
    );
    
    // Create codec
    let codec = MessageCodec::new();
    
    // Encode message to frame
    let frame = codec.encode_message(&message).await.unwrap();
    
    // Decode frame back to message
    let decoded = codec.decode_message(frame).await.unwrap();
    
    // Verify decoded message matches original
    assert_eq!(decoded.message_type, message.message_type);
    assert_eq!(decoded.protocol_version, message.protocol_version);
    assert_eq!(decoded.security_level, message.security_level);
    assert_eq!(decoded.payload, message.payload);
}

#[test]
async fn test_frame_reader_and_writer() {
    // Create duplex channel for testing
    let (client, server) = duplex(1024);
    
    // Create reader and writer
    let mut reader = FrameReader::new(client);
    let mut writer = FrameWriter::new(server);
    
    // Create test frame
    let test_payload = BytesMut::from(&b"Test frame payload"[..]);
    let frame = Frame::new(test_payload.clone());
    
    // Write frame
    writer.write_frame(frame).await.unwrap();
    
    // Read frame
    let read_frame = reader.read_frame().await.unwrap().unwrap();
    
    // Verify read frame matches original
    assert_eq!(&read_frame.payload[..], &test_payload[..]);
}

#[test]
async fn test_frame_reader_with_multiple_frames() {
    // Create duplex channel for testing
    let (client, server) = duplex(1024);
    
    // Create reader and writer
    let mut reader = FrameReader::new(client);
    let mut writer = FrameWriter::new(server);
    
    // Create multiple test frames
    let payloads = vec![
        BytesMut::from(&b"Frame 1"[..]),
        BytesMut::from(&b"Frame 2"[..]),
        BytesMut::from(&b"Frame 3"[..]),
    ];
    
    // Write all frames
    for payload in &payloads {
        let frame = Frame::new(payload.clone());
        writer.write_frame(frame).await.unwrap();
    }
    
    // Read all frames and verify
    for expected_payload in &payloads {
        let read_frame = reader.read_frame().await.unwrap().unwrap();
        assert_eq!(&read_frame.payload[..], &expected_payload[..]);
    }
}

#[test]
async fn test_reader_with_fragmented_data() {
    // Create duplex channel for testing
    let (client, server) = duplex(1024);
    
    // Create reader
    let mut reader = FrameReader::new(client);
    
    // Create test frame
    let test_payload = BytesMut::from(&b"Fragmented frame test"[..]);
    let frame = Frame::new(test_payload.clone());
    let serialized = frame.serialize();
    
    // Write data in fragments (manually)
    use tokio::io::AsyncWriteExt;
    
    // Write part of header
    server.write_all(&serialized[0..4]).await.unwrap();
    
    // No complete frame yet
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    
    // Write rest of header
    server.write_all(&serialized[4..8]).await.unwrap();
    
    // Write part of payload
    server.write_all(&serialized[8..12]).await.unwrap();
    
    // Still no complete frame
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    
    // Write rest of payload
    server.write_all(&serialized[12..]).await.unwrap();
    
    // Should now be able to read complete frame
    let read_frame = reader.read_frame().await.unwrap().unwrap();
    assert_eq!(&read_frame.payload[..], &test_payload[..]);
} 