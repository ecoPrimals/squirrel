use std::io;
use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use serde::{Serialize, Deserialize};
use crate::error::{MCPError, Result};
use crate::types::MCPMessage;
use crate::error::transport::TransportError;

/// Maximum frame size (10 MB)
const MAX_FRAME_SIZE: usize = 10 * 1024 * 1024;

/// Frame header size (4 bytes for length)
const HEADER_SIZE: usize = 4;

/// A frame used for message transport
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// Frame payload
    pub payload: BytesMut,
}

impl Frame {
    /// Create a new frame with the given payload
    pub fn new(payload: BytesMut) -> Self {
        Self { payload }
    }
    
    /// Get a reference to the frame payload
    pub fn payload(&self) -> &BytesMut {
        &self.payload
    }
    
    /// Get the length of the frame payload
    pub fn len(&self) -> usize {
        self.payload.len()
    }
    
    /// Check if the frame payload is empty
    pub fn is_empty(&self) -> bool {
        self.payload.is_empty()
    }
}

/// A reader for frames from an async stream
pub struct FrameReader<R> {
    reader: R,
    buffer: BytesMut,
}

impl<R: AsyncRead + Unpin> FrameReader<R> {
    /// Create a new frame reader
    pub fn new(reader: R) -> Self {
        Self { 
            reader,
            buffer: BytesMut::with_capacity(MAX_FRAME_SIZE),
        }
    }
    
    /// Read a frame from the underlying stream
    pub async fn read_frame(&mut self) -> crate::error::Result<Option<Frame>> {
        // If the buffer is empty, try to read some data
        if self.buffer.is_empty() {
            if self.read_to_buffer().await? == 0 {
                // EOF, no more frames
                return Ok(None);
            }
        }
        
        // Check if we have at least a complete header
        if self.buffer.len() < 4 {
            // Not enough data for header, try to read more
            if self.read_to_buffer().await? == 0 {
                // EOF, incomplete frame
                if !self.buffer.is_empty() {
                    return Err(TransportError::InvalidFrame("Incomplete frame at end of stream".into()).into());
                }
                return Ok(None);
            }
            
            // Still not enough for header?
            if self.buffer.len() < 4 {
                return Err(TransportError::InvalidFrame("Incomplete frame header".into()).into());
            }
        }
        
        // Parse and return the frame
        self.parse_frame()
    }
    
    /// Parse a frame from the current buffer
    fn parse_frame(&mut self) -> crate::error::Result<Option<Frame>> {
        // Read the frame length from the header
        let mut header = [0u8; 4];
        header.copy_from_slice(&self.buffer[0..4]);
        let frame_len = u32::from_be_bytes(header) as usize;
        
        // Check if the frame is too large
        if frame_len > MAX_FRAME_SIZE {
            return Err(TransportError::InvalidFrame(format!(
                "Frame too large: {} bytes (max is {} bytes)",
                frame_len, MAX_FRAME_SIZE
            )).into());
        }
        
        // Check if we have the complete frame
        if self.buffer.len() < 4 + frame_len {
            // Not enough data, don't consume anything
            return Ok(None);
        }
        
        // We have a complete frame, consume it
        self.buffer.advance(4); // Skip header
        let payload = self.buffer.split_to(frame_len);
        
        // Return the frame
        Ok(Some(Frame::new(payload)))
    }
    
    /// Read more data into the buffer
    async fn read_to_buffer(&mut self) -> crate::error::Result<usize> {
        // Make sure we have capacity for at least the header
        if self.buffer.capacity() < 4 {
            self.buffer.reserve(4);
        }
        
        // Read into the buffer
        let bytes_read = self.reader.read_buf(&mut self.buffer).await
            .map_err(|e| {
                let transport_err: crate::error::MCPError = TransportError::IoError(e).into();
                transport_err
            })?;
        
        Ok(bytes_read)
    }
}

/// A writer for frames to an async stream
pub struct FrameWriter<W> {
    writer: W,
}

