#[tokio::test]
async fn test_session_manager_creation() {
    // Test 15: Session manager
    let config = SessionConfig::default();
    let manager = SessionManager::new(config);
    assert!(manager.is_empty().await);
} 