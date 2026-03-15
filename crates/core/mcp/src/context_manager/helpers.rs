// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::types::Context;
use uuid::Uuid;
use tracing::warn;

/// Validates if a rule applies to a context
///
/// This function checks whether a given rule applies to a specific context
/// based on its properties.
///
/// # Arguments
/// * `rule` - The rule string to validate
/// * `context` - The context to validate against
///
/// # Returns
/// `true` if the rule applies to the context, `false` otherwise
pub fn rule_validator(rule: &str, context: &Context) -> bool {
    // This is a placeholder implementation - add real validation logic as needed
    match rule {
        "has_id" => context.id != Uuid::nil(),
        "has_name" => !context.name.is_empty(),
        "has_data" => !context.data.is_null(),
        _ => {
            warn!("Unknown validation rule: {}", rule);
            true // Default to passing for unknown rules
        }
    }
}

/// Helper function to check if an Option is None or its value satisfies a predicate
pub fn is_none_or_matches<T, F>(opt: Option<&T>, predicate: F) -> bool
where
    F: FnOnce(&T) -> bool,
{
    match opt {
        None => true,
        Some(value) => predicate(value),
    }
}
