// Frame implementation for MCP transport
//
// This module provides a framing mechanism for MCP messages sent over byte streams.
// It handles encoding/decoding messages into frames and preserving message boundaries.
//
// The framing protocol is simple:
// - Each frame starts with a 4-byte length header, containing the byte length of the payload
// - The header is in big-endian byte order for network compatibility
// - The payload follows immediately after the header
// - No explicit footer or frame boundary marker is used

use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
use bytes::{BytesMut, BufMut, Buf};
use crate::protocol::MCPMessage;
use crate::error::{MCPError, TransportError};
use serde_json;
use tokio_util::codec::{Decoder, Encoder};

// Constants for framing
const HEADER_SIZE: usize = 4;      // 4-byte header for length
const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024; // 16 MB max frame size

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
    #[must_use]
    pub const fn new(payload: BytesMut) -> Self {
        Self { payload }
    }
    
    /// Create a new frame from a byte vector
    ///
    /// Converts the vector into a BytesMut and creates a frame.
    ///
    /// # Arguments
    ///
    /// * `vec` - The byte vector to use as the payload
    ///
    /// # Returns
    ///
    /// A new Frame instance containing the payload
    #[must_use]
    pub fn from_vec(vec: Vec<u8>) -> Self {
        let mut bytes = BytesMut::with_capacity(vec.len());
        bytes.extend_from_slice(&vec);
        Self { payload: bytes }
    }
    
    /// Get a reference to the frame payload
    ///
    /// # Returns
    ///
    /// A reference to the frame's payload
    #[must_use]
    pub const fn payload(&self) -> &BytesMut {
        &self.payload
    }
    
    /// Get the length of the frame payload in bytes
    ///
    /// # Returns
    ///
    /// The length of the frame payload
    #[must_use]
    pub fn len(&self) -> usize {
        self.payload.len()
    }
    
    /// Check if the frame payload is empty
    ///
    /// # Returns
    ///
    /// True if the frame payload is empty, false otherwise
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.payload.is_empty()
    }
}

/// Encoder for MCP frames
///
/// This encoder implements the tokio_util::codec::Encoder trait for
/// encoding Frame instances to be sent over a byte transport.
#[derive(Debug, Default)]
pub struct FrameEncoder;

