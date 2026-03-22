// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Message Frame Transport (public API)
//!
//! This module provides the canonical [`Frame`] type and frame-based message transport for MCP.
//!
//! **Layering**: [`Frame`] uses `Bytes` for zero-copy cloning. The codec/framing layer
//! ([`crate::transport::framing`]) produces and consumes this type; it handles buffering
//! and length-prefixed wire format.
//!
//! Re-exports from framing: [`FrameReader`], [`FrameWriter`] for stream I/O.

use std::io::{Read, Write};
use std::marker::PhantomData;
use std::sync::Arc;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::error::{MCPError, Result};
use crate::protocol::types::MCPMessage;

use universal_constants::limits;

// Re-export framing I/O for consumers that need FrameReader/FrameWriter
pub use crate::transport::framing::{FrameReader, FrameWriter};

/// Maximum frame size (16MB)
///
/// Re-exports [`universal_constants::limits::MAX_TRANSPORT_FRAME_SIZE`] for backward compatibility.
pub const MAX_FRAME_SIZE: usize = limits::MAX_TRANSPORT_FRAME_SIZE;

/// Frame header size (4 bytes for length)
pub const FRAME_HEADER_SIZE: usize = 4;

/// Canonical message frame type for MCP transport.
///
/// Uses `Bytes` for payload — `Bytes::clone()` is O(1) (ref-count increment).
/// The codec layer ([`crate::transport::framing`]) produces/consumes this type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    /// Raw frame payload bytes (Bytes for zero-copy clone in encode)
    #[serde(
        serialize_with = "serialize_payload",
        deserialize_with = "deserialize_payload"
    )]
    pub payload: Bytes,
}

fn serialize_payload<S>(b: &Bytes, s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serde_bytes::serialize(b.as_ref(), s)
}

fn deserialize_payload<'de, D>(d: D) -> std::result::Result<Bytes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: Vec<u8> = serde_bytes::deserialize(d)?;
    Ok(Bytes::from(v))
}

impl Frame {
    /// Create a new frame with the given payload
    #[must_use]
    pub fn new(payload: Vec<u8>) -> Self {
        Self {
            payload: Bytes::from(payload),
        }
    }

    /// Create a frame from a byte vector
    #[must_use]
    pub fn from_vec(data: Vec<u8>) -> Self {
        Self {
            payload: Bytes::from(data),
        }
    }

    /// Create a frame from `Bytes` (zero-copy: `Bytes::clone` is O(1))
    pub fn from_bytes(bytes: &Bytes) -> Self {
        Self {
            payload: bytes.clone(),
        }
    }

    /// Get the frame size including header
    #[inline]
    #[must_use]
    pub const fn size(&self) -> usize {
        FRAME_HEADER_SIZE + self.payload.len()
    }

    /// Write frame to a writer
    #[must_use = "I/O errors should be handled"]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "Frame protocol uses u32 length prefix"
    )]
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Write length header (big endian)
        let length = self.payload.len() as u32;
        writer.write_all(&length.to_be_bytes()).map_err(|e| {
            MCPError::Transport(format!("Failed to write frame header: {e}").into())
        })?;

        // Write payload
        writer.write_all(&self.payload).map_err(|e| {
            MCPError::Transport(format!("Failed to write frame payload: {e}").into())
        })?;

        Ok(())
    }

    /// Read frame from a reader
    #[must_use = "I/O errors should be handled"]
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        // Read length header
        let mut length_bytes = [0u8; 4];
        reader
            .read_exact(&mut length_bytes)
            .map_err(|e| MCPError::Transport(format!("Failed to read frame header: {e}").into()))?;

        let length = u32::from_be_bytes(length_bytes) as usize;

        // Validate frame size
        if length > MAX_FRAME_SIZE {
            return Err(MCPError::Transport(
                format!("Frame size {length} exceeds maximum {MAX_FRAME_SIZE}").into(),
            ));
        }

        // Read payload
        let mut payload = vec![0u8; length];
        reader.read_exact(&mut payload).map_err(|e| {
            MCPError::Transport(format!("Failed to read frame payload: {e}").into())
        })?;

        Ok(Self {
            payload: Bytes::from(payload),
        })
    }
}

