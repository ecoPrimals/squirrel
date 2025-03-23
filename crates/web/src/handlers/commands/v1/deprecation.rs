//! # Legacy Commands API Deprecation Plan
//! 
//! This module provides documentation and guidance for the planned deprecation
//! of the legacy Commands API endpoints. The transition to the new Command Execution
//! API will occur gradually to ensure minimal disruption to existing clients.
//!
//! ## Deprecation Timeline
//!
//! | Date | Milestone |
//! |------|-----------|
//! | Current | Legacy endpoints remain fully functional alongside new API |
//! | 3 months from now | Deprecation warnings added to legacy endpoint responses |
//! | 6 months from now | Legacy endpoints enter maintenance mode (no new features) |
//! | 12 months from now | Legacy endpoints return 410 Gone status with migration info |
//!
//! ## Migration Path
//!
//! Clients should transition from:
//! - `/api/commands-legacy/*` endpoints
//!
//! To the new:
//! - `/api/commands/*` endpoints
//!
//! ## API Differences
//!
//! The new Command Execution API offers several improvements:
//!
//! 1. Consistent error handling with standardized error codes
//! 2. Improved pagination support for command history
//! 3. Enhanced security model with explicit user context
//! 4. Better performance through optimized database access
//! 5. More detailed command execution status tracking
//!
//! ## Implementation Strategy
//!
//! During the transition period:
//! 
//! 1. Both API versions will coexist, with the legacy API delegating to the new implementation
//! 2. API clients should be updated to use the new endpoints
//! 3. Monitoring will track usage of legacy endpoints to inform deprecation timing
//! 4. Documentation will clearly mark legacy endpoints as deprecated
//!
//! ## Example Migration
//!
//! ### Legacy Command Execution
//! ```
//! POST /api/commands-legacy/execute
//! {
//!     "name": "some-command",
//!     "parameters": { "key": "value" }
//! }
//! ```
//!
//! ### New Command Execution
//! ```
//! POST /api/commands/execute
//! {
//!     "command": "some-command",
//!     "params": { "key": "value" },
//!     "context": {
//!         "user_id": "user-123"
//!     }
//! }
//! ```
//!
//! ## Backwards Compatibility
//!
//! The internal implementation of `MockCommandService` has been updated to delegate
//! to the new `CommandService` implementation, ensuring consistent behavior between
//! legacy and new endpoints during the transition period.

/// Internal marker trait to tag code that supports legacy API endpoints
/// that are scheduled for deprecation.
///
/// Any code implementing this trait should be considered temporary and 
/// will be removed when the legacy API is fully deprecated.
pub trait LegacyDeprecated {
    /// Returns the expected removal date for this legacy functionality
    fn removal_date(&self) -> &'static str {
        "12 months from implementation date"
    }
    
    /// Returns the recommended migration path
    fn migration_path(&self) -> &'static str;
}

/// Apply deprecation header to API responses from legacy endpoints
pub fn add_deprecation_headers(response: &mut axum::response::Response) {
    use axum::http::header::{SUNSET, WARNING};
    
    // RFC 8594 Sunset header format
    response.headers_mut().insert(
        SUNSET, 
        format!(
            "{}",
            // Set the sunset date 12 months from now
            chrono::Utc::now()
                .checked_add_months(chrono::Months::new(12))
                .unwrap_or_else(|| chrono::Utc::now())
                .format("%a, %d %b %Y %H:%M:%S GMT")
        ).parse().unwrap()
    );
    
    // Warning header per RFC 7234
    response.headers_mut().insert(
        WARNING,
        "299 - \"Deprecated API. See documentation for migration guidance.\"".parse().unwrap()
    );
} 