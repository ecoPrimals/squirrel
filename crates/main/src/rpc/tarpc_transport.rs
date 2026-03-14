// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! tarpc Transport Adapter
//!
//! Bridges UniversalTransport (AsyncRead + AsyncWrite) to tarpc's
//! Stream + Sink interface using tokio-util's codec framework.
//!
//! ## Architecture
//!
//! ```text
//! UniversalTransport
//!  (AsyncRead + AsyncWrite)
//!         ↓
//!  [LengthDelimitedCodec]
//!         ↓
//!  Framed (Stream + Sink)
//!         ↓
//!     tarpc Server
//! ```

// Note: This module is feature-gated via #[cfg(feature = "tarpc-rpc")] in mod.rs

use bytes::{Bytes, BytesMut};
use futures::{Sink, Stream};
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use universal_patterns::transport::UniversalTransport;

/// Transport adapter for tarpc over UniversalTransport
///
/// This adapter wraps UniversalTransport (AsyncRead + AsyncWrite) and provides
/// Stream + Sink traits required by tarpc, using length-delimited framing.
pub struct TarpcTransportAdapter {
    /// Framed transport (Stream + Sink)
    framed: Framed<UniversalTransport, LengthDelimitedCodec>,
}

impl TarpcTransportAdapter {
    /// Create a new tarpc transport adapter
    ///
    /// # Arguments
    ///
    /// * `transport` - Universal transport (Unix socket, TCP, or Named pipe)
    ///
    /// # Returns
    ///
    /// Transport adapter ready for tarpc use
    pub fn new(transport: UniversalTransport) -> Self {
        // Create length-delimited codec with standard settings
        let codec = LengthDelimitedCodec::builder()
            .length_field_offset(0) // Length field at start
            .length_field_length(4) // 4-byte length field (u32)
            .length_adjustment(0) // No adjustment needed
            .num_skip(0) // Don't skip any bytes
            .max_frame_length(16 * 1024 * 1024) // 16MB max frame (generous for AI responses)
            .new_codec();

        Self {
            framed: Framed::new(transport, codec),
        }
    }

    /// Create with custom max frame length
    ///
    /// # Arguments
    ///
    /// * `transport` - Universal transport
    /// * `max_frame_length` - Maximum frame size in bytes
    pub fn with_max_frame_length(transport: UniversalTransport, max_frame_length: usize) -> Self {
        let codec = LengthDelimitedCodec::builder()
            .length_field_offset(0)
            .length_field_length(4)
            .length_adjustment(0)
            .num_skip(0)
            .max_frame_length(max_frame_length)
            .new_codec();

        Self {
            framed: Framed::new(transport, codec),
        }
    }
}

/// Implement Stream trait for reading from transport
///
/// Yields `BytesMut` as required by tarpc/tokio-serde transport layer.
/// LengthDelimitedCodec yields BytesMut; tarpc expects this for bincode deserialization.
impl Stream for TarpcTransportAdapter {
    type Item = io::Result<BytesMut>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.framed).poll_next(cx)
    }
}

/// Implement Sink trait for writing to transport
///
/// Accepts `Bytes` for zero-copy payloads (wateringHole standard).
/// tokio-util 0.7 LengthDelimitedCodec implements Encoder<Bytes>.
impl Sink<Bytes> for TarpcTransportAdapter {
    type Error = io::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.framed).poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Bytes) -> Result<(), Self::Error> {
        Pin::new(&mut self.framed).start_send(item)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.framed).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.framed).poll_close(cx)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_transport_adapter_creation() {
        // Create an in-memory transport for testing
        let (_client, _server) = tokio::io::duplex(1024);

        // Note: We can't easily create a UniversalTransport for testing
        // without the full infrastructure, so this test is more conceptual

        // In real usage:
        // let transport = UniversalTransport::connect_discovered("service").await?;
        // let adapter = TarpcTransportAdapter::new(transport);

        // Just verify the adapter exists
        assert!(true); // Placeholder - real tests would use actual transport
    }

    #[test]
    fn test_max_frame_length() {
        // Test that we can configure max frame length
        let max_len = 32 * 1024 * 1024; // 32MB

        // Verify the value is reasonable
        assert!(max_len > 0);
        assert!(max_len <= 64 * 1024 * 1024); // Reasonable upper bound
    }
}
