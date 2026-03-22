// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! # Production Rate Limiting & `DoS` Protection
#![expect(dead_code, reason = "Rate limiting infrastructure awaiting activation")]
//!
//! This module provides comprehensive rate limiting to protect against:
//! - Denial of Service (`DoS`) attacks
//! - API abuse and excessive requests
//! - Resource exhaustion attacks
//! - Brute force authentication attempts

mod bucket;
pub mod config;
mod production;
pub mod types;

#[cfg(test)]
mod tests;

pub use config::RateLimitConfig;
pub use production::ProductionRateLimiter;
pub use types::{ClientRequestCounter, EndpointType, RateLimitResult, RateLimitStatistics};
