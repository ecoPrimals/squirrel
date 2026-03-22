// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Thread-safe [`MCPProtocolAdapter`] and factory helpers.
//!
//! Implementation lives in [`core`]; integration tests in [`tests`].

mod core;
#[cfg(test)]
mod tests;

pub use core::{
    create_initialized_protocol_adapter, create_protocol_adapter, create_protocol_adapter_with_config,
    create_protocol_adapter_with_protocol, MCPProtocolAdapter,
};
