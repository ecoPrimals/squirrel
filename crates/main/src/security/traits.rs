// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Simple Security Coordination Traits
//!
//! Focuses on coordinating with security primals via capability-based discovery.

use super::types::{SecurityContext, SecurityRequest, SecurityResponse};
use crate::error::PrimalError;

/// Security coordination trait (native async - Phase 4 migration)
///
/// Delegates security operations to discovered security primals.
pub trait SecurityCoordinatorTrait: Send + Sync {
    /// Coordinate security request with the security primal
    fn coordinate_security(
        &mut self,
        request: SecurityRequest,
    ) -> impl std::future::Future<Output = Result<SecurityResponse, PrimalError>> + Send;

    /// Authenticate via security primal delegation
    fn authenticate_with_security_provider(
        &mut self,
        user_id: &str,
    ) -> impl std::future::Future<Output = Result<String, PrimalError>> + Send;

    /// Check if request requires security primal coordination
    fn requires_security_delegation(
        &self,
        request_type: &super::types::SecurityRequestType,
    ) -> bool;

    /// Get cached security context
    fn get_security_context(&self, session_id: &str) -> Option<&SecurityContext>;

    /// Simple health check
    fn is_healthy(&self) -> bool;
}