/// Async frame reader for reading length-prefixed frames from a stream
pub struct AsyncFrameReader<R> {
    reader: R,
    _phantom: PhantomData<R>,
}

impl<R: AsyncRead + Unpin> AsyncFrameReader<R> {
    /// Creates a new async frame reader wrapping the given reader
    pub const fn new(reader: R) -> Self {
        Self {
            reader,
            _phantom: PhantomData,
        }
    }

    /// Reads a single frame from the underlying stream
    pub async fn read_frame(&mut self) -> Result<Frame> {
        use tokio::io::AsyncReadExt;

        // Read length header
        let mut length_bytes = [0u8; 4];
        self.reader
            .read_exact(&mut length_bytes)
            .await
            .map_err(|e| MCPError::Transport(format!("Failed to read frame header: {e}").into()))?;

        let length = u32::from_be_bytes(length_bytes) as usize;

        // Validate frame size
        if length > MAX_FRAME_SIZE {
            return Err(MCPError::Transport(
                format!("Frame size {length} exceeds maximum {MAX_FRAME_SIZE}").into(),
            ));
        }

        // Read payload
        let mut payload = vec![0u8; length];
        self.reader.read_exact(&mut payload).await.map_err(|e| {
            MCPError::Transport(format!("Failed to read frame payload: {e}").into())
        })?;

        Ok(Frame {
            payload: Bytes::from(payload),
        })
    }
}

/// Async frame writer for writing length-prefixed frames to a stream
pub struct AsyncFrameWriter<W> {
    writer: W,
    _phantom: PhantomData<W>,
}

impl<W: AsyncWrite + Unpin> AsyncFrameWriter<W> {
    /// Creates a new async frame writer wrapping the given writer
    pub const fn new(writer: W) -> Self {
        Self {
            writer,
            _phantom: PhantomData,
        }
    }

    /// Writes a frame to the underlying stream
    #[expect(
        clippy::cast_possible_truncation,
        reason = "Frame protocol uses u32 length prefix"
    )]
    pub async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        // Write length header (big endian)
        let length = frame.payload.len() as u32;
        self.writer
            .write_all(&length.to_be_bytes())
            .await
            .map_err(|e| {
                MCPError::Transport(format!("Failed to write frame header: {e}").into())
            })?;

        // Write payload
        self.writer.write_all(&frame.payload).await.map_err(|e| {
            MCPError::Transport(format!("Failed to write frame payload: {e}").into())
        })?;

        Ok(())
    }

    /// Flushes any buffered data to the underlying stream
    pub async fn flush(&mut self) -> Result<()> {
        use tokio::io::AsyncWriteExt;
        self.writer.flush().await.map_err(|e| {
            MCPError::Transport(format!("Failed to flush frame writer: {e}").into())
        })?;
        Ok(())
    }
}

/// Frame codec for encoding/decoding frames
pub trait FrameCodec: Send + Sync + std::fmt::Debug {
    /// Error type for encode/decode operations
    type Error;

    /// Encodes a frame into bytes
    fn encode(&self, frame: &Frame) -> std::result::Result<Bytes, Self::Error>;
    /// Decodes bytes into a frame
    fn decode(&self, data: &[u8]) -> std::result::Result<Frame, Self::Error>;
}

/// Default frame codec (no-op)
#[derive(Debug)]
pub struct DefaultFrameCodec;

impl FrameCodec for DefaultFrameCodec {
    type Error = MCPError;

    fn encode(&self, frame: &Frame) -> std::result::Result<Bytes, Self::Error> {
        // Bytes::clone is O(1) ref-count increment — zero-copy
        Ok(frame.payload.clone())
    }

