// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tool capability definitions
//!
//! This module contains capability types for tools.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::parameters::{Parameter, ReturnType};

/// Tool capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability name
    pub name: String,
    /// Capability description
    pub description: String,
    /// Capability parameters
    pub parameters: Vec<Parameter>,
    /// Capability return type
    pub return_type: Option<ReturnType>,
    /// Security level for this capability (0-10, 0 being lowest)
    pub security_level: Option<u8>,
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

