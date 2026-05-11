// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Pre-dispatch capability gate (JH-0 / JH-2 ecosystem standard).
//!
//! Classifies every JSON-RPC method as [`MethodVisibility::Public`] or
//! [`MethodVisibility::Protected`] and gates dispatch based on the current
//! [`GateMode`]. Ships in [`GateMode::Permissive`] (all calls allowed)
//! per the primalSpring `METHOD_GATE_STANDARD.md` adoption guide.
//!
//! JH-2 adds [`ResourceEnvelope`] enforcement: ionic tokens carry resource
//! limits (`mem_mb`, `cpu_cores`, `method_allowlist`) that are checked at
//! dispatch time. When no token is present and the gate is permissive,
//! dispatch proceeds without limits (backward compatible).
//!
//! Reference: toadStool `method_gate.rs` (14 tests), primalSpring canonical.

use serde::{Deserialize, Serialize};
use tracing::trace;

use super::jsonrpc_types::{JsonRpcError, normalize_method};

/// Ecosystem-standard error code for unauthorized access (identity could not
/// be established). JSON-RPC server-defined range: -32000..-32099.
pub const UNAUTHORIZED: i32 = -32000;

/// Ecosystem-standard error code for permission denied (identity established
/// but insufficient scope/allowlist). JSON-RPC server-defined range.
pub const PERMISSION_DENIED: i32 = -32001;

/// Whether a method is freely callable or requires authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MethodVisibility {
    /// Callable by any peer without credentials.
    Public,
    /// Requires a valid token / caller identity when the gate is enforcing.
    Protected,
}

/// Gate operating mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateMode {
    /// All calls allowed regardless of caller identity (JH-0 default).
    Permissive,
    /// Protected methods require valid authentication (JH-2 future).
    Enforcing,
}

/// Resource limits carried in an ionic token (JH-2).
///
/// When present, dispatch handlers enforce that the requested resources
/// fall within these bounds. Fields are optional — `None` means
/// "unlimited for this dimension".
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceEnvelope {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mem_mb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_cores: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_timeout_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub method_allowlist: Vec<String>,
}

impl ResourceEnvelope {
    /// An empty allowlist means "all methods permitted".
    pub fn allows_method(&self, method: &str) -> bool {
        self.method_allowlist.is_empty() || self.method_allowlist.iter().any(|m| m == method)
    }
}

/// Caller identity and resource context extracted from a request (JH-2).
///
/// Threaded through the dispatch path so handlers can enforce per-caller
/// resource limits. `None` fields mean "no token / unknown".
#[derive(Debug, Clone, Default)]
pub struct CallerContext {
    /// Caller identity (e.g. DID from ionic token). `None` = anonymous.
    pub identity: Option<String>,
    /// Resource envelope from the ionic token. `None` = no token presented.
    pub envelope: Option<ResourceEnvelope>,
}

impl CallerContext {
    /// Anonymous caller with no token (permissive-mode default).
    pub fn anonymous() -> Self {
        Self::default()
    }

    pub const fn has_envelope(&self) -> bool {
        self.envelope.is_some()
    }
}

/// Pre-dispatch capability gate.
///
/// Sits between request parsing and method routing. In `Permissive` mode
/// every call passes through. In `Enforcing` mode, protected methods
/// require valid credentials and the caller's [`ResourceEnvelope`]
/// constraints are checked.
pub struct MethodGate {
    mode: GateMode,
}

impl MethodGate {
    pub const fn new(mode: GateMode) -> Self {
        Self { mode }
    }

    /// JH-0 default: all calls allowed.
    pub const fn permissive() -> Self {
        Self::new(GateMode::Permissive)
    }

    #[cfg(test)]
    pub fn mode(&self) -> GateMode {
        self.mode
    }

    /// Basic pre-dispatch check (JH-0). Anonymous caller context.
    #[cfg(test)]
    pub fn check(&self, method: &str) -> Result<(), JsonRpcError> {
        self.check_with_context(method, &CallerContext::anonymous())
    }

