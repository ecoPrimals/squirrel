//! Simple Security Coordination Traits
//!
//! Focused on coordinating with BearDog for security operations.

use async_trait::async_trait;

use super::{
    health::SecurityHealth,
    types::{SecurityContext, SecurityRequest, SecurityResponse},
};
use crate::error::PrimalError;

/// Simple Security Coordinator trait
///
/// Focuses on coordinating with BearDog rather than implementing full security system
#[async_trait]
pub trait SecurityCoordinator: Send + Sync {
    /// Coordinate security request with BearDog
    async fn coordinate_security(
        &mut self,
        request: SecurityRequest,
    ) -> Result<SecurityResponse, PrimalError>;

    /// Authenticate via BearDog
    async fn authenticate_with_beardog(&mut self, user_id: &str) -> Result<String, PrimalError>;

    /// Check if request requires BearDog coordination
    fn requires_beardog_security(&self, request_type: &super::types::SecurityRequestType) -> bool;

    /// Get cached security context
    fn get_security_context(&self, session_id: &str) -> Option<&SecurityContext>;

    /// Simple health check
    fn is_healthy(&self) -> bool;
}
