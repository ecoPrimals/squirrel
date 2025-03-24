// Tool lifecycle module
//
// This module provides implementations of tool lifecycle hooks and managers.

mod state_validator;


pub use state_validator::{
    StateTransitionGraph, StateTransitionValidator, StateTransitionViolation, StateValidationHook,
    StateRollbackAttempt
};

// Re-export types from the lifecycle_original module
pub use crate::tool::lifecycle_original::{
    BasicLifecycleHook, CompositeLifecycleHook, LifecycleEvent, LifecycleRecord,
    RecoveryAction, RecoveryStrategy, SecurityLifecycleHook,
}; 