use crate::session::{SessionConfig, SessionManagerImpl};
use std::sync::Arc;

/// Web integration module for squirrel
pub struct WebIntegration {
    pub session_manager: Arc<SessionManagerImpl>,
}

impl WebIntegration {
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
