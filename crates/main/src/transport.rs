// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Transport abstraction — re-exports from `universal_patterns::transport`.
//!
//! `TransportEndpoint`, `TransportStream`, and `connect_transport*` live in
//! `universal-patterns` so every workspace crate can use them without depending
//! on `squirrel` (the main crate).

pub use universal_patterns::transport::{
    TransportEndpoint, TransportStream, connect_transport, connect_transport_with_timeout,
};