    fn decode(&self, data: &[u8]) -> std::result::Result<Frame, Self::Error> {
        Ok(Frame {
            payload: Bytes::copy_from_slice(data),
        })
    }
}

/// Bidirectional frame transport over async read/write streams
pub struct FrameTransport<R, W, C = DefaultFrameCodec> {
    reader: AsyncFrameReader<R>,
    writer: AsyncFrameWriter<W>,
    #[expect(dead_code, reason = "planned feature not yet wired")]
    codec: C,
}

impl<R, W, C> FrameTransport<R, W, C>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
    C: FrameCodec,
{
    /// Creates a new frame transport with the given reader, writer, and codec
    pub const fn new(reader: R, writer: W, codec: C) -> Self {
        Self {
            reader: AsyncFrameReader::new(reader),
            writer: AsyncFrameWriter::new(writer),
            codec,
        }
    }

    /// Reads a frame from the transport
    pub async fn read_frame(&mut self) -> Result<Frame> {
        self.reader.read_frame().await
    }

    /// Writes a frame to the transport
    pub async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
        self.writer.write_frame(frame).await
    }

    /// Flushes any buffered data
    pub async fn flush(&mut self) -> Result<()> {
        self.writer.flush().await
    }
}

/// Framed stream wrapper for MCP frame encoding/decoding
pub struct FramedStream<T, C = DefaultFrameCodec> {
    /// Underlying stream (reserved for frame-based streaming system)
    #[expect(dead_code, reason = "planned feature not yet wired")]
    stream: T,
    /// Frame codec (reserved for frame-based streaming system)
    #[expect(dead_code, reason = "planned feature not yet wired")]
    codec: C,
}

impl<T, C> FramedStream<T, C>
where
    T: AsyncRead + AsyncWrite + Unpin,
    C: FrameCodec,
{
    /// Creates a new framed stream with the given stream and codec
    pub const fn new(stream: T, codec: C) -> Self {
        Self { stream, codec }
    }
}

/// Message codec for MCP messages
#[derive(Clone)]
pub struct MessageCodec {
    /// Frame codec for encoding/decoding frames (reserved for future extensibility)
    #[expect(dead_code, reason = "planned feature not yet wired")]
    frame_codec: Arc<dyn FrameCodec<Error = MCPError> + Send + Sync>,
}

impl std::fmt::Debug for MessageCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageCodec")
            .field("frame_codec", &"<dyn FrameCodec>")
            .finish()
    }
}

impl MessageCodec {
    /// Creates a new message codec with default frame codec
    #[must_use]
    pub fn new() -> Self {
        Self {
            frame_codec: Arc::new(DefaultFrameCodec),
        }
    }

    /// Creates a message codec with a custom frame codec
    pub fn with_frame_codec(
        frame_codec: Arc<dyn FrameCodec<Error = MCPError> + Send + Sync>,
    ) -> Self {
        Self { frame_codec }
    }

    /// Encode an MCP message into a frame
    pub fn encode_message(&self, message: &MCPMessage) -> Result<Frame> {
        let json = serde_json::to_string(message)
            .map_err(|e| MCPError::Transport(format!("Failed to serialize message: {e}").into()))?;

        Ok(Frame {
            payload: Bytes::from(json.into_bytes()),
        })
    }

    /// Decode a frame into an MCP message
    pub fn decode_message(&self, frame: &Frame) -> Result<MCPMessage> {
        let json = std::str::from_utf8(&frame.payload)
            .map_err(|e| MCPError::Transport(format!("Invalid UTF-8 in frame: {e}").into()))?;

        let message = serde_json::from_str(json).map_err(|e| {
            MCPError::Transport(format!("Failed to deserialize message: {e}").into())
        })?;

        Ok(message)
    }
}

impl Default for MessageCodec {
    fn default() -> Self {
        Self::new()
    }
}

/// Frame stream trait
pub trait FrameStream: Send + Sync {
    /// Read a frame from the stream
    fn read_frame(&mut self) -> impl std::future::Future<Output = Result<Frame>> + Send;

