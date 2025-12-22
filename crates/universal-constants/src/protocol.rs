//! Protocol Configuration Constants
//!
//! Protocol-specific constants used throughout the Squirrel system,
//! consolidated from `crates/core/mcp/src/constants.rs` and
//! `crates/core/mcp/src/protocol/constants.rs`.
//!
//! # Categories
//!
//! - **MCP Protocol**: Machine Context Protocol constants
//! - **HTTP Headers**: Standard HTTP headers and values
//! - **WebSocket**: WebSocket protocol constants

// ============================================================================
// MCP Protocol
// ============================================================================

/// Default MCP subprotocol identifier
pub const DEFAULT_MCP_SUBPROTOCOL: &str = "mcp";

/// Default MCP protocol version
pub const DEFAULT_PROTOCOL_VERSION: &str = "1.0";

/// Default MCP protocol version (v2)
pub const PROTOCOL_VERSION_V2: &str = "2.0";

// ============================================================================
// HTTP Headers & Content Types
// ============================================================================

/// Default user agent string
pub const DEFAULT_USER_AGENT: &str = "squirrel-mcp/1.0";

/// Default content type
pub const DEFAULT_CONTENT_TYPE: &str = "application/json";

/// JSON content type
pub const CONTENT_TYPE_JSON: &str = "application/json";

/// Binary content type
pub const CONTENT_TYPE_BINARY: &str = "application/octet-stream";

/// Text content type
pub const CONTENT_TYPE_TEXT: &str = "text/plain";

// ============================================================================
// Character Sets & Encodings
// ============================================================================

/// Default character encoding
pub const DEFAULT_CHARSET: &str = "utf-8";

// ============================================================================
// Protocol Features
// ============================================================================

/// Feature flag: Bidirectional streaming support
pub const FEATURE_BIDIRECTIONAL_STREAMING: &str = "bidirectional-streaming";

/// Feature flag: Multi-agent support
pub const FEATURE_MULTI_AGENT: &str = "multi-agent";

/// Feature flag: Context preservation
pub const FEATURE_CONTEXT_PRESERVATION: &str = "context-preservation";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_protocol() {
        assert_eq!(DEFAULT_MCP_SUBPROTOCOL, "mcp");
        assert_eq!(DEFAULT_PROTOCOL_VERSION, "1.0");
        assert_eq!(PROTOCOL_VERSION_V2, "2.0");
    }

    #[test]
    fn test_http_headers() {
        assert_eq!(DEFAULT_USER_AGENT, "squirrel-mcp/1.0");
        assert_eq!(DEFAULT_CONTENT_TYPE, "application/json");
        assert_eq!(CONTENT_TYPE_JSON, "application/json");
    }

    #[test]
    fn test_charset() {
        assert_eq!(DEFAULT_CHARSET, "utf-8");
    }

    #[test]
    fn test_features() {
        assert_eq!(FEATURE_BIDIRECTIONAL_STREAMING, "bidirectional-streaming");
        assert_eq!(FEATURE_MULTI_AGENT, "multi-agent");
        assert_eq!(FEATURE_CONTEXT_PRESERVATION, "context-preservation");
    }
}
