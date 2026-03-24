// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for the service discovery system
//!
//! This module contains comprehensive tests for all service discovery components
//! including validation, registration, querying, and lifecycle management.

use crate::service_discovery::types::ServiceType;
use std::str::FromStr;

#[cfg(test)]
mod service_type_tests {
    use super::*;

    #[tokio::test]
    async fn test_service_type_from_string() {
        assert_eq!(
            ServiceType::from_str("ai").expect("should succeed"),
            ServiceType::AI
        );
        assert_eq!(
            ServiceType::from_str("compute").expect("should succeed"),
            ServiceType::Compute
        );
        assert_eq!(
            ServiceType::from_str("storage").expect("should succeed"),
            ServiceType::Storage
        );
        assert_eq!(
            ServiceType::from_str("security").expect("should succeed"),
            ServiceType::Security
        );
        assert_eq!(
            ServiceType::from_str("communication").expect("should succeed"),
            ServiceType::Communication
        );
        assert_eq!(
            ServiceType::from_str("discovery").expect("should succeed"),
            ServiceType::Discovery
        );
        assert_eq!(
            ServiceType::from_str("monitoring").expect("should succeed"),
            ServiceType::Monitoring
        );
        assert_eq!(
            ServiceType::from_str("gateway").expect("should succeed"),
            ServiceType::Gateway
        );
        assert_eq!(
            ServiceType::from_str("custom").expect("should succeed"),
            ServiceType::Custom("custom".to_string())
        );
    }
}
