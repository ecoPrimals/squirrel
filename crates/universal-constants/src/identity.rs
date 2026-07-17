// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Primal identity constants for JWT and auth.
//!
//! Per ecoPrimals wateringHole standards, primals have self-knowledge only.
//! These constants provide the single source of truth for Squirrel's identity
//! in JWT claims, validation, and crypto provider key lookups.
//!
//! Other primals are discovered at runtime via capability-based discovery.

/// Primal identity — Squirrel's own identifier (matches `niche::PRIMAL_ID`).
pub const PRIMAL_ID: &str = "squirrel";

/// JWT issuer claim — identifies Squirrel MCP as the token issuer.
pub const JWT_ISSUER: &str = "squirrel-mcp";

/// JWT audience claim — identifies Squirrel MCP API as the token consumer.
pub const JWT_AUDIENCE: &str = "squirrel-mcp-api";

/// Primary capability domain for biomeOS Neural API registration.
///
/// Matches `niche::DOMAIN`. Used by biomeOS for domain-scoped routing.
pub const PRIMAL_DOMAIN: &str = "ai";

/// Default JWT signing key ID in the crypto capability provider.
///
/// Overridable via `JWT_KEY_ID` environment variable at runtime.
pub const JWT_SIGNING_KEY_ID: &str = "squirrel-jwt-signing-key";

/// Bootstrap admin username for standalone dev mode (intentionally weak).
pub const BOOTSTRAP_ADMIN_USER: &str = "admin";

/// Bootstrap admin email for standalone dev mode.
///
/// Not used for actual email delivery — only as a placeholder identity
/// in the seeded dev user.
pub const BOOTSTRAP_ADMIN_EMAIL: &str = "admin@localhost";

/// Default ecosystem router IPC service identifier.
///
/// Used when resolving the IPC endpoint for ecosystem routing (biomeOS Neural
/// API). Overridable via `SQUIRREL_ECOSYSTEM_IPC_SERVICE` or
/// `ECOSYSTEM_ROUTER_SERVICE_ID` environment variables.
pub const DEFAULT_ECOSYSTEM_IPC_SERVICE: &str = "nat0";
