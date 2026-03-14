// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Size, Count, and Capacity Limit Constants
//!
//! All limit values used throughout the Squirrel system, consolidated from:
//! - `crates/config/src/constants.rs`
//! - `crates/core/mcp/src/constants.rs`
//!
//! # Categories
//!
//! - **Connection Limits**: Maximum connections, concurrent operations
//! - **Buffer Sizes**: Memory buffers, channel sizes
//! - **Message Sizes**: Maximum message and payload sizes
//! - **Service Limits**: Maximum services, registrations

// ============================================================================
// Connection Limits
// ============================================================================

/// Default maximum connections (100)
///
/// Maximum number of concurrent connections a service will accept.
pub const DEFAULT_MAX_CONNECTIONS: usize = 100;

/// Default maximum services (1000)
///
/// Maximum number of services that can be registered in the service mesh.
pub const DEFAULT_MAX_SERVICES: usize = 1000;

/// Default maximum retries (3)
///
/// Maximum number of retry attempts for failed operations.
pub const DEFAULT_MAX_RETRIES: u32 = 3;

// ============================================================================
// Buffer Sizes
// ============================================================================

/// Default buffer size (8KB)
///
/// Standard buffer size for I/O operations.
pub const DEFAULT_BUFFER_SIZE: usize = 8 * 1024; // 8KB

/// Default chunk size (4KB)
///
/// Size of chunks for streaming operations.
pub const DEFAULT_CHUNK_SIZE: usize = 4 * 1024; // 4KB

/// Default channel buffer size (1000)
///
/// Size of async channel buffers.
pub const DEFAULT_CHANNEL_BUFFER_SIZE: usize = 1000;

// ============================================================================
// Message Sizes
// ============================================================================

/// Default maximum message size (16MB)
///
/// Maximum size for a single message or payload.
pub const DEFAULT_MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024; // 16MB

/// Default maximum context length (128K tokens)
///
/// Maximum context length for AI operations.
pub const DEFAULT_MAX_CONTEXT_LENGTH: usize = 128_000;

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert bytes to kilobytes
#[must_use]
pub const fn bytes_to_kb(bytes: usize) -> usize {
    bytes / 1024
}

/// Convert bytes to megabytes
#[must_use]
pub const fn bytes_to_mb(bytes: usize) -> usize {
    bytes / (1024 * 1024)
}

/// Convert kilobytes to bytes
#[must_use]
pub const fn kb_to_bytes(kb: usize) -> usize {
    kb * 1024
}

/// Convert megabytes to bytes
#[must_use]
pub const fn mb_to_bytes(mb: usize) -> usize {
    mb * 1024 * 1024
}

#[cfg(test)]
mod tests {
    use super::{
        bytes_to_kb, bytes_to_mb, kb_to_bytes, mb_to_bytes, DEFAULT_BUFFER_SIZE,
        DEFAULT_CHANNEL_BUFFER_SIZE, DEFAULT_CHUNK_SIZE, DEFAULT_MAX_CONNECTIONS,
        DEFAULT_MAX_CONTEXT_LENGTH, DEFAULT_MAX_MESSAGE_SIZE, DEFAULT_MAX_RETRIES,
        DEFAULT_MAX_SERVICES,
    };

    #[test]
    fn test_connection_limits() {
        assert_eq!(DEFAULT_MAX_CONNECTIONS, 100);
        assert_eq!(DEFAULT_MAX_SERVICES, 1000);
        assert_eq!(DEFAULT_MAX_RETRIES, 3);
    }

    #[test]
    fn test_buffer_sizes() {
        assert_eq!(DEFAULT_BUFFER_SIZE, 8192);
        assert_eq!(DEFAULT_CHUNK_SIZE, 4096);
        assert_eq!(DEFAULT_CHANNEL_BUFFER_SIZE, 1000);
    }

    #[test]
    fn test_message_sizes() {
        assert_eq!(DEFAULT_MAX_MESSAGE_SIZE, 16 * 1024 * 1024);
        assert_eq!(DEFAULT_MAX_CONTEXT_LENGTH, 128_000);
    }

    #[test]
    fn test_size_conversions() {
        assert_eq!(bytes_to_kb(8192), 8);
        assert_eq!(bytes_to_mb(16 * 1024 * 1024), 16);
        assert_eq!(kb_to_bytes(8), 8192);
        assert_eq!(mb_to_bytes(16), 16 * 1024 * 1024);
    }
}
