// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

#![deny(unsafe_code)]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TaskRequirements {
    pub cpu: Option<f64>,
    pub memory: Option<u64>,
    pub storage: Option<u64>,
    pub network: Option<f64>,
    pub required_capabilities: Vec<String>,
    pub preferred_primals: Vec<PrimalType>,
    pub constraints: std::collections::HashMap<String, String>,
} 