    /// Full pre-dispatch check with caller context (JH-2).
    ///
    /// In `Enforcing` mode:
    /// - Anonymous callers are rejected on `Protected` methods (UNAUTHORIZED).
    /// - Callers with a token are checked against their envelope's allowlist.
    ///
    /// In `Permissive` mode: always allowed unless the token itself restricts
    /// the method via its allowlist.
    pub fn check_with_context(
        &self,
        method: &str,
        ctx: &CallerContext,
    ) -> Result<(), JsonRpcError> {
        let normalized = normalize_method(method);
        let visibility = classify_method(normalized);

        trace!(
            method = normalized,
            ?visibility,
            mode = ?self.mode,
            has_identity = ctx.identity.is_some(),
            has_envelope = ctx.has_envelope(),
            "method gate check"
        );

        match self.mode {
            GateMode::Permissive => {
                if let Some(ref env) = ctx.envelope
                    && !env.allows_method(normalized)
                {
                    return Err(JsonRpcError {
                        code: PERMISSION_DENIED,
                        message: format!("Token does not permit method: {normalized}"),
                        data: None,
                    });
                }
                Ok(())
            }
            GateMode::Enforcing => match visibility {
                MethodVisibility::Public => Ok(()),
                MethodVisibility::Protected => {
                    if ctx.identity.is_none() {
                        return Err(JsonRpcError {
                            code: UNAUTHORIZED,
                            message: "Authentication required for protected method".to_string(),
                            data: None,
                        });
                    }
                    if let Some(ref env) = ctx.envelope
                        && !env.allows_method(normalized)
                    {
                        return Err(JsonRpcError {
                            code: PERMISSION_DENIED,
                            message: format!("Token does not permit method: {normalized}"),
                            data: None,
                        });
                    }
                    Ok(())
                }
            },
        }
    }
}

