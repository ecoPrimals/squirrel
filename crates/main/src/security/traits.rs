// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Simple Security Coordination Traits
//!
//! Focused on coordinating with `BearDog` for security operations.

use super::types::{SecurityContext, SecurityRequest, SecurityResponse};
use crate::error::PrimalError;

/// Simple Security Coordinator trait (native async - Phase 4 migration)
///
/// Focuses on coordinating with `BearDog` rather than implementing full security system
pub trait SecurityCoordinator: Send + Sync {
    /// Coordinate security request with `BearDog`
    fn coordinate_security(
        &mut self,
        request: SecurityRequest,
    ) -> impl std::future::Future<Output = Result<SecurityResponse, PrimalError>> + Send;

    /// Authenticate via `BearDog`
    fn authenticate_with_beardog(
        &mut self,
        user_id: &str,
    ) -> impl std::future::Future<Output = Result<String, PrimalError>> + Send;

    /// Check if request requires `BearDog` coordination
    fn requires_beardog_security(&self, request_type: &super::types::SecurityRequestType) -> bool;

    /// Get cached security context
    fn get_security_context(&self, session_id: &str) -> Option<&SecurityContext>;

    /// Simple health check
    fn is_healthy(&self) -> bool;
}
