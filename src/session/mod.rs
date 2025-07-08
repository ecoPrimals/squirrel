impl SessionManager {
    /// Create a new session manager with the given configuration
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            cleanup_task: None,
        }
    }
    
    /// Get the number of active sessions (for testing)
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }
    
    /// Check if sessions are empty (for testing)
    pub async fn is_empty(&self) -> bool {
        self.sessions.read().await.is_empty()
    }
} 