// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

mod config_tests;
mod connection_tests;
mod frame_tests;
mod transport_tests;
mod integration_tests;

// Re-export test modules
pub use config_tests::*;
pub use connection_tests::*;
pub use frame_tests::*;
pub use transport_tests::*;
pub use integration_tests::*; 