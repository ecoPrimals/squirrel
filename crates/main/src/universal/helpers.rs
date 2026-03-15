// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Helper functions for the universal primal system
//!
//! This module provides utility functions for capability validation
//! and other common operations.

use super::types::PrimalCapability;

/// Validate that a provided capability satisfies a required capability
///
/// This function checks if the provided capability meets the requirements
/// of the required capability, accounting for flexible matching (e.g., a
/// primal providing GPT-4 and Claude-3 can satisfy a requirement for GPT-4).
#[must_use]
pub fn validate_capability_compatibility(
    provided: &PrimalCapability,
    required: &PrimalCapability,
) -> bool {
    match (provided, required) {
        (
            PrimalCapability::ModelInference {
                models: provided_models,
            },
            PrimalCapability::ModelInference {
                models: required_models,
            },
        ) => {
            // Check if all required models are in the provided models
            required_models.iter().all(|req_model| {
                provided_models
                    .iter()
                    .any(|prov_model| prov_model == req_model)
            })
        }
        (
            PrimalCapability::ContextManagement {
                max_context_length: provided_length,
            },
            PrimalCapability::ContextManagement {
                max_context_length: required_length,
            },
        ) => provided_length >= required_length,
        (
            PrimalCapability::DataStorage {
                max_size_bytes: provided_size,
                ..
            },
            PrimalCapability::DataStorage {
                max_size_bytes: required_size,
                ..
            },
        ) => provided_size >= required_size,
        (
            PrimalCapability::RateLimiting {
                max_requests_per_second: provided_rate,
            },
            PrimalCapability::RateLimiting {
                max_requests_per_second: required_rate,
            },
        ) => provided_rate >= required_rate,
        // For other capabilities, check if they are the same variant
        (provided, required) => {
            std::mem::discriminant(provided) == std::mem::discriminant(required)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_validation() {
        let provided = PrimalCapability::ModelInference {
            models: vec!["gpt-4".to_string(), "claude-3".to_string()],
        };

        let required = PrimalCapability::ModelInference {
            models: vec!["gpt-4".to_string()],
        };

        assert!(validate_capability_compatibility(&provided, &required));

        let required = PrimalCapability::ModelInference {
            models: vec!["gpt-5".to_string()],
        };

        assert!(!validate_capability_compatibility(&provided, &required));
    }

    #[test]
    fn test_context_length_validation() {
        let provided = PrimalCapability::ContextManagement {
            max_context_length: 8192,
        };

        let required = PrimalCapability::ContextManagement {
            max_context_length: 4096,
        };

        assert!(validate_capability_compatibility(&provided, &required));

        let required = PrimalCapability::ContextManagement {
            max_context_length: 16384,
        };

        assert!(!validate_capability_compatibility(&provided, &required));
    }
}
