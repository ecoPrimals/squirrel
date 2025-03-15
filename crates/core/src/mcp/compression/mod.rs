use crate::error::{MCPError, Result};
use crate::mcp::types::CompressionFormat;
use std::io::{Read, Write};
use zstd::stream::{copy_encode, copy_decode};
use flate2::read::{GzEncoder, GzDecoder};
use lz4::EncoderBuilder as Lz4EncoderBuilder;
use lz4::Decoder as Lz4Decoder;

/// Compresses data using the specified compression format
pub fn compress(data: &[u8], format: CompressionFormat) -> Result<Vec<u8>> {
    match format {
        CompressionFormat::None => Ok(data.to_vec()),
        CompressionFormat::Gzip => compress_gzip(data),
        CompressionFormat::Zstd => compress_zstd(data),
        CompressionFormat::Lz4 => compress_lz4(data),
    }
}

/// Decompresses data using the specified compression format
pub fn decompress(data: &[u8], format: CompressionFormat) -> Result<Vec<u8>> {
    match format {
        CompressionFormat::None => Ok(data.to_vec()),
        CompressionFormat::Gzip => decompress_gzip(data),
        CompressionFormat::Zstd => decompress_zstd(data),
        CompressionFormat::Lz4 => decompress_lz4(data),
    }
}

// Zstd compression
fn compress_zstd(data: &[u8]) -> Result<Vec<u8>> {
    let mut compressed = Vec::new();
    zstd::stream::copy_encode(data, &mut compressed, 3)
        .map_err(|e| MCPError::Compression(format!("Zstd compression failed: {}", e)))?;
    Ok(compressed)
}

fn decompress_zstd(data: &[u8]) -> Result<Vec<u8>> {
    let mut decompressed = Vec::new();
    zstd::stream::copy_decode(data, &mut decompressed)
        .map_err(|e| MCPError::Compression(format!("Zstd decompression failed: {}", e)))?;
    Ok(decompressed)
}

// Gzip compression
fn compress_gzip(data: &[u8]) -> Result<Vec<u8>> {
    let mut compressed = Vec::new();
    let mut encoder = GzEncoder::new(data, flate2::Compression::default());
    std::io::copy(&mut encoder, &mut compressed)
        .map_err(|e| MCPError::Compression(format!("Gzip compression failed: {}", e)))?;
    Ok(compressed)
}

fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>> {
    let mut decompressed = Vec::new();
    let mut decoder = GzDecoder::new(data);
    decoder.read_to_end(&mut decompressed)
        .map_err(|e| MCPError::Compression(format!("Gzip decompression failed: {}", e)))?;
    Ok(decompressed)
}

// LZ4 compression
fn compress_lz4(data: &[u8]) -> Result<Vec<u8>> {
    let mut compressed = Vec::new();
    let mut encoder = Lz4EncoderBuilder::new()
        .build(&mut compressed)
        .map_err(|e| MCPError::Compression(format!("LZ4 encoder creation failed: {}", e)))?;
    encoder.write_all(data)
        .map_err(|e| MCPError::Compression(format!("LZ4 compression failed: {}", e)))?;
    encoder.finish()
        .0
        .map_err(|e| MCPError::Compression(format!("LZ4 compression finalization failed: {}", e)))
}

fn decompress_lz4(data: &[u8]) -> Result<Vec<u8>> {
    let mut decompressed = Vec::new();
    let mut decoder = Lz4Decoder::new(data)
        .map_err(|e| MCPError::Compression(format!("LZ4 decoder creation failed: {}", e)))?;
    decoder.read_to_end(&mut decompressed)
        .map_err(|e| MCPError::Compression(format!("LZ4 decompression failed: {}", e)))?;
    Ok(decompressed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_roundtrip() {
        let test_data = b"Hello, World! This is a test message for compression.";
        
        for format in &[
            CompressionFormat::None,
            CompressionFormat::Gzip,
            CompressionFormat::Zstd,
            CompressionFormat::Lz4,
        ] {
            let compressed = compress(test_data, *format).unwrap();
            let decompressed = decompress(&compressed, *format).unwrap();
            assert_eq!(&decompressed, test_data);
        }
    }

    #[test]
    fn test_compression_ratio() {
        let test_data = "Hello, World! ".repeat(1000).into_bytes();
        
        for format in &[
            CompressionFormat::Gzip,
            CompressionFormat::Zstd,
            CompressionFormat::Lz4,
        ] {
            let compressed = compress(&test_data, *format).unwrap();
            assert!(compressed.len() < test_data.len(), 
                "Compression format {:?} should reduce data size", format);
        }
    }
} 