impl FrameEncoder {
    /// Create a new frame encoder
    ///
    /// # Returns
    ///
    /// A new FrameEncoder instance
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Encoder<Frame> for FrameEncoder {
    type Error = crate::error::MCPError;

    fn encode(&mut self, frame: Frame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Calculate frame length and ensure it's within limits
        let frame_len = frame.len();
        if frame_len > MAX_FRAME_SIZE {
            let error_msg = format!(
                "Frame too large: {} bytes (max is {} bytes)",
                frame_len, MAX_FRAME_SIZE
            );
            return Err(MCPError::Transport(TransportError::FramingError(error_msg)));
        }
        
        // Write frame length header
        let header = u32::try_from(frame_len)
            .map_err(|e| {
                MCPError::Transport(TransportError::FramingError(format!(
                    "Frame size exceeds maximum representable u32 value: {} bytes",
                    frame_len
                )))
            })?
            .to_be_bytes();
        
        // Reserve space in the buffer for the header and payload
        dst.reserve(HEADER_SIZE + frame_len);
        
        // Write the header and payload
        dst.put_slice(&header);
        dst.put_slice(&frame.payload);
        
        Ok(())
    }
}

/// Decoder for MCP frames
///
/// This decoder implements the tokio_util::codec::Decoder trait for
/// decoding bytes into Frame instances.
#[derive(Debug, Default)]
pub struct FrameDecoder;

impl FrameDecoder {
    /// Create a new frame decoder
    ///
    /// # Returns
    ///
    /// A new FrameDecoder instance
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Decoder for FrameDecoder {
    type Item = Frame;
    type Error = crate::error::MCPError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Check if we have enough data for a header
        if src.len() < HEADER_SIZE {
            // Not enough data for a header, wait for more
            return Ok(None);
        }
        
        // Read frame length from header
        let mut header = [0u8; HEADER_SIZE];
        header.copy_from_slice(&src[..HEADER_SIZE]);
        let frame_len = u32::from_be_bytes(header) as usize;
        
        // Check if the frame length is valid
        if frame_len > MAX_FRAME_SIZE {
            return Err(MCPError::Transport(TransportError::FramingError(
                format!("Frame too large: {frame_len} bytes (max is {MAX_FRAME_SIZE} bytes)").into()
            )));
        }
        
        // Check if we have the complete frame data
        if src.len() < HEADER_SIZE + frame_len {
            // Not enough data yet, reserve space and wait for more
            src.reserve(HEADER_SIZE + frame_len - src.len());
            return Ok(None);
        }
        
        // We have a complete frame, extract it
        src.advance(HEADER_SIZE); // Skip past the header
        let payload = src.split_to(frame_len);
        
        // Create and return the frame
        Ok(Some(Frame::new(payload)))
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
    /// * `reader` - The underlying `AsyncRead` stream to read from
    ///
    /// # Returns
    ///
    /// A new `FrameReader` instance
    #[must_use]
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
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - There is an I/O error reading from the underlying stream
    /// - The frame format is invalid
    /// - The end of stream is reached with a partial frame
    pub async fn read_frame(&mut self) -> crate::error::Result<Option<Frame>> {
        // If the buffer is empty, try to read some data
        if self.buffer.is_empty() {
            if self.read_to_buffer().await? == 0 {
                // EOF, no more frames
                return Ok(None);
            }
        }
        
        // Check if we have at least a complete header
        if self.buffer.len() < HEADER_SIZE {
            // Not enough data for header, try to read more
            if self.read_to_buffer().await? == 0 {
                // EOF, incomplete frame
                if !self.buffer.is_empty() {
                    return Err(MCPError::Transport(
                        format!("Incomplete frame at end of stream: received {} bytes", self.buffer.len()).into()
                    ));
                }
                return Ok(None);
            }
            
            // Still not enough for header?
            if self.buffer.len() < HEADER_SIZE {
                return Err(MCPError::Transport(
                    format!("Incomplete frame header: only {} bytes available", self.buffer.len()).into()
                ));
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
            return Err(MCPError::Transport(TransportError::FramingError(
                format!("Frame too large: {frame_len} bytes (max is {MAX_FRAME_SIZE} bytes)").into()
            )));
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
                MCPError::Transport(TransportError::ReadError(e.to_string()))
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
    /// * `writer` - The underlying `AsyncWrite` stream to write to
    ///
    /// # Returns
    ///
    /// A new `FrameWriter` instance
    #[must_use]
    pub const fn new(writer: W) -> Self {
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
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The frame is too large (exceeds `MAX_FRAME_SIZE`)
    /// - There is an I/O error while writing the header or payload
    /// - The writer cannot be flushed
    pub async fn write_frame(&mut self, frame: Frame) -> crate::error::Result<()> {
        // Check frame size
        if frame.len() > MAX_FRAME_SIZE {
            let error_msg = format!(
                "Frame too large: {} bytes (max is {} bytes)",
                frame.len(), MAX_FRAME_SIZE
            );
            return Err(MCPError::Transport(TransportError::FramingError(error_msg)));
        }
        
        // Write frame length header
        let header = u32::try_from(frame.len())
            .map_err(|e| {
                MCPError::Transport(TransportError::FramingError(format!(
                    "Frame size exceeds maximum representable u32 value: {} bytes",
                    frame.len()
                )))
            })?
            .to_be_bytes();
            
        self.writer.write_all(&header).await
            .map_err(|e| {
                MCPError::Transport(TransportError::WriteError(e.to_string()))
            })?;
        
        // Write frame payload
        self.writer.write_all(&frame.payload).await
            .map_err(|e| {
                MCPError::Transport(TransportError::WriteError(e.to_string()))
            })?;
        
        // Flush the writer
        self.writer.flush().await
            .map_err(|e| {
                MCPError::Transport(TransportError::WriteError(e.to_string()))
            })?;
        
        Ok(())
    }
}

/// Codec for encoding and decoding MCP messages
///
/// Provides functionality for converting between `MCPMessages` and frames.
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
    /// A new `MessageCodec` instance for encoding and decoding messages
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
    
    /// Encode a message to a frame
    ///
    /// Serializes an `MCPMessage` to JSON and creates a frame containing
    /// the serialized data.
    ///
    /// # Arguments
    ///
    /// * `message` - The MCP message to encode
    ///
    /// # Returns
    ///
    /// Result containing the encoded frame or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The message cannot be serialized to JSON
    pub async fn encode_message(&self, message: &MCPMessage) -> crate::error::Result<Frame> {
        // Serialize the message to JSON
        let json = serde_json::to_vec(message)
            .map_err(|e| {
                MCPError::Transport(TransportError::FramingError(e.to_string()))
            })?;
        
        // Create a frame
        let mut buffer = BytesMut::with_capacity(json.len());
        buffer.put_slice(&json);
        
        Ok(Frame::new(buffer))
    }
    
    /// Decode a message from a frame
    ///
    /// Deserializes an `MCPMessage` from the JSON data in a frame.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame containing the serialized message
    ///
    /// # Returns
    ///
    /// Result containing the decoded message or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The frame data cannot be deserialized into an `MCPMessage`
    /// - The JSON data in the frame is invalid or malformed
    pub async fn decode_message(&self, frame: &Frame) -> crate::error::Result<MCPMessage> {
        // Deserialize the message from JSON
        let message = serde_json::from_slice(&frame.payload)
            .map_err(|e| {
                MCPError::Transport(TransportError::FramingError(e.to_string()))
            })?;
        
        Ok(message)
    }
}

impl Decoder for MessageCodec {
    type Item = MCPMessage;
    type Error = MCPError;

    fn decode(&mut self, src: &mut BytesMut) -> std::result::Result<Option<Self::Item>, Self::Error> {
        // Check if we have enough data to read the length
        if src.len() < HEADER_SIZE {
            return Ok(None);
        }

        // Read the message length (u32)
        let mut length_bytes = [0u8; HEADER_SIZE];
        length_bytes.copy_from_slice(&src[..HEADER_SIZE]);
        let length = u32::from_be_bytes(length_bytes) as usize;

        // Check for potentially malicious large frame size early
        if length > MAX_FRAME_SIZE {
             return Err(MCPError::Transport(
                format!("Frame size too large: {} > {}", length, MAX_FRAME_SIZE).into()
            ));
        }

        // Check if we have the complete message frame (header + payload)
        if src.len() < HEADER_SIZE + length {
            // Not enough data yet, reserve space and wait for more
            src.reserve(HEADER_SIZE + length - src.len());
            return Ok(None);
        }

        // Consume the length prefix
        src.advance(HEADER_SIZE);

        // Read the message data payload
        let data = src.split_to(length);

        // Deserialize the message from the payload (assuming JSON)
        serde_json::from_slice::<MCPMessage>(&data)
            .map(Some)
            .map_err(|e| MCPError::Transport(TransportError::FramingError(e.to_string())))
    }

    fn decode_eof(&mut self, buf: &mut BytesMut) -> std::result::Result<Option<Self::Item>, Self::Error> {
        match self.decode(buf)? {
            Some(frame) => Ok(Some(frame)),
            None => {
                if buf.is_empty() {
                    Ok(None)
                } else {
                    // Data remains but not enough for a full frame, indicates an error
                    Err(MCPError::Transport(TransportError::FramingError("Bytes remaining on stream at EOF".into())))
                }
            }
        }
    }
}

impl Encoder<MCPMessage> for MessageCodec {
    type Error = MCPError;

    fn encode(&mut self, item: MCPMessage, dst: &mut BytesMut) -> std::result::Result<(), Self::Error> {
        // Serialize the message (assuming JSON for now)
        let encoded = serde_json::to_vec(&item)
            .map_err(|e| MCPError::Transport(TransportError::FramingError(e.to_string())))?;

        let len = encoded.len();

        // Check if frame size exceeds limit before encoding
        if len > MAX_FRAME_SIZE {
            return Err(MCPError::Transport(TransportError::FramingError(
                format!("Message too large to encode: {} > {}", len, MAX_FRAME_SIZE).into()
            )));
        }

        // Write the length prefix (u32)
        dst.reserve(HEADER_SIZE + len);
        dst.put_u32(len as u32); // Length is of the payload

        // Write the message data payload
        dst.put_slice(&encoded);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use crate::protocol::types::MessageType;
    
    
    

    #[tokio::test]
    async fn test_frame_round_trip() {
        // Create an MCPMessage with proper version format
        let message = MCPMessage {
            id: crate::protocol::types::MessageId::new(),
            timestamp: chrono::Utc::now(),
            version: crate::protocol::types::ProtocolVersion::default(),
            type_: MessageType::Command,
            payload: serde_json::json!({ "test": "value" }),
            metadata: None,
            security: crate::security::types::SecurityMetadata::default(),
            trace_id: None,
        };
        
        // Create a serializable message for the wire
        let wire_payload = serde_json::json!({
            "id": message.id.0,
            "timestamp": message.timestamp.timestamp_millis(),
            "version": "1.0",
            "type_": "Command",
            "payload": { "test": "value" },
            "metadata": null,
            "security": message.security,
            "trace_id": null
        });
        
        // Serialize the message to bytes
        let bytes = serde_json::to_vec(&wire_payload).unwrap();
        let mut buffer = BytesMut::with_capacity(bytes.len());
        buffer.put_slice(&bytes);
        
        // Create a frame
        let frame = Frame::new(buffer);
        
        // Create codec and decode
        let codec = MessageCodec::new();
        let decoded = codec.decode_message(&frame).await.unwrap();
        
        // Verify
        assert_eq!(decoded.type_, message.type_);
        assert_eq!(decoded.payload, message.payload);
    }
    
    #[tokio::test]
    async fn test_frame_reader_writer() {
        // Create an MCPMessage with proper version format
        let message = MCPMessage {
            id: crate::protocol::types::MessageId::new(),
            timestamp: chrono::Utc::now(),
            version: crate::protocol::types::ProtocolVersion::default(),
            type_: MessageType::Command,
            payload: serde_json::json!({ "test": "value" }),
            metadata: None,
            security: crate::security::types::SecurityMetadata::default(),
            trace_id: None,
        };
        
        // Create a serializable message for the wire
        let wire_payload = serde_json::json!({
            "id": message.id.0,
            "timestamp": message.timestamp.timestamp_millis(),
            "version": "1.0",
            "type_": "Command",
            "payload": { "test": "value" },
            "metadata": null,
            "security": message.security,
            "trace_id": null
        });
        
        // Serialize the message to bytes
        let bytes = serde_json::to_vec(&wire_payload).unwrap();
        let mut buffer_frame = BytesMut::with_capacity(bytes.len());
        buffer_frame.put_slice(&bytes);
        
        // Create a frame
        let frame = Frame::new(buffer_frame);
        
        // Create a buffer for the frame writer/reader
        let mut buffer = Vec::new();
        
        // Write frame
        {
            let mut writer = FrameWriter::new(&mut buffer);
            writer.write_frame(frame).await.unwrap();
        }
        
        // Read frame
        let mut reader = FrameReader::new(&buffer[..]);
        let read_frame = reader.read_frame().await.unwrap().unwrap();
        
        // Create codec and decode
        let codec = MessageCodec::new();
        let decoded = codec.decode_message(&read_frame).await.unwrap();
        
        // Verify
        assert_eq!(decoded.type_, message.type_);
        assert_eq!(decoded.payload, message.payload);
    }
    
    #[test]
    fn test_frame_encoder_decoder() {
        // Create test data
        let message_bytes = b"Hello, MCP!";
        let mut buf = BytesMut::with_capacity(1024);
        buf.extend_from_slice(message_bytes);
        
        // Create a frame
        let frame = Frame::new(buf.clone());
        
        // Use the encoder to encode the frame
        let mut encoder = FrameEncoder::new();
        let mut encoded_buf = BytesMut::new();
        encoder.encode(frame.clone(), &mut encoded_buf).unwrap();
        
        // Verify the encoded buffer has the correct format
        assert_eq!(encoded_buf.len(), HEADER_SIZE + message_bytes.len());
        
        // First 4 bytes should be the length of the payload in big-endian format
        let expected_len = u32::try_from(message_bytes.len()).unwrap().to_be_bytes();
        assert_eq!(&encoded_buf[0..4], &expected_len);
        
        // Following bytes should be the payload
        assert_eq!(&encoded_buf[4..], message_bytes);
        
        // Now decode the frame
        let mut decoder = FrameDecoder::new();
        let decoded_frame = decoder.decode(&mut encoded_buf).unwrap().unwrap();
        
        // Make sure the decoded frame has the same payload as the original
        assert_eq!(decoded_frame.payload(), frame.payload());
    }
} 