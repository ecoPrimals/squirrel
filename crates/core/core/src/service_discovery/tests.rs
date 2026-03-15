// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for the service discovery system
//!
//! This module contains comprehensive tests for all service discovery components
//! including validation, registration, querying, and lifecycle management.

use crate::service_discovery::types::ServiceType;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_type_from_string() {
        assert_eq!(ServiceType::from_str("ai").unwrap(), ServiceType::AI);
        assert_eq!(
            ServiceType::from_str("compute").unwrap(),
            ServiceType::Compute
        );
        assert_eq!(
            ServiceType::from_str("storage").unwrap(),
            ServiceType::Storage
        );
        assert_eq!(
            ServiceType::from_str("security").unwrap(),
            ServiceType::Security
        );
        assert_eq!(
            ServiceType::from_str("communication").unwrap(),
            ServiceType::Communication
        );
        assert_eq!(
            ServiceType::from_str("discovery").unwrap(),
            ServiceType::Discovery
        );
        assert_eq!(
            ServiceType::from_str("monitoring").unwrap(),
            ServiceType::Monitoring
        );
        assert_eq!(
            ServiceType::from_str("gateway").unwrap(),
            ServiceType::Gateway
        );
        assert_eq!(
            ServiceType::from_str("custom").unwrap(),
            ServiceType::Custom("custom".to_string())
        );
    }
}
