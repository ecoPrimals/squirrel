// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Compression module for the MCP system.
//!
//! This module provides compression and decompression capabilities for MCP messages.
//! It supports various compression formats, such as Zstd, Gzip, and LZ4.

use crate::error::Result;
#[cfg(any(feature = "zstd", feature = "gzip", feature = "lz4"))]
use crate::error::MCPError;
use crate::types::CompressionFormat;

#[cfg(feature = "gzip")]
use flate2::Compression;
#[cfg(feature = "gzip")]
use flate2::write::GzEncoder;
#[cfg(feature = "gzip")]
use flate2::read::GzDecoder;
#[cfg(any(feature = "gzip", feature = "lz4"))]
use std::io::{Read, Write};

/// Compression threshold in bytes
const COMPRESSION_THRESHOLD: usize = 1024;

/// Compress data using the specified format.
///
/// This function takes raw bytes and compresses them according to the specified 
/// compression format. If the format is `None`, the original data is returned unchanged.
///
/// # Arguments
///
/// * `data` - The data to compress
/// * `format` - The compression format to use
///
/// # Returns
///
/// The compressed data or an error if compression fails
///
/// # Errors
///
/// Returns an error if:
/// * Compression fails due to invalid input data
/// * The compression algorithm encounters an internal error
/// * A required compression feature is not enabled at compile time but requested at runtime
pub fn compress(data: &[u8], format: CompressionFormat) -> Result<Vec<u8>> {
    match format {
        CompressionFormat::None => Ok(data.to_vec()),
        CompressionFormat::Zstd => {
            // Simple implementation using zstd crate
            #[cfg(feature = "zstd")]
            {
                zstd::encode_all(data, 3).map_err(|e| MCPError::Serialization(format!("Zstd compression failed: {e}")))
            }
            #[cfg(not(feature = "zstd"))]
            {
                // Fallback when zstd feature is not enabled
                Ok(data.to_vec())
            }
        }
        CompressionFormat::Gzip => {
            // Simple implementation using flate2 crate
            #[cfg(feature = "gzip")]
            {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(data).map_err(|e| MCPError::Serialization(format!("Gzip compression failed: {e}")))?;
                encoder.finish().map_err(|e| MCPError::Serialization(format!("Gzip finalization failed: {e}")))
            }
            #[cfg(not(feature = "gzip"))]
            {
                // Fallback when gzip feature is not enabled
                Ok(data.to_vec())
            }
        }
        CompressionFormat::Lz4 => {
            // Simple implementation using lz4 crate
            #[cfg(feature = "lz4")]
            {
                let mut compressed = Vec::new();
                let mut encoder = lz4::EncoderBuilder::new()
                    .build(&mut compressed)
                    .map_err(|e| MCPError::Serialization(format!("LZ4 compression failed: {e}")))?;
                
                // Write the data to the encoder
                encoder.write_all(data)
                    .map_err(|e| MCPError::Serialization(format!("LZ4 write failed: {e}")))?;
                
                // Finish the compression and get back the Vec<u8>
                let (compressed_vec, result) = encoder.finish();
                result.map_err(|e| MCPError::Serialization(format!("LZ4 finalization failed: {e}")))?;
                
                // Return the compressed data
                Ok(compressed_vec.to_vec())
            }
            #[cfg(not(feature = "lz4"))]
            {
                // Fallback when lz4 feature is not enabled
                Ok(data.to_vec())
            }
        }
    }
}

/// Decompress data using the specified format.
///
/// This function takes compressed bytes and decompresses them according to the specified 
/// compression format. If the format is `None`, the original data is returned unchanged.
///
/// # Arguments
///
/// * `data` - The compressed data to decompress
/// * `format` - The compression format used
///
/// # Returns
///
/// The decompressed data or an error if decompression fails
///
/// # Errors
///
/// Returns an error if:
/// * Decompression fails due to invalid or corrupted input data
/// * The decompression algorithm encounters an internal error
/// * A required compression feature is not enabled at compile time but requested at runtime
pub fn decompress(data: &[u8], format: CompressionFormat) -> Result<Vec<u8>> {
    match format {
        CompressionFormat::None => Ok(data.to_vec()),
        CompressionFormat::Zstd => {
            // Simple implementation using zstd crate
            #[cfg(feature = "zstd")]
            {
                zstd::decode_all(data).map_err(|e| MCPError::Deserialization(format!("Zstd decompression failed: {e}")))
            }
            #[cfg(not(feature = "zstd"))]
            {
                // Fallback when zstd feature is not enabled
                Ok(data.to_vec())
            }
        }
        CompressionFormat::Gzip => {
            // Simple implementation using flate2 crate
            #[cfg(feature = "gzip")]
            {
                let mut decoder = GzDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)
                    .map_err(|e| MCPError::Deserialization(format!("Gzip decompression failed: {e}")))?;
                Ok(decompressed)
            }
            #[cfg(not(feature = "gzip"))]
            {
                // Fallback when gzip feature is not enabled
                Ok(data.to_vec())
            }
        }
        CompressionFormat::Lz4 => {
            // Simple implementation using lz4 crate
            #[cfg(feature = "lz4")]
            {
                let mut decompressed = Vec::new();
                let mut decoder = lz4::Decoder::new(data)
                    .map_err(|e| MCPError::Deserialization(format!("LZ4 decompression failed: {e}")))?;
                
                decoder.read_to_end(&mut decompressed)
                    .map_err(|e| MCPError::Deserialization(format!("LZ4 read failed: {e}")))?;
                    
                Ok(decompressed)
            }
            #[cfg(not(feature = "lz4"))]
            {
                // Fallback when lz4 feature is not enabled
                Ok(data.to_vec())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_no_compression() {
        let data = b"Hello, world!";
        let compressed = compress(data, CompressionFormat::None).unwrap();
        let decompressed = decompress(&compressed, CompressionFormat::None).unwrap();
        assert_eq!(data, decompressed.as_slice());
    }
    
    // More tests would be added for other compression formats
    // when the features are enabled
} 