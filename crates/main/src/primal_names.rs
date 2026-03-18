// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Centralized primal name hints for socket discovery.
//!
//! Follows the groundSpring V106 / wetSpring V119 / airSpring v0.8.2 pattern:
//! every primal name used in socket path construction or display-string mapping
//! lives here as a typed constant. Runtime discovery uses capabilities, not
//! names — these hints are only for socket file naming conventions and logging.
//!
//! ## TRUE PRIMAL principle
//!
//! Squirrel discovers other primals by **capability**, never by name.
//! These constants exist solely so that:
//! - socket filenames follow the ecosystem `{primal}.sock` convention
//! - display / logging messages are consistent
//! - legacy type-mapping code has a single source of truth
//!
//! Actual routing goes through Songbird (`discovery.find_primals`) or
//! biomeOS (`capability.call`).

/// Self — Squirrel's own primal identifier.
pub const SQUIRREL: &str = "squirrel";

/// biomeOS orchestrator (Phase 2 primal).
pub const BIOMEOS: &str = "biomeos";

/// Songbird service-mesh (Phase 1 primal).
pub const SONGBIRD: &str = "songbird";

/// BearDog cryptographic identity (Phase 1 primal).
pub const BEARDOG: &str = "beardog";

/// NestGate storage primal (Phase 1 primal).
pub const NESTGATE: &str = "nestgate";

/// ToadStool compute primal (Phase 1 primal).
pub const TOADSTOOL: &str = "toadstool";

/// coralReef GPU compiler primal (root-level primal).
pub const CORALREEF: &str = "coralreef";

/// barraCuda GPU math primal (root-level primal).
pub const BARRACUDA: &str = "barracuda";

/// rhizoCrypt cryptographic storage primal (Phase 2 primal).
pub const RHIZOCRYPT: &str = "rhizocrypt";

/// petalTongue natural language primal (Phase 2 primal).
pub const PETALTONGUE: &str = "petaltongue";

/// sweetGrass environmental sensing primal (Phase 2 primal).
pub const SWEETGRASS: &str = "sweetgrass";

/// loamSpine provenance primal (Phase 2 primal).
pub const LOAMSPINE: &str = "loamspine";

/// skunkBat anomaly detection primal (Phase 2 primal).
pub const SKUNKBAT: &str = "skunkbat";

// -- Socket filename conventions --------------------------------------------------

/// Standard biomeOS socket subdirectory under XDG_RUNTIME_DIR.
pub const BIOMEOS_SOCKET_DIR: &str = "biomeos";

/// Default Songbird socket filename.
pub const SONGBIRD_SOCKET_NAME: &str = "songbird-default.sock";

/// Default biomeOS socket filename.
pub const BIOMEOS_SOCKET_NAME: &str = "biomeos.sock";

/// Default Neural API socket filename.
pub const NEURAL_API_SOCKET_NAME: &str = "neural-api.sock";

// -- Capability domains consumed by Squirrel --------------------------------------
// These are the capability domains Squirrel looks for during discovery.
// They do NOT name a specific primal — any primal exposing the capability
// can satisfy the dependency.

/// Crypto/auth capability domain (typically BearDog).
pub const CAP_DOMAIN_CRYPTO: &str = "crypto";

/// Compute capability domain (typically ToadStool).
pub const CAP_DOMAIN_COMPUTE: &str = "compute";

/// Storage capability domain (typically NestGate).
pub const CAP_DOMAIN_STORAGE: &str = "storage";

/// Discovery capability domain (typically Songbird).
pub const CAP_DOMAIN_DISCOVERY: &str = "discovery";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_identity_matches_niche() {
        assert_eq!(SQUIRREL, crate::niche::PRIMAL_ID);
    }

    #[test]
    fn all_names_are_lowercase_ascii() {
        for name in [
            SQUIRREL,
            BIOMEOS,
            SONGBIRD,
            BEARDOG,
            NESTGATE,
            TOADSTOOL,
            CORALREEF,
            BARRACUDA,
            RHIZOCRYPT,
            PETALTONGUE,
            SWEETGRASS,
            LOAMSPINE,
            SKUNKBAT,
        ] {
            assert!(
                name.chars().all(|c| c.is_ascii_lowercase()),
                "{name} should be lowercase ASCII"
            );
        }
    }

    #[test]
    fn socket_names_end_with_sock() {
        for name in [
            SONGBIRD_SOCKET_NAME,
            BIOMEOS_SOCKET_NAME,
            NEURAL_API_SOCKET_NAME,
        ] {
            assert!(
                std::path::Path::new(name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("sock")),
                "{name} should end with .sock"
            );
        }
    }
}