/// Classify a method name into its visibility tier.
///
/// Public methods are introspection / health / identity / auth / provenance —
/// always callable. Everything else (AI inference, tool orchestration, context
/// management, provider registration, transport negotiation) is protected.
///
/// Uses **normalized** method names (after stripping `squirrel.` / `mcp.` prefix).
pub fn classify_method(method: &str) -> MethodVisibility {
    match method {
        // Introspection, health probes, capabilities, discovery, lifecycle.status
        "health.check"
        | "health.liveness"
        | "health.readiness"
        | "system.health"
        | "system.status"
        | "system.metrics"
        | "system.ping"
        | "identity.get"
        | "capabilities.list"
        | "capability.list"
        | "primal.capabilities"
        | "capabilities.announce"
        | "capability.announce"
        | "capabilities.discover"
        | "capability.discover"
        | "lifecycle.status"
        | "discovery.peers"
        | "discovery.list" => MethodVisibility::Public,

        // Auth — must be public so callers can check their own status
        m if m.starts_with("auth.") => MethodVisibility::Public,

        // Provenance — read-only introspection
        m if m.starts_with("provenance.") => MethodVisibility::Public,

        // Protected: ai.*, inference.*, tool.*, context.*, graph.*,
        // lifecycle.register, provider.*, btsp.*
        _ => MethodVisibility::Protected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── classify_method tests ──────────────────────────────────────────

    #[test]
    fn health_methods_are_public() {
        assert_eq!(classify_method("health.check"), MethodVisibility::Public);
        assert_eq!(classify_method("health.liveness"), MethodVisibility::Public);
        assert_eq!(
            classify_method("health.readiness"),
            MethodVisibility::Public
        );
    }

    #[test]
    fn system_methods_are_public() {
        assert_eq!(classify_method("system.health"), MethodVisibility::Public);
        assert_eq!(classify_method("system.status"), MethodVisibility::Public);
        assert_eq!(classify_method("system.metrics"), MethodVisibility::Public);
        assert_eq!(classify_method("system.ping"), MethodVisibility::Public);
    }

    #[test]
    fn identity_and_capabilities_are_public() {
        assert_eq!(classify_method("identity.get"), MethodVisibility::Public);
        assert_eq!(
            classify_method("capabilities.list"),
            MethodVisibility::Public
        );
        assert_eq!(classify_method("capability.list"), MethodVisibility::Public);
        assert_eq!(
            classify_method("primal.capabilities"),
            MethodVisibility::Public
        );
    }

    #[test]
    fn capability_announce_discover_are_public() {
        assert_eq!(
            classify_method("capabilities.announce"),
            MethodVisibility::Public
        );
        assert_eq!(
            classify_method("capability.discover"),
            MethodVisibility::Public
        );
    }

    #[test]
    fn lifecycle_status_public_register_protected() {
        assert_eq!(
            classify_method("lifecycle.status"),
            MethodVisibility::Public
        );
        assert_eq!(
            classify_method("lifecycle.register"),
            MethodVisibility::Protected
        );
    }

    #[test]
    fn discovery_methods_are_public() {
        assert_eq!(classify_method("discovery.peers"), MethodVisibility::Public);
        assert_eq!(classify_method("discovery.list"), MethodVisibility::Public);
    }

    #[test]
    fn auth_prefix_is_public() {
        assert_eq!(classify_method("auth.check"), MethodVisibility::Public);
        assert_eq!(classify_method("auth.mode"), MethodVisibility::Public);
        assert_eq!(classify_method("auth.peer_info"), MethodVisibility::Public);
    }

    #[test]
    fn provenance_prefix_is_public() {
        assert_eq!(
            classify_method("provenance.query"),
            MethodVisibility::Public
        );
        assert_eq!(classify_method("provenance.get"), MethodVisibility::Public);
    }

    #[test]
    fn ai_methods_are_protected() {
        assert_eq!(classify_method("ai.query"), MethodVisibility::Protected);
        assert_eq!(classify_method("ai.complete"), MethodVisibility::Protected);
        assert_eq!(classify_method("ai.chat"), MethodVisibility::Protected);
        assert_eq!(
            classify_method("ai.list_providers"),
            MethodVisibility::Protected
        );
    }

    #[test]
    fn inference_methods_are_protected() {
        assert_eq!(
            classify_method("inference.complete"),
            MethodVisibility::Protected
        );
        assert_eq!(
            classify_method("inference.embed"),
            MethodVisibility::Protected
        );
        assert_eq!(
            classify_method("inference.register_provider"),
            MethodVisibility::Protected
        );
    }

    #[test]
    fn tool_and_context_methods_are_protected() {
        assert_eq!(classify_method("tool.execute"), MethodVisibility::Protected);
        assert_eq!(classify_method("tool.list"), MethodVisibility::Protected);
        assert_eq!(
            classify_method("context.create"),
            MethodVisibility::Protected
        );
        assert_eq!(
            classify_method("context.summarize"),
            MethodVisibility::Protected
        );
    }

    #[test]
    fn graph_and_provider_and_btsp_are_protected() {
        assert_eq!(classify_method("graph.parse"), MethodVisibility::Protected);
        assert_eq!(
            classify_method("provider.register"),
            MethodVisibility::Protected
        );
        assert_eq!(
            classify_method("btsp.negotiate"),
            MethodVisibility::Protected
        );
    }

    #[test]
    fn unknown_method_is_protected() {
        assert_eq!(
            classify_method("some.unknown.method"),
            MethodVisibility::Protected
        );
    }

    // ── MethodGate permissive mode tests ───────────────────────────────

    #[test]
    fn permissive_allows_public_methods() {
        let gate = MethodGate::permissive();
        assert!(gate.check("health.check").is_ok());
        assert!(gate.check("identity.get").is_ok());
        assert!(gate.check("capabilities.list").is_ok());
    }

    #[test]
    fn permissive_allows_protected_methods() {
        let gate = MethodGate::permissive();
        assert!(gate.check("ai.query").is_ok());
        assert!(gate.check("inference.complete").is_ok());
        assert!(gate.check("tool.execute").is_ok());
        assert!(gate.check("context.create").is_ok());
    }

    #[test]
    fn permissive_normalizes_prefixed_methods() {
        let gate = MethodGate::permissive();
        assert!(gate.check("squirrel.ai.query").is_ok());
        assert!(gate.check("mcp.tool.execute").is_ok());
        assert!(gate.check("squirrel.health.check").is_ok());
    }

    // ── MethodGate enforcing mode tests ────────────────────────────────

    #[test]
    fn enforcing_allows_public_methods_without_identity() {
        let gate = MethodGate::new(GateMode::Enforcing);
        assert!(gate.check("health.check").is_ok());
        assert!(gate.check("identity.get").is_ok());
        assert!(gate.check("capabilities.list").is_ok());
        assert!(gate.check("auth.mode").is_ok());
        assert!(gate.check("system.ping").is_ok());
    }

    #[test]
    fn enforcing_rejects_protected_methods_without_identity() {
        let gate = MethodGate::new(GateMode::Enforcing);
        let err = gate.check("ai.query").unwrap_err();
        assert_eq!(err.code, UNAUTHORIZED);

        let err = gate.check("inference.complete").unwrap_err();
        assert_eq!(err.code, UNAUTHORIZED);

        let err = gate.check("tool.execute").unwrap_err();
        assert_eq!(err.code, UNAUTHORIZED);
    }

    #[test]
    fn enforcing_allows_protected_with_identity() {
        let gate = MethodGate::new(GateMode::Enforcing);
        let ctx = CallerContext {
            identity: Some("did:eco:test".to_string()),
            envelope: None,
        };
        assert!(gate.check_with_context("ai.query", &ctx).is_ok());
        assert!(gate.check_with_context("inference.complete", &ctx).is_ok());
    }

    // ── ResourceEnvelope tests ─────────────────────────────────────────

    #[test]
    fn envelope_empty_allowlist_permits_all() {
        let env = ResourceEnvelope::default();
        assert!(env.allows_method("ai.query"));
        assert!(env.allows_method("anything"));
    }

    #[test]
    fn envelope_allowlist_restricts_methods() {
        let env = ResourceEnvelope {
            method_allowlist: vec!["ai.query".to_string(), "health.check".to_string()],
            ..Default::default()
        };
        assert!(env.allows_method("ai.query"));
        assert!(env.allows_method("health.check"));
        assert!(!env.allows_method("tool.execute"));
    }

    #[test]
    fn permissive_respects_envelope_allowlist() {
        let gate = MethodGate::permissive();
        let ctx = CallerContext {
            identity: Some("caller".to_string()),
            envelope: Some(ResourceEnvelope {
                method_allowlist: vec!["ai.query".to_string()],
                ..Default::default()
            }),
        };
        assert!(gate.check_with_context("ai.query", &ctx).is_ok());
        let err = gate.check_with_context("tool.execute", &ctx).unwrap_err();
        assert_eq!(err.code, PERMISSION_DENIED);
    }

    #[test]
    fn enforcing_respects_envelope_allowlist() {
        let gate = MethodGate::new(GateMode::Enforcing);
        let ctx = CallerContext {
            identity: Some("did:eco:test".to_string()),
            envelope: Some(ResourceEnvelope {
                method_allowlist: vec!["ai.query".to_string()],
                ..Default::default()
            }),
        };
        assert!(gate.check_with_context("ai.query", &ctx).is_ok());
        let err = gate
            .check_with_context("inference.complete", &ctx)
            .unwrap_err();
        assert_eq!(err.code, PERMISSION_DENIED);
    }

    // ── GateMode construction ──────────────────────────────────────────

    #[test]
    fn gate_reports_mode() {
        assert_eq!(MethodGate::permissive().mode(), GateMode::Permissive);
        assert_eq!(
            MethodGate::new(GateMode::Enforcing).mode(),
            GateMode::Enforcing
        );
    }

    #[test]
    fn caller_context_anonymous_has_no_identity_or_envelope() {
        let ctx = CallerContext::anonymous();
        assert!(ctx.identity.is_none());
        assert!(!ctx.has_envelope());
    }
}
