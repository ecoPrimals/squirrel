// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Orchestration primitives for multi-primal composition.
//!
//! Absorbs the `DeploymentGraphDef` pattern from ludoSpring exp054 so
//! Squirrel can participate in graph-aware multi-node workflows.

pub mod deploy_graph;

pub use deploy_graph::{DeploymentGraphDef, GraphNode, TickConfig};
