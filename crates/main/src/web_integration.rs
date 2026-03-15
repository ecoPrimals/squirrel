// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use crate::session::{SessionConfig, SessionManagerImpl};
use std::sync::Arc;

/// Web integration module for squirrel
pub struct WebIntegration {
    pub session_manager: Arc<SessionManagerImpl>,
}

impl WebIntegration {
    /// Create a new web integration with default session management
    ///
    /// Initializes a new web integration instance with a default session manager
    /// configured for web-based interactions and session handling.
    pub fn new() -> Self {
        let session_manager = Arc::new(SessionManagerImpl::new(SessionConfig::default()));
        Self { session_manager }
    }
}

impl Default for WebIntegration {
    fn default() -> Self {
        Self::new()
    }
}
