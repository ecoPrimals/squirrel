// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! AI Subsystem
//!
//! Central AI capabilities including routing and providers.
//!
//! Model splitting has been fully relocated to ToadStool (GPU/VRAM) and
//! Songbird (cross-tower coordination). Squirrel discovers those primals
//! at runtime via capability-based IPC.
pub mod model_splitting;
