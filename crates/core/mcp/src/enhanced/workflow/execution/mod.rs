// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Workflow Execution Engine Submodules
//!
//! This module contains submodules for the workflow execution engine:
//! - `handlers`: Step execution handlers for different step types
//! - `resolver`: Variable resolution and input substitution
//! - `condition`: Condition evaluation logic

pub mod handlers;
pub mod resolver;
pub mod condition;
