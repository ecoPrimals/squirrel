// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Message Frame Transport
//!
//! This module provides frame-based message transport for MCP protocol.

use std::io::{Read, Write};
use std::marker::PhantomData;
use std::sync::Arc;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::error::{MCPError, Result};
use crate::protocol::types::MCPMessage;

/// Maximum frame size (16MB)
pub const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;

/// Frame header size (4 bytes for length)
pub const FRAME_HEADER_SIZE: usize = 4;

/// Frame represents a message frame with header and payload
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    /// Raw frame payload bytes
    #[serde(with = "serde_bytes")]
    pub payload: Vec<u8>,
}

impl Frame {
    /// Create a new frame with the given payload
    pub fn new(payload: Vec<u8>) -> Self {
        Self { payload }
    }

    /// Create a frame from a byte vector
    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { payload: data }
    }

    /// Create a frame from Bytes
    pub fn from_bytes(bytes: Bytes) -> Self {
        Self {
            payload: bytes.to_vec(),
        }
    }

    /// Get the frame size including header
    pub fn size(&self) -> usize {
        FRAME_HEADER_SIZE + self.payload.len()
    }

    /// Write frame to a writer
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

        Ok(Self { payload })
    }
}

/// Async frame reader for reading length-prefixed frames from a stream
pub struct AsyncFrameReader<R> {
    reader: R,
    _phantom: PhantomData<R>,
}

impl<R: AsyncRead + Unpin> AsyncFrameReader<R> {
    /// Creates a new async frame reader wrapping the given reader
    pub fn new(reader: R) -> Self {
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

        Ok(Frame { payload })
    }
}

/// Async frame writer for writing length-prefixed frames to a stream
pub struct AsyncFrameWriter<W> {
    writer: W,
    _phantom: PhantomData<W>,
}

impl<W: AsyncWrite + Unpin> AsyncFrameWriter<W> {
    /// Creates a new async frame writer wrapping the given writer
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            _phantom: PhantomData,
        }
    }

    /// Writes a frame to the underlying stream
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
        Ok(Bytes::from(frame.payload.clone()))
    }

    fn decode(&self, data: &[u8]) -> std::result::Result<Frame, Self::Error> {
        Ok(Frame::from_vec(data.to_vec()))
    }
}

/// Bidirectional frame transport over async read/write streams
pub struct FrameTransport<R, W, C = DefaultFrameCodec> {
    reader: AsyncFrameReader<R>,
    writer: AsyncFrameWriter<W>,
    #[allow(dead_code)] // Reserved for future frame encoding customization
    codec: C,
}

impl<R, W, C> FrameTransport<R, W, C>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
    C: FrameCodec,
{
    /// Creates a new frame transport with the given reader, writer, and codec
    pub fn new(reader: R, writer: W, codec: C) -> Self {
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
    #[allow(dead_code)]
    stream: T,
    /// Frame codec (reserved for frame-based streaming system)
    #[allow(dead_code)]
    codec: C,
}

impl<T, C> FramedStream<T, C>
where
    T: AsyncRead + AsyncWrite + Unpin,
    C: FrameCodec,
{
    /// Creates a new framed stream with the given stream and codec
    pub fn new(stream: T, codec: C) -> Self {
        Self { stream, codec }
    }
}

/// Message codec for MCP messages
#[derive(Clone)]
pub struct MessageCodec {
    /// Frame codec for encoding/decoding frames (reserved for future extensibility)
    #[allow(dead_code)]
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
    pub async fn encode_message(&self, message: &MCPMessage) -> Result<Frame> {
        let json = serde_json::to_string(message)
            .map_err(|e| MCPError::Transport(format!("Failed to serialize message: {e}").into()))?;

        let frame = Frame::new(json.into_bytes());
        Ok(frame)
    }

    /// Decode a frame into an MCP message
    pub async fn decode_message(&self, frame: &Frame) -> Result<MCPMessage> {
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

/// Implements FrameStream for FrameTransport
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
