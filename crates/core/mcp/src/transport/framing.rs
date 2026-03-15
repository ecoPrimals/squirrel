// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Message framing for transport layer
//! 
//! This module provides the framing implementation for MCP messages
//! to be sent over various transport channels.

use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio_util::codec::{Decoder, Encoder};
use crate::error::transport::TransportError;
use crate::protocol::types::MCPMessage;

// Constants
const HEADER_SIZE: usize = 4; // 4 bytes for u32 message length

/// Maximum frame size - configurable via environment variable
///
/// Default: 16MB
/// Environment variable: MCP_MAX_FRAME_SIZE
const MAX_FRAME_SIZE: usize = {
    match std::env::var("MCP_MAX_FRAME_SIZE") {
        Ok(val) => match val.parse::<usize>() {
            Ok(size) => size,
            Err(_) => 16 * 1024 * 1024, // 16MB fallback
        },
        Err(_) => 16 * 1024 * 1024, // 16MB default
    }
};

/// A frame of data in the MCP protocol
#[derive(Debug, Clone)]
pub struct Frame {
    /// Frame payload containing the message data
    pub payload: BytesMut,
}

impl Frame {
    /// Create a new frame with the given payload
    ///
    /// # Arguments
    ///
    /// * `payload` - The payload to include in the frame
    ///
    /// # Returns
    ///
    /// A new frame containing the payload
    #[must_use]
    pub const fn new(payload: BytesMut) -> Self {
        Self { payload }
    }

    /// Get a reference to the frame payload
    ///
    /// # Returns
    ///
    /// A reference to the payload bytes
    #[must_use]
    pub const fn payload(&self) -> &BytesMut {
        &self.payload
    }

    /// Get the length of the frame payload
    ///
    /// # Returns
    ///
    /// The length of the payload in bytes
    #[must_use]
    pub fn len(&self) -> usize {
        self.payload.len()
    }

    /// Check if the frame payload is empty
    ///
    /// # Returns
    ///
    /// True if the payload has zero length
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.payload.is_empty()
    }
}

