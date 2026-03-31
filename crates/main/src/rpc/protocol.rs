// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! RPC Protocol Abstraction
//!
//! This module defines the protocol selection layer that allows Squirrel
//! to support multiple RPC protocols (JSON-RPC 2.0, tarpc) with automatic
//! protocol negotiation.
//!
//! ## Philosophy
//!
//! - **Capability-Based**: Protocol selection based on client capabilities
//! - **Backward Compatible**: JSON-RPC 2.0 as default/fallback
//! - **Performance**: tarpc for high-performance binary RPC
//! - **Gradual Evolution**: No big-bang migration required

use serde::{Deserialize, Serialize};
use std::fmt;

/// RPC Protocol selector
///
/// Defines which RPC protocol to use for communication.
/// Supports multiple protocols for gradual evolution and performance optimization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum IpcProtocol {
    /// JSON-RPC 2.0 (text-based, human-readable)
    ///
    /// - Default protocol
    /// - Backward compatible
    /// - Easy debugging
    /// - Language-agnostic
    #[default]
    JsonRpc,

    /// tarpc (binary, type-safe, high-performance)
    ///
    /// - Pure Rust RPC framework
    /// - Lower latency
    /// - Smaller payloads
    /// - Type safety
    /// - Cascading cancellation
    /// - Deadline propagation
    #[cfg(feature = "tarpc-rpc")]
    Tarpc,
}

impl fmt::Display for IpcProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JsonRpc => write!(f, "jsonrpc"),
            #[cfg(feature = "tarpc-rpc")]
            Self::Tarpc => write!(f, "tarpc"),
        }
    }
}

impl IpcProtocol {
    /// Parse protocol from string
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel::rpc::protocol::IpcProtocol;
    ///
    /// assert_eq!(IpcProtocol::from_str("jsonrpc"), Some(IpcProtocol::JsonRpc));
    /// assert_eq!(IpcProtocol::from_str("invalid"), None);
    /// ```
    #[expect(
        clippy::should_implement_trait,
        reason = "Custom from_str avoids FromStr trait conflict"
    )]
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "jsonrpc" | "json-rpc" | "json_rpc" => Some(Self::JsonRpc),
            #[cfg(feature = "tarpc-rpc")]
            "tarpc" => Some(Self::Tarpc),
            _ => None,
        }
    }

    /// Get all supported protocols
    ///
    /// Returns a list of protocols supported by this build.
    #[must_use]
    pub fn supported() -> Vec<Self> {
        let mut protocols = vec![Self::JsonRpc];
        #[cfg(feature = "tarpc-rpc")]
        protocols.push(Self::Tarpc);
        protocols
    }

    /// Check if a protocol is supported
    #[must_use]
    pub fn is_supported(&self) -> bool {
        Self::supported().contains(self)
    }

    /// Get protocol name for negotiation
    #[must_use]
    pub const fn negotiation_name(&self) -> &'static str {
        match self {
            Self::JsonRpc => "jsonrpc",
            #[cfg(feature = "tarpc-rpc")]
            Self::Tarpc => "tarpc",
        }
    }
}

/// Protocol negotiation result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolNegotiation {
    /// Selected protocol
    pub protocol: IpcProtocol,

    /// Client-requested protocol (if any)
    pub requested: Option<IpcProtocol>,

    /// Negotiation successful
    pub success: bool,

    /// Negotiation message (for logging)
    pub message: String,
}

impl ProtocolNegotiation {
    /// Successful negotiation
    #[must_use]
    pub fn success(protocol: IpcProtocol, requested: Option<IpcProtocol>) -> Self {
        Self {
            protocol,
            requested,
            success: true,
            message: format!("Protocol negotiated: {protocol}"),
        }
    }

    /// Failed negotiation (fallback to default)
    #[must_use]
    pub fn fallback(requested: Option<IpcProtocol>, reason: &str) -> Self {
        Self {
            protocol: IpcProtocol::default(),
            requested,
            success: false,
            message: format!("Protocol negotiation failed ({reason}), using default"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_default() {
        assert_eq!(IpcProtocol::default(), IpcProtocol::JsonRpc);
    }

    #[test]
    fn test_protocol_from_str() {
        assert_eq!(IpcProtocol::from_str("jsonrpc"), Some(IpcProtocol::JsonRpc));
        assert_eq!(
            IpcProtocol::from_str("json-rpc"),
            Some(IpcProtocol::JsonRpc)
        );
        assert_eq!(
            IpcProtocol::from_str("JSON_RPC"),
            Some(IpcProtocol::JsonRpc)
        );

        #[cfg(feature = "tarpc-rpc")]
        {
            assert_eq!(IpcProtocol::from_str("tarpc"), Some(IpcProtocol::Tarpc));
            assert_eq!(IpcProtocol::from_str("TARPC"), Some(IpcProtocol::Tarpc));
        }

        assert_eq!(IpcProtocol::from_str("invalid"), None);
    }

    #[test]
    fn test_protocol_supported() {
        let supported = IpcProtocol::supported();
        assert!(supported.contains(&IpcProtocol::JsonRpc));

        #[cfg(feature = "tarpc-rpc")]
        assert!(supported.contains(&IpcProtocol::Tarpc));

        assert!(IpcProtocol::JsonRpc.is_supported());

        #[cfg(feature = "tarpc-rpc")]
        assert!(IpcProtocol::Tarpc.is_supported());
    }

    #[test]
    fn test_protocol_display() {
        assert_eq!(IpcProtocol::JsonRpc.to_string(), "jsonrpc");

        #[cfg(feature = "tarpc-rpc")]
        assert_eq!(IpcProtocol::Tarpc.to_string(), "tarpc");
    }

    #[test]
    fn test_protocol_negotiation_success() {
        let negotiation = ProtocolNegotiation::success(IpcProtocol::JsonRpc, None);
        assert!(negotiation.success);
        assert_eq!(negotiation.protocol, IpcProtocol::JsonRpc);
    }

    #[test]
    fn test_protocol_negotiation_fallback() {
        let negotiation = ProtocolNegotiation::fallback(None, "invalid protocol");
        assert!(!negotiation.success);
        assert_eq!(negotiation.protocol, IpcProtocol::default());
    }
}