    /// Write a frame to the stream
    fn write_frame(
        &mut self,
        frame: &Frame,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Flush the stream
    fn flush(&mut self) -> impl std::future::Future<Output = Result<()>> + Send;
}

/// Implements `FrameStream` for `FrameTransport`
impl<R, W, C> FrameStream for FrameTransport<R, W, C>
where
    R: AsyncRead + Unpin + Send + Sync,
    W: AsyncWrite + Unpin + Send + Sync,
    C: FrameCodec + Send + Sync,
{
    async fn read_frame(&mut self) -> Result<Frame> {
        self.read_frame().await
    }

    async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
        self.write_frame(frame).await
    }

    async fn flush(&mut self) -> Result<()> {
        self.flush().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_frame_new_and_size() {
        let payload = vec![1u8, 2, 3, 4, 5];
        let frame = Frame::new(payload.clone());
        assert_eq!(frame.payload.as_ref(), &payload[..]);
        assert_eq!(frame.size(), FRAME_HEADER_SIZE + 5);
    }

    #[test]
    fn test_frame_from_vec() {
        let data = vec![10u8, 20, 30];
        let frame = Frame::from_vec(data);
        assert_eq!(frame.payload.as_ref(), &[10, 20, 30]);
    }

    #[test]
    fn test_frame_from_bytes() {
        let bytes = Bytes::from_static(&[7u8, 8, 9]);
        let frame = Frame::from_bytes(&bytes);
        assert_eq!(frame.payload.as_ref(), &[7, 8, 9]);
    }

    #[test]
    fn test_frame_write_and_read_sync() {
        let payload = vec![1u8, 2, 3, 4, 5];
        let frame = Frame::new(payload);

        let mut buf = Vec::new();
        frame.write_to(&mut buf).expect("write");
        assert_eq!(buf.len(), FRAME_HEADER_SIZE + 5);
        assert_eq!(&buf[0..4], 5u32.to_be_bytes());

        let mut cursor = Cursor::new(&buf);
        let read_frame = Frame::read_from(&mut cursor).expect("read");
        assert_eq!(read_frame.payload.as_ref(), frame.payload.as_ref());
    }

    #[test]
    fn test_frame_read_rejects_oversized() {
        let mut buf = vec![0xff, 0xff, 0xff, 0xff]; // u32::MAX
        buf.resize(4 + 100, 0);
        let mut cursor = Cursor::new(&buf);
        let result = Frame::read_from(&mut cursor);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
    }

    #[test]
    fn test_default_frame_codec_encode_decode() {
        let codec = DefaultFrameCodec;
        let frame = Frame::new(vec![1, 2, 3]);
        let encoded = codec.encode(&frame).expect("encode");
        assert_eq!(encoded.as_ref(), &[1, 2, 3]);
        let decoded = codec.decode(&encoded).expect("decode");
        assert_eq!(decoded.payload.as_ref(), &[1, 2, 3]);
    }

    #[tokio::test]
    async fn test_message_codec_encode_decode() {
        use crate::protocol::types::{MCPMessage, MessageType};

        let codec = MessageCodec::new();
        let message = MCPMessage::new(MessageType::Command, serde_json::json!({"cmd": "test"}));
        let frame = codec.encode_message(&message).expect("encode");
        assert!(!frame.payload.is_empty());

        let decoded = codec.decode_message(&frame).expect("decode");
        assert_eq!(decoded.type_, MessageType::Command);
        assert_eq!(
            decoded.payload.get("cmd").and_then(|v| v.as_str()),
            Some("test")
        );
    }

    #[tokio::test]
    async fn test_message_codec_decode_invalid_utf8() {
        let codec = MessageCodec::new();
        let frame = Frame::new(vec![0xff, 0xfe, 0xfd]); // Invalid UTF-8
        let result = codec.decode_message(&frame);
        assert!(result.is_err());
    }

    #[test]
    fn test_message_codec_default() {
        let _ = MessageCodec::default();
    }
}