/// Reader for MCP frames from a byte stream
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
    /// * `reader` - The async reader to read frames from
    ///
    /// # Returns
    ///
    /// A new frame reader for the given reader
    #[must_use]
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: BytesMut::with_capacity(8 * 1024), // Initial buffer size, can grow
        }
    }

    /// Read a frame from the stream
    ///
    /// Reads and returns the next complete frame from the stream. This method
    /// may block until a complete frame is available or an error occurs.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Frame))` - A complete frame was read
    /// * `Ok(None)` - End of stream was reached (clean shutdown)
    /// * `Err` - An error occurred while reading
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - There is an I/O error while reading
    /// - A frame is too large (exceeds `MAX_FRAME_SIZE`)
    /// - A frame header is invalid
    pub async fn read_frame(&mut self) -> crate::error::Result<Option<Frame>> {
        loop {
            // First, check if we have a complete frame in the buffer
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            // If we don't have a complete frame, read more data
            let bytes_read = self.read_to_buffer().await?;
            
            // If we read 0 bytes, we're at EOF
            if bytes_read == 0 {
                // If we have any incomplete data in the buffer, it's an error
                if !self.buffer.is_empty() {
                    let transport_err: crate::error::MCPError = TransportError::InvalidFrame(
                        format!("Incomplete frame at EOF: {} bytes left", self.buffer.len())
                    ).into();
                    return Err(transport_err);
                }
                
                // Clean EOF
                return Ok(None);
            }
        }
    }

    /// Attempt to parse a complete frame from the buffer
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Frame))` if a complete frame was parsed
    /// * `Ok(None)` if the buffer doesn't contain a complete frame yet
    /// * `Err` if the frame header is invalid
    fn parse_frame(&mut self) -> crate::error::Result<Option<Frame>> {
        // Need at least the header to parse frame length
        if self.buffer.len() < HEADER_SIZE {
            return Ok(None);
        }

        // Read the frame length from the header
        let mut length_bytes = [0u8; HEADER_SIZE];
        length_bytes.copy_from_slice(&self.buffer[..HEADER_SIZE]);
        let length = u32::from_be_bytes(length_bytes) as usize;

        // Validate frame size
        if length > MAX_FRAME_SIZE {
            let error_msg = format!(
                "Frame too large: {} bytes (max is {} bytes)",
                length, MAX_FRAME_SIZE
            );
            let transport_err: crate::error::MCPError = TransportError::InvalidFrame(error_msg).into();
            return Err(transport_err);
        }

        // Check if we have a complete frame
        if self.buffer.len() < HEADER_SIZE + length {
            return Ok(None);
        }

        // We have a complete frame
        self.buffer.advance(HEADER_SIZE); // Skip past the header
        let payload = self.buffer.split_to(length); // Extract the payload

        Ok(Some(Frame::new(payload)))
    }

    /// Read more data into the buffer
    ///
    /// # Returns
    ///
    /// The number of bytes read, or an error if reading failed
    async fn read_to_buffer(&mut self) -> crate::error::Result<usize> {
        // Grow the buffer if needed
        if self.buffer.remaining_mut() < 1024 {
            self.buffer.reserve(8 * 1024);
        }

        // Read into the buffer
        let bytes_read = self.reader.read_buf(&mut self.buffer).await
            .map_err(|e| {
                let transport_err: crate::error::MCPError = TransportError::IoError(e.to_string()).into();
                transport_err
            })?;

        Ok(bytes_read)
    }
}

/// Writer for MCP frames to a byte stream
pub struct FrameWriter<W> {
    /// The underlying writer for the byte stream
    writer: W,
}

impl<W: AsyncWrite + Unpin> FrameWriter<W> {
    /// Create a new frame writer
    ///
    /// # Arguments
    ///
    /// * `writer` - The async writer to write frames to
    ///
    /// # Returns
    ///
    /// A new frame writer for the given writer
    #[must_use]
    pub const fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Write a frame to the stream
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
            let transport_err: crate::error::MCPError = TransportError::InvalidFrame(error_msg).into();
            return Err(transport_err);
        }
        
        // Write frame length header
        let header = u32::try_from(frame.len())
            .map_err(|_| {
                let error_msg = format!(
                    "Frame size exceeds maximum representable u32 value: {} bytes",
                    frame.len()
                );
                let transport_err: crate::error::MCPError = TransportError::InvalidFrame(error_msg).into();
                transport_err
            })?
            .to_be_bytes();
            
        self.writer.write_all(&header).await
            .map_err(|e| {
                let transport_err: crate::error::MCPError = TransportError::IoError(e.to_string()).into();
                transport_err
            })?;
        
        // Write frame payload
        self.writer.write_all(&frame.payload).await
            .map_err(|e| {
                let transport_err: crate::error::MCPError = TransportError::IoError(e.to_string()).into();
                transport_err
            })?;
        
        // Flush the writer
        self.writer.flush().await
            .map_err(|e| {
                let transport_err: crate::error::MCPError = TransportError::IoError(e.to_string()).into();
                transport_err
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
                let transport_err: crate::error::MCPError = TransportError::SerializationError(e.to_string()).into();
                transport_err
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
                let transport_err: crate::error::MCPError = TransportError::SerializationError(e.to_string()).into();
                transport_err
            })?;
        
        Ok(message)
    }
}

impl Decoder for MessageCodec {
    type Item = MCPMessage;
    type Error = TransportError;

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
             return Err(TransportError::InvalidFrame(
                format!("Frame size too large: {} > {}", length, MAX_FRAME_SIZE)
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
            .map_err(|e| TransportError::SerializationError(e.to_string()))
    }

    fn decode_eof(&mut self, buf: &mut BytesMut) -> std::result::Result<Option<Self::Item>, Self::Error> {
        match self.decode(buf)? {
            Some(frame) => Ok(Some(frame)),
            None => {
                if buf.is_empty() {
                    Ok(None)
                } else {
                    // Data remains but not enough for a full frame, indicates an error
                    Err(TransportError::InvalidFrame("Bytes remaining on stream at EOF".into()))
                }
            }
        }
    }
}

impl Encoder<MCPMessage> for MessageCodec {
    type Error = TransportError;

    fn encode(&mut self, item: MCPMessage, dst: &mut BytesMut) -> std::result::Result<(), Self::Error> {
        // Serialize the message to JSON
        let json = serde_json::to_vec(&item)
            .map_err(|e| TransportError::SerializationError(e.to_string()))?;
        
        // Get the length of the serialized data
        let length = json.len();
        
        // Check the size limit
        if length > MAX_FRAME_SIZE {
            return Err(TransportError::InvalidFrame(
                format!("Message too large: {} > {}", length, MAX_FRAME_SIZE)
            ));
        }
        
        // Reserve space for the length header and message data
        dst.reserve(HEADER_SIZE + length);
        
        // Write the length header
        dst.put_u32(length as u32);
        
        // Write the serialized message
        dst.put_slice(&json);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::{MessageType, MessageId};
    
    #[tokio::test]
    async fn test_frame_round_trip() {
        // Create a test message
        let message = MCPMessage::new(
            MessageType::Request,
            "test-message",
            serde_json::json!({
                "field1": "value1",
                "field2": 42
            }),
        );
        
        // Create a codec
        let codec = MessageCodec::new();
        
        // Encode the message to a frame
        let frame = codec.encode_message(&message).await.unwrap();
        
        // Decode the frame back to a message
        let decoded = codec.decode_message(&frame).await.unwrap();
        
        // Verify that the decoded message is the same as the original
        assert_eq!(decoded.message_type(), message.message_type());
        assert_eq!(decoded.id().0, message.id().0);
        assert_eq!(decoded.payload(), message.payload());
    }
    
    #[tokio::test]
    async fn test_frame_reader_writer() {
        // Create a test frame
        let mut payload = BytesMut::with_capacity(16);
        payload.put_slice(b"Hello, world!");
        let frame = Frame::new(payload);
        
        // Create a mock writer/reader pair
        let (a, b) = tokio::io::duplex(1024);
        let mut writer = FrameWriter::new(a);
        let mut reader = FrameReader::new(b);
        
        // Write the frame
        writer.write_frame(frame.clone()).await.unwrap();
        
        // Read the frame
        let read_frame = reader.read_frame().await.unwrap().unwrap();
        
        // Verify the frame data
        assert_eq!(read_frame.payload.as_ref(), frame.payload.as_ref());
    }
} 