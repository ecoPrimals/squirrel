// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Error handling for AI tools
//!
//! All error types delegate to `universal_error::tools::AIToolsError`.
//!
//! ```ignore
//! use squirrel_ai_tools::error::{Error, Result};
//! // or directly:
//! use universal_error::tools::AIToolsError;
//! ```

/// Result type alias for AI tools operations.
pub type Result<T> = std::result::Result<T, universal_error::tools::AIToolsError>;

/// Error type alias for convenience.
pub type Error = universal_error::tools::AIToolsError;
