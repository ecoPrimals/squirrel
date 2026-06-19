// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Security env vars

/// Security endpoint
pub const ENDPOINT: &str = "SECURITY_ENDPOINT";
/// Security host
pub const HOST: &str = "SECURITY_HOST";
/// Security port
pub const PORT: &str = "SECURITY_PORT";
/// Security socket
pub const SOCKET: &str = "SECURITY_SOCKET";
/// Security service endpoint
pub const SERVICE_ENDPOINT: &str = "SECURITY_SERVICE_ENDPOINT";
/// Security service host
pub const SERVICE_HOST: &str = "SECURITY_SERVICE_HOST";
/// Security service name
pub const SERVICE_NAME: &str = "SECURITY_SERVICE_NAME";
/// Security auth service endpoint
pub const AUTH_SERVICE_ENDPOINT: &str = "SECURITY_AUTH_SERVICE_ENDPOINT";
/// Security authentication port
pub const AUTHENTICATION_PORT: &str = "SECURITY_AUTHENTICATION_PORT";
/// Security token file
pub const TOKEN_FILE: &str = "SECURITY_TOKEN_FILE";
/// Security trust domain
pub const TRUST_DOMAIN: &str = "SECURITY_TRUST_DOMAIN";
/// JWT secret
pub const JWT_SECRET: &str = "JWT_SECRET";
/// JWT key ID
pub const JWT_KEY_ID: &str = "JWT_KEY_ID";
/// JWT expiry (hours)
pub const JWT_EXPIRY_HOURS: &str = "JWT_EXPIRY_HOURS";
/// TLS cert path
pub const TLS_CERT_PATH: &str = "TLS_CERT_PATH";
/// TLS key path
pub const TLS_KEY_PATH: &str = "TLS_KEY_PATH";
/// CA cert path
pub const CA_CERT_PATH: &str = "CA_CERT_PATH";
/// Family seed for BTSP key derivation
pub const FAMILY_SEED: &str = "FAMILY_SEED";
