// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Universal pattern mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! # G68 Platform Substrate Abstraction
//!
//! Cross-platform abstractions for filesystem operations that differ between
//! Unix and Windows. Replaces raw `#[cfg(unix)]` blocks scattered across the
//! codebase with a single abstraction layer.
//!
//! ## Layers
//!
//! | Layer | Abstraction | Unix | Windows |
//! |-------|-------------|------|---------|
//! | L1 | [`create_capability_alias`] | `symlink()` | Discovery file |
//! | L2 | [`set_access`] / [`check_world_accessible`] | `chmod` mode bits | `readonly` + ACL (future) |

mod access;
mod link;

pub use access::{AccessLevel, check_world_accessible, set_access, set_access_async};
pub use link::{cleanup_capability_alias, create_capability_alias};
