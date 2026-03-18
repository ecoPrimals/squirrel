// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Canonical primal identifiers and display names.
//!
//! Machine identifiers (lowercase, kebab-safe) live at the top level.
//! Human-readable display names for UI and logging live in [`display`].
//!
//! Follows the neuralSpring V118 display-name pattern.

/// Squirrel — Universal AI Primal
pub const SQUIRREL: &str = "squirrel";
/// `BearDog` — Cryptographic identity and trust
pub const BEARDOG: &str = "beardog";
/// Songbird — Service discovery and IPC mesh
pub const SONGBIRD: &str = "songbird";
/// `ToadStool` — GPU compute dispatch
pub const TOADSTOOL: &str = "toadstool";
/// `NestGate` — Storage and model cache
pub const NESTGATE: &str = "nestgate";
/// biomeOS — Operating system / orchestrator
pub const BIOMEOS: &str = "biomeos";
/// coralReef — Distributed state
pub const CORALREEF: &str = "coralreef";
/// barraCuda — GPU runtime
pub const BARRACUDA: &str = "barracuda";
/// rhizoCrypt — DAG provenance
pub const RHIZOCRYPT: &str = "rhizocrypt";
/// sweetGrass — Attribution and anchoring
pub const SWEETGRASS: &str = "sweetgrass";
/// loamSpine — Infrastructure backbone
pub const LOAMSPINE: &str = "loamspine";
/// petalTongue — Natural language interface
pub const PETALTONGUE: &str = "petaltongue";
/// skunkBat — Adversarial testing / chaos
pub const SKUNKBAT: &str = "skunkbat";

/// Human-readable display names for UI, logging, and error messages.
///
/// These preserve the original mixed-case branding of each primal.
pub mod display {
    /// Squirrel display name
    pub const SQUIRREL: &str = "Squirrel";
    /// `BearDog` display name
    pub const BEARDOG: &str = "BearDog";
    /// Songbird display name
    pub const SONGBIRD: &str = "Songbird";
    /// `ToadStool` display name
    pub const TOADSTOOL: &str = "ToadStool";
    /// `NestGate` display name
    pub const NESTGATE: &str = "NestGate";
    /// biomeOS display name
    pub const BIOMEOS: &str = "biomeOS";
    /// coralReef display name
    pub const CORALREEF: &str = "coralReef";
    /// barraCuda display name
    pub const BARRACUDA: &str = "barraCuda";
    /// rhizoCrypt display name
    pub const RHIZOCRYPT: &str = "rhizoCrypt";
    /// sweetGrass display name
    pub const SWEETGRASS: &str = "sweetGrass";
    /// loamSpine display name
    pub const LOAMSPINE: &str = "loamSpine";
    /// petalTongue display name
    pub const PETALTONGUE: &str = "petalTongue";
    /// skunkBat display name
    pub const SKUNKBAT: &str = "skunkBat";
}

/// Look up the display name for a machine identifier.
///
/// Returns the input unchanged if it is not a known primal.
#[must_use]
pub fn display_name(machine_id: &str) -> &str {
    match machine_id {
        SQUIRREL => display::SQUIRREL,
        BEARDOG => display::BEARDOG,
        SONGBIRD => display::SONGBIRD,
        TOADSTOOL => display::TOADSTOOL,
        NESTGATE => display::NESTGATE,
        BIOMEOS => display::BIOMEOS,
        CORALREEF => display::CORALREEF,
        BARRACUDA => display::BARRACUDA,
        RHIZOCRYPT => display::RHIZOCRYPT,
        SWEETGRASS => display::SWEETGRASS,
        LOAMSPINE => display::LOAMSPINE,
        PETALTONGUE => display::PETALTONGUE,
        SKUNKBAT => display::SKUNKBAT,
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_primals_have_display_names() {
        assert_eq!(display_name(SQUIRREL), "Squirrel");
        assert_eq!(display_name(BEARDOG), "BearDog");
        assert_eq!(display_name(BIOMEOS), "biomeOS");
        assert_eq!(display_name(BARRACUDA), "barraCuda");
    }

    #[test]
    fn unknown_id_returns_itself() {
        assert_eq!(display_name("unknown-primal"), "unknown-primal");
    }

    #[test]
    fn machine_ids_are_lowercase() {
        let ids = [
            SQUIRREL,
            BEARDOG,
            SONGBIRD,
            TOADSTOOL,
            NESTGATE,
            BIOMEOS,
            CORALREEF,
            BARRACUDA,
            RHIZOCRYPT,
            SWEETGRASS,
            LOAMSPINE,
            PETALTONGUE,
            SKUNKBAT,
        ];
        for id in ids {
            assert_eq!(id, id.to_lowercase(), "{id} should be lowercase");
        }
    }
}
