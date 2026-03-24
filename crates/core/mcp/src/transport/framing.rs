// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Message framing (codec layer) for transport.
//!
//! This module provides the internal framing implementation for MCP messages:
//! - [`FrameReader`] / [`FrameWriter`]: length-prefixed frame I/O over async streams
//! - `FramingMessageCodec`: tokio_util `Decoder`/`Encoder` for `MCPMessage` wire format
//!
//! **Relationship to `crate::transport::frame`**: This is the codec/wire layer. It produces
//! and consumes the public [`Frame`] type (with `Bytes` payload).
//! The public `Frame` is the canonical API; this module handles buffering and parsing.

use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio_util::codec::{Decoder, Encoder};

use crate::error::transport::TransportError;
use crate::protocol::types::MCPMessage;
use crate::transport::frame::Frame;

use universal_constants::limits;

// Constants
const HEADER_SIZE: usize = 4; // 4 bytes for u32 message length

/// Maximum frame size (reuses [`limits::MAX_TRANSPORT_FRAME_SIZE`] for consistency).
const MAX_FRAME_SIZE: usize = limits::MAX_TRANSPORT_FRAME_SIZE;

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

    /// Read a frame from the stream.
    ///
    /// Returns the public [`Frame`] type.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Frame))` - A complete frame was read
    /// * `Ok(None)` - End of stream was reached (clean shutdown)
    /// * `Err` - An error occurred while reading
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
                        format!("Incomplete frame at EOF: {} bytes left", self.buffer.len()),
                    )
                    .into();
                    return Err(transport_err);
                }

                // Clean EOF
                return Ok(None);
            }
        }
    }

    /// Attempt to parse a complete frame from the buffer.
    fn parse_frame(&mut self) -> crate::error::Result<Option<Frame>> {
        if self.buffer.len() < HEADER_SIZE {
            return Ok(None);
        }

        let mut length_bytes = [0u8; HEADER_SIZE];
        length_bytes.copy_from_slice(&self.buffer[..HEADER_SIZE]);
        let length = u32::from_be_bytes(length_bytes) as usize;

        if length > MAX_FRAME_SIZE {
            let error_msg =
                format!("Frame too large: {length} bytes (max is {MAX_FRAME_SIZE} bytes)");
            let transport_err: crate::error::MCPError =
                TransportError::InvalidFrame(error_msg).into();
            return Err(transport_err);
        }

        if self.buffer.len() < HEADER_SIZE + length {
            return Ok(None);
        }

        self.buffer.advance(HEADER_SIZE);
        let payload = self.buffer.split_to(length);
        Ok(Some(Frame {
            payload: payload.freeze(),
        }))
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
        let bytes_read = self.reader.read_buf(&mut self.buffer).await.map_err(|e| {
            let transport_err: crate::error::MCPError =
                TransportError::IoError(e.to_string()).into();
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

    /// Write a frame to the stream.
    pub async fn write_frame(&mut self, frame: &Frame) -> crate::error::Result<()> {
        let len = frame.payload.len();
        if len > MAX_FRAME_SIZE {
            let error_msg = format!("Frame too large: {len} bytes (max is {MAX_FRAME_SIZE} bytes)");
            let transport_err: crate::error::MCPError =
                TransportError::InvalidFrame(error_msg).into();
            return Err(transport_err);
        }

        let header = u32::try_from(len)
            .map_err(|_| {
                let transport_err: crate::error::MCPError = TransportError::InvalidFrame(format!(
                    "Frame size exceeds u32 max: {len} bytes"
                ))
                .into();
                transport_err
            })?
            .to_be_bytes();

        self.writer.write_all(&header).await.map_err(|e| {
            let transport_err: crate::error::MCPError =
                TransportError::IoError(e.to_string()).into();
            transport_err
        })?;

        self.writer.write_all(&frame.payload).await.map_err(|e| {
            let transport_err: crate::error::MCPError =
                TransportError::IoError(e.to_string()).into();
            transport_err
        })?;

        self.writer.flush().await.map_err(|e| {
            let transport_err: crate::error::MCPError =
                TransportError::IoError(e.to_string()).into();
            transport_err
        })?;

        Ok(())
    }
}

/// Codec for encoding and decoding MCP messages to/from frames.
///
/// Produces the public [`Frame`] type.
#[derive(Debug, Default)]
pub struct FramingMessageCodec;

impl FramingMessageCodec {
    /// Create a new framing message codec.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Encode a message to a frame.
    pub fn encode_message(&self, message: &MCPMessage) -> crate::error::Result<Frame> {
        let json = serde_json::to_vec(message).map_err(|e| {
            let transport_err: crate::error::MCPError =
                TransportError::SerializationError(e.to_string()).into();
            transport_err
        })?;
        Ok(Frame {
            payload: bytes::Bytes::from(json),
        })
    }

    /// Decode a message from a frame.
    pub fn decode_message(&self, frame: &Frame) -> crate::error::Result<MCPMessage> {
        serde_json::from_slice(&frame.payload).map_err(|e| {
            let transport_err: crate::error::MCPError =
                TransportError::SerializationError(e.to_string()).into();
            transport_err
        })
    }
}

impl Decoder for FramingMessageCodec {
    type Item = MCPMessage;
    type Error = TransportError;

    fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> std::result::Result<Option<Self::Item>, Self::Error> {
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
            return Err(TransportError::InvalidFrame(format!(
                "Frame size too large: {length} > {MAX_FRAME_SIZE}"
            )));
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

    fn decode_eof(
        &mut self,
        buf: &mut BytesMut,
    ) -> std::result::Result<Option<Self::Item>, Self::Error> {
        match self.decode(buf)? {
            Some(frame) => Ok(Some(frame)),
            None => {
                if buf.is_empty() {
                    Ok(None)
                } else {
                    // Data remains but not enough for a full frame, indicates an error
                    Err(TransportError::InvalidFrame(
                        "Bytes remaining on stream at EOF".into(),
                    ))
                }
            }
        }
    }
}

impl Encoder<MCPMessage> for FramingMessageCodec {
    type Error = TransportError;

    fn encode(
        &mut self,
        item: MCPMessage,
        dst: &mut BytesMut,
    ) -> std::result::Result<(), Self::Error> {
        // Serialize the message to JSON
        let json = serde_json::to_vec(&item)
            .map_err(|e| TransportError::SerializationError(e.to_string()))?;

        // Get the length of the serialized data
        let length = json.len();

        // Check the size limit
        if length > MAX_FRAME_SIZE {
            return Err(TransportError::InvalidFrame(format!(
                "Message too large: {length} > {MAX_FRAME_SIZE}"
            )));
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
    use crate::protocol::types::MessageType;
    use universal_constants::limits;

    #[tokio::test]
    async fn test_frame_round_trip() {
        let message = MCPMessage::new(
            MessageType::Command,
            serde_json::json!({
                "field1": "value1",
                "field2": 42
            }),
        );

        let codec = FramingMessageCodec::new();
        let frame = codec.encode_message(&message).expect("should succeed");
        let decoded = codec.decode_message(&frame).expect("should succeed");

        assert_eq!(decoded.type_, message.type_);
        assert_eq!(decoded.id, message.id);
        assert_eq!(decoded.payload, message.payload);
    }

    #[tokio::test]
    async fn test_frame_reader_writer() {
        let frame = Frame {
            payload: bytes::Bytes::from_static(b"Hello, world!"),
        };

        let (a, b) = tokio::io::duplex(1024);
        let mut writer = FrameWriter::new(a);
        let mut reader = FrameReader::new(b);

        writer.write_frame(&frame).await.expect("should succeed");
        let read_frame = reader
            .read_frame()
            .await
            .expect("should succeed")
            .expect("should succeed");

        assert_eq!(read_frame.payload.as_ref(), frame.payload.as_ref());
    }

    #[tokio::test]
    async fn frame_reader_eof_with_leftover_bytes_errors() {
        let data: Vec<u8> = vec![0, 0, 0, 5, 1, 2]; // length 5 but only 2 payload bytes
        let mut reader = FrameReader::new(std::io::Cursor::new(data));
        let err = reader.read_frame().await.expect_err("incomplete");
        assert!(format!("{err:?}").contains("Invalid") || err.to_string().contains("frame"));
    }

    #[tokio::test]
    async fn frame_reader_rejects_oversized_length_prefix() {
        use universal_constants::limits;
        let len: u32 = (limits::MAX_TRANSPORT_FRAME_SIZE as u64 + 1)
            .try_into()
            .expect("fits u32 for test");
        let mut buf = Vec::new();
        buf.extend_from_slice(&len.to_be_bytes());
        let mut reader = FrameReader::new(std::io::Cursor::new(buf));
        let err = reader.read_frame().await.expect_err("too large");
        assert!(err.to_string().contains("large") || err.to_string().contains("Frame"));
    }

    #[tokio::test]
    async fn frame_writer_rejects_oversized_payload() {
        use universal_constants::limits;
        let mut w = FrameWriter::new(tokio::io::sink());
        let frame = Frame {
            payload: bytes::Bytes::from(vec![0u8; limits::MAX_TRANSPORT_FRAME_SIZE + 1]),
        };
        let err = w.write_frame(&frame).await.expect_err("size");
        assert!(err.to_string().contains("large") || err.to_string().contains("Frame"));
    }

    #[test]
    fn framing_message_codec_decode_needs_full_header() {
        let mut codec = FramingMessageCodec::new();
        let mut buf = bytes::BytesMut::from(&[1u8, 2u8][..]);
        assert!(codec.decode(&mut buf).expect("should succeed").is_none());
    }

    #[test]
    fn framing_message_codec_decode_invalid_json_errors() {
        let mut codec = FramingMessageCodec::new();
        let mut buf = bytes::BytesMut::new();
        let bad_payload = br#"{"not":"mcp""#.to_vec();
        buf.put_u32(bad_payload.len() as u32);
        buf.put_slice(&bad_payload);
        let err = codec.decode(&mut buf).expect_err("bad json");
        assert!(matches!(err, TransportError::SerializationError(_)));
    }

    #[test]
    fn framing_message_codec_decode_oversized_length() {
        let mut codec = FramingMessageCodec::new();
        let mut buf = bytes::BytesMut::new();
        buf.put_u32((limits::MAX_TRANSPORT_FRAME_SIZE as u32).saturating_add(1));
        let err = codec.decode(&mut buf).expect_err("large");
        assert!(matches!(err, TransportError::InvalidFrame(_)));
    }

    #[test]
    fn framing_message_codec_decode_eof_leaves_bytes() {
        let mut codec = FramingMessageCodec::new();
        let mut buf = bytes::BytesMut::from(&[1u8, 2u8, 3u8][..]);
        let err = codec.decode_eof(&mut buf).expect_err("leftover");
        assert!(matches!(err, TransportError::InvalidFrame(_)));
    }

    #[test]
    fn framing_message_codec_encode_message_too_large() {
        let mut codec = FramingMessageCodec::new();
        let big = serde_json::Value::String("x".repeat(limits::MAX_TRANSPORT_FRAME_SIZE + 1));
        let msg = MCPMessage::new(MessageType::Command, big);
        let mut dst = bytes::BytesMut::new();
        let err = Encoder::encode(&mut codec, msg, &mut dst).expect_err("too big");
        assert!(matches!(err, TransportError::InvalidFrame(_)));
    }
}
