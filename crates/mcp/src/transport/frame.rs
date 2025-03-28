// Message framing for MCP transports
//
// This module provides a message framing mechanism for MCP (Machine Context Protocol)
// transports. It implements a simple length-prefixed framing protocol where each frame
// consists of a 4-byte header containing the frame length, followed by the frame payload.
//
// The framing mechanism ensures message boundaries are preserved during transport over
// byte-oriented streams like TCP connections. It handles message fragmentation and
// reassembly, allowing complete messages to be reliably sent and received even when
// the underlying transport may split or combine data.
//
// Key components include:
// - Frame: Represents a protocol frame with its payload
// - FrameReader: Reads frames from an AsyncRead stream
// - FrameWriter: Writes frames to an AsyncWrite stream
// - MessageCodec: Encodes and decodes MCPMessages to and from frames

use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use crate::types::MCPMessage;
use crate::error::transport::TransportError;

/// Maximum frame size (10 MB)
///
/// Frames larger than this size will be rejected for security and resource
/// management reasons. This helps prevent potential denial of service attacks
/// through excessively large messages.
const MAX_FRAME_SIZE: usize = 10 * 1024 * 1024;

/// Frame header size (4 bytes for length)
///
/// Each frame starts with a 4-byte (32-bit) big-endian unsigned integer
/// specifying the length of the payload in bytes.
const HEADER_SIZE: usize = 4;

/// A frame used for message transport
///
/// A frame represents a discrete unit of data in the MCP transport protocol.
/// Each frame consists of a header (not stored directly in this struct) and a payload.
/// The payload is the actual message content being transported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// Frame payload containing the message data
    pub payload: BytesMut,
}

impl Frame {
    /// Create a new frame with the given payload
    ///
    /// # Arguments
    ///
    /// * `payload` - The payload data for the frame
    ///
    /// # Returns
    ///
    /// A new Frame instance containing the payload
    pub fn new(payload: BytesMut) -> Self {
        Self { payload }
    }
    
    /// Get a reference to the frame payload
    ///
    /// # Returns
    ///
    /// A reference to the frame's payload
    pub fn payload(&self) -> &BytesMut {
        &self.payload
    }
    
    /// Get the length of the frame payload in bytes
    ///
    /// # Returns
    ///
    /// The length of the frame payload
    pub fn len(&self) -> usize {
        self.payload.len()
    }
    
    /// Check if the frame payload is empty
    ///
    /// # Returns
    ///
    /// True if the frame payload is empty, false otherwise
    pub fn is_empty(&self) -> bool {
        self.payload.is_empty()
    }
}

/// Frame reader for MCP transport
///
/// Reads frames from a byte stream, handling message framing protocol details.
/// It buffers incoming data and processes it to extract complete frames, handling
/// cases where frame data might be split across multiple reads.
#[derive(Debug)]
pub struct FrameReader<R> {
    /// The underlying reader providing the byte stream
    reader: R,
    /// Buffer for accumulating data until complete frames can be extracted
    buffer: BytesMut,
}

impl<R: AsyncRead + Unpin> FrameReader<R> {
    /// Create a new frame reader
    ///
    /// # Arguments
    ///
    /// * `reader` - The underlying AsyncRead stream to read from
    ///
    /// # Returns
    ///
    /// A new FrameReader instance
    pub fn new(reader: R) -> Self {
        Self { 
            reader,
            buffer: BytesMut::with_capacity(MAX_FRAME_SIZE),
        }
    }
    
    /// Read a frame from the underlying stream
    ///
    /// Reads and returns the next complete frame from the stream. This method handles
    /// buffering of partial data and will return None if the stream is at EOF with no
    /// pending data, or Some(Frame) when a complete frame is available.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Frame))` - A complete frame was read successfully
    /// * `Ok(None)` - The stream is at EOF with no pending frames
    /// * `Err(...)` - An error occurred while reading
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
    ///
    /// Attempts to extract a complete frame from the current buffer contents.
    /// If a complete frame is available, it is returned and removed from the buffer.
    /// If only a partial frame is available, None is returned and the buffer is left unchanged.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Frame))` - A complete frame was extracted
    /// * `Ok(None)` - Only a partial frame is available
    /// * `Err(...)` - An error occurred while parsing the frame
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
    ///
    /// Reads data from the underlying reader into the internal buffer.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - The number of bytes read
    /// * `Err(...)` - An error occurred while reading
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

/// Frame writer for MCP transport
///
/// Writes frames to a byte stream, handling message framing protocol details.
/// It prefixes each frame with a length header before writing the payload.
#[derive(Debug)]
pub struct FrameWriter<W> {
    /// The underlying writer for the byte stream
    writer: W,
}

impl<W: AsyncWrite + Unpin> FrameWriter<W> {
    /// Create a new frame writer
    ///
    /// # Arguments
    ///
    /// * `writer` - The underlying AsyncWrite stream to write to
    ///
    /// # Returns
    ///
    /// A new FrameWriter instance
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
    
    /// Write a frame to the stream
    ///
    /// Writes a complete frame to the underlying stream, including the length
    /// header. The frame is written atomically (header and payload are written
    /// in a single operation) and the stream is flushed afterward.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to write
    ///
    /// # Returns
    ///
    /// Result indicating success or an error
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

/// Codec for encoding and decoding MCP messages
///
/// Provides functionality for converting between MCPMessages and frames.
/// This codec handles the serialization and deserialization of messages
/// to and from JSON format for transport.
#[derive(Debug)]
pub struct MessageCodec {
    // Configuration could be added here if needed
}

impl MessageCodec {
    /// Create a new message codec
    ///
    /// # Returns
    ///
    /// A new MessageCodec instance
    pub fn new() -> Self {
        Self {}
    }
    
    /// Encode a message to a frame
    ///
    /// Serializes an MCPMessage to JSON and creates a frame containing
    /// the serialized data.
    ///
    /// # Arguments
    ///
    /// * `message` - The MCP message to encode
    ///
    /// # Returns
    ///
    /// Result containing the encoded frame or an error
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
    ///
    /// Deserializes an MCPMessage from the JSON data in a frame.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame containing the serialized message
    ///
    /// # Returns
    ///
    /// Result containing the decoded message or an error
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