// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Frame module for message encoding and transmission
//!
//! This module provides functionality for framing MCP messages before transmission
//! over a transport layer. Framing ensures that message boundaries are preserved
//! and that messages can be properly reconstructed at the receiving end.
//!
//! ## Re-export Details
//!
//! This module re-exports the entire frame module from the transport module.
//! The implementation has been moved to the transport module to keep all
//! transport-related functionality together.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use squirrel_mcp::frame::{Frame, FrameEncoder, FrameDecoder};
//! use bytes::{BytesMut, Buf};
//! use tokio_util::codec::{Decoder, Encoder};
//!
//! // Create a frame for a message
//! let message_bytes = b"Hello, World!".to_vec();
//! let frame = Frame::from_vec(message_bytes.clone());
//!
//! // Encode the frame
//! let mut encoder = FrameEncoder::new();
//! let mut buffer = BytesMut::new();
//! encoder.encode(frame.clone(), &mut buffer).expect("example");
//!
//! // Later, decode the frame from the buffer
//! let mut decoder = FrameDecoder::new();
//! let decoded_frame = decoder.decode(&mut buffer).expect("example").expect("example");
//! 
//! // Compare payload content with original bytes
//! assert_eq!(decoded_frame.payload().len(), message_bytes.len());
//! for (i, b) in message_bytes.iter().enumerate() {
//!     assert_eq!(decoded_frame.payload()[i], *b);
//! }
//! ```
//!
//! ## Frame Structure
//!
//! Each frame consists of:
//!
//! 1. A header containing the message length (4 bytes, big-endian)
//! 2. A payload containing the actual message data
//!
//! For more details on the frame implementation, see the documentation in the
//! transport module.

// Re-export the frame module from transport
pub use crate::transport::frame::*; 