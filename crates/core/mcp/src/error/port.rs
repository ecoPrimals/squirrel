// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use thiserror::Error;

/// Error kinds for port-related errors.
///
/// These represent different ways in which network port operations can fail,
/// such as when a port is not available, access is denied, or the port is
/// outside the valid range.
#[derive(Debug, Clone, Error)]
pub enum PortErrorKind {
    /// The requested port is not available
    #[error("Port {0} is not available")]
    NotAvailable(u16),

    /// Access to the requested port was denied
    #[error("Access to port {0} was denied")]
    AccessDenied(u16),

    /// The port is outside the valid range
    #[error("Port {0} is outside the valid range")]
    InvalidRange(u16),

    /// The port is already in use
    #[error("Port {0} is already in use")]
    InUse(u16),

    /// A generic port-related error
    #[error("Port error: {0}")]
    Other(String),
}
