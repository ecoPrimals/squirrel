//! Common trait definitions for security types.
//!
//! This module provides traits for common security concepts like Resources and Actions,
//! which allow for type-safe interactions while avoiding circular dependencies.

use std::fmt::{Debug, Display};
use std::any::Any;
use serde_json::Value;

/// Trait for resources that can be authorized
///
/// Implementing this trait allows a type to be used with the authorization system
/// without creating direct dependencies on the security module.
pub trait ResourceTrait: Debug + Display {
    /// Get the identifier for this resource
    fn id(&self) -> &str;
    
    /// Get optional attributes for this resource
    fn attributes(&self) -> Option<&Value>;
    
    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Trait for actions that can be performed on resources
///
/// Implementing this trait allows a type to be used with the authorization system
/// without creating direct dependencies on the security module.
pub trait ActionTrait: Debug + Display {
    /// Get the string representation of this action
    fn as_ref(&self) -> &str;
    
    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Helper function to create a permission string from an action and resource
pub fn make_permission_string<A: ActionTrait, R: ResourceTrait>(action: &A, resource: &R) -> String {
    format!("{}:{}", action.as_ref(), resource.id())
} 