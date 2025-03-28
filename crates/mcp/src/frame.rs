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
//!
//! // Create a frame for a message
//! let message_bytes = b"Hello, World!".to_vec();
//! let frame = Frame::new(message_bytes);
//!
//! // Encode the frame
//! let mut encoder = FrameEncoder::new();
//! let mut buffer = BytesMut::new();
//! encoder.encode(frame, &mut buffer).unwrap();
//!
//! // Later, decode the frame from the buffer
//! let mut decoder = FrameDecoder::new();
//! let decoded_frame = decoder.decode(&mut buffer).unwrap().unwrap();
//! 
//! assert_eq!(decoded_frame.payload(), b"Hello, World!");
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