impl<W: AsyncWrite + Unpin> FrameWriter<W> {
    /// Create a new frame writer
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
    
    /// Write a frame to the stream
    pub async fn write_frame(&mut self, frame: Frame) -> crate::error::Result<()> {
        // Check frame size
        if frame.len() > MAX_FRAME_SIZE {
            let error_msg = format!(
                "Frame too large: {} bytes (max is {} bytes)",
                frame.len(), MAX_FRAME_SIZE
            );
            let transport_err: crate::error::MCPError = TransportError::InvalidFrame(error_msg).into();
            return Err(transport_err);
        }
        
        // Write frame length header
        let header = (frame.len() as u32).to_be_bytes();
        self.writer.write_all(&header).await
            .map_err(|e| {
                let transport_err: crate::error::MCPError = TransportError::IoError(e).into();
                transport_err
            })?;
        
        // Write frame payload
        self.writer.write_all(&frame.payload).await
            .map_err(|e| {
                let transport_err: crate::error::MCPError = TransportError::IoError(e).into();
                transport_err
            })?;
        
        // Flush the writer
        self.writer.flush().await
            .map_err(|e| {
                let transport_err: crate::error::MCPError = TransportError::IoError(e).into();
                transport_err
            })?;
        
        Ok(())
    }
}

/// A codec for encoding and decoding messages to/from frames
pub struct MessageCodec {
    // Configuration could be added here if needed
}

impl MessageCodec {
    /// Create a new message codec
    pub fn new() -> Self {
        Self {}
    }
    
    /// Encode a message to a frame
    pub async fn encode_message(&self, message: &MCPMessage) -> crate::error::Result<Frame> {
        // Serialize the message to JSON
        let json = serde_json::to_vec(message)
            .map_err(|e| {
                let transport_err: crate::error::MCPError = TransportError::SerializationError(e).into();
                transport_err
            })?;
        
        // Create a frame
        let mut buffer = BytesMut::with_capacity(json.len());
        buffer.put_slice(&json);
        
        Ok(Frame::new(buffer))
    }
    
    /// Decode a message from a frame
    pub async fn decode_message(&self, frame: &Frame) -> crate::error::Result<MCPMessage> {
        // Deserialize the message from JSON
        let message = serde_json::from_slice(&frame.payload)
            .map_err(|e| {
                let transport_err: crate::error::MCPError = TransportError::SerializationError(e).into();
                transport_err
            })?;
        
        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{MessageType, MessageId};
    
    #[tokio::test]
    async fn test_frame_round_trip() {
        // Create a message
        let message = MCPMessage::new(
            MessageType::Command,
            serde_json::json!({ "test": "value" }),
        );
        
        // Create codec
        let codec = MessageCodec::new();
        
        // Encode
        let frame = codec.encode_message(&message).await.unwrap();
        
        // Decode
        let decoded = codec.decode_message(&frame).await.unwrap();
        
        // Verify
        assert_eq!(decoded.type_, message.type_);
        assert_eq!(decoded.payload, message.payload);
    }
    
    #[tokio::test]
    async fn test_frame_reader_writer() {
        // Create a message
        let message = MCPMessage::new(
            MessageType::Command,
            serde_json::json!({ "test": "value" }),
        );
        
        // Create codec
        let codec = MessageCodec::new();
        
        // Encode
        let frame = codec.encode_message(&message).await.unwrap();
        
        // Create a buffer
        let mut buffer = Vec::new();
        
        // Write frame
        {
            let mut writer = FrameWriter::new(&mut buffer);
            writer.write_frame(frame).await.unwrap();
        }
        
        // Read frame
        let mut reader = FrameReader::new(&buffer[..]);
        let read_frame = reader.read_frame().await.unwrap().unwrap();
        
        // Decode
        let decoded = codec.decode_message(&read_frame).await.unwrap();
        
        // Verify
        assert_eq!(decoded.type_, message.type_);
        assert_eq!(decoded.payload, message.payload);
    }
} 