// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! External primal env vars

/// `BearDog` endpoint
pub const BEARDOG_ENDPOINT: &str = "BEARDOG_ENDPOINT";
/// `BearDog` socket
pub const BEARDOG_SOCKET: &str = "BEARDOG_SOCKET";
/// `BearDog` family seed
pub const BEARDOG_FAMILY_SEED: &str = "BEARDOG_FAMILY_SEED";
/// `Songbird` endpoint
pub const SONGBIRD_ENDPOINT: &str = "SONGBIRD_ENDPOINT";
/// `Songbird` port
pub const SONGBIRD_PORT: &str = "SONGBIRD_PORT";
/// `Songbird` socket
pub const SONGBIRD_SOCKET: &str = "SONGBIRD_SOCKET";
/// `Songbird` auth token
pub const SONGBIRD_AUTH_TOKEN: &str = "SONGBIRD_AUTH_TOKEN";
/// `Songbird` batch size
pub const SONGBIRD_BATCH_SIZE: &str = "SONGBIRD_BATCH_SIZE";
/// `Songbird` flush interval
pub const SONGBIRD_FLUSH_INTERVAL: &str = "SONGBIRD_FLUSH_INTERVAL";
/// `Songbird` federation enabled (canonical; renamed from `SONGBIRD_MESH_ENABLED` in biomeOS v4.03)
pub const SONGBIRD_FEDERATION_ENABLED: &str = "SONGBIRD_FEDERATION_ENABLED";
/// `Songbird` federation TCP port (default 7700)
pub const SONGBIRD_FEDERATION_PORT: &str = "SONGBIRD_FEDERATION_PORT";
/// `Songbird` federation bind address
pub const SONGBIRD_FEDERATION_BIND: &str = "SONGBIRD_FEDERATION_BIND";
/// `Songbird` peers list (comma-separated host:port for mesh seeding)
pub const SONGBIRD_PEERS: &str = "SONGBIRD_PEERS";
/// `Songbird` service config path (file-based service discovery override)
pub const SONGBIRD_SERVICE_CONFIG_PATH: &str = "SONGBIRD_SERVICE_CONFIG_PATH";
/// `NestGate` endpoint
pub const NESTGATE_ENDPOINT: &str = "NESTGATE_ENDPOINT";
/// `NestGate` port
pub const NESTGATE_PORT: &str = "NESTGATE_PORT";
/// `ToadStool` endpoint
pub const TOADSTOOL_ENDPOINT: &str = "TOADSTOOL_ENDPOINT";
/// `ToadStool` port
pub const TOADSTOOL_PORT: &str = "TOADSTOOL_PORT";
/// Crypto endpoint
pub const CRYPTO_ENDPOINT: &str = "CRYPTO_ENDPOINT";
/// Crypto signing endpoint
pub const CRYPTO_SIGNING_ENDPOINT: &str = "CRYPTO_SIGNING_ENDPOINT";
