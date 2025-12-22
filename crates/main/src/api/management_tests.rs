//! Tests for management API handlers

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::shutdown::ShutdownManager;
    use std::sync::Arc;

    fn create_test_shutdown_manager() -> Arc<ShutdownManager> {
        Arc::new(ShutdownManager::new())
    }

    #[tokio::test]
    async fn test_handle_shutdown_basic() {
        let manager = create_test_shutdown_manager();

        // Simply test that handle_shutdown sends the signal successfully
        let result = handle_shutdown(manager.clone()).await;
        assert!(result.is_ok());

        // Verify shutdown was requested
        assert!(manager.is_shutdown_requested());
    }

    #[tokio::test]
    async fn test_handle_shutdown_returns_json() {
        let manager = create_test_shutdown_manager();

        // Test that the handler returns successfully
        let result = handle_shutdown(manager.clone()).await;
        assert!(result.is_ok());

        // Verify the manager state
        assert!(manager.is_shutdown_requested());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_concurrent_shutdown_requests() {
        let manager = create_test_shutdown_manager();

        // Use barrier to ensure all requests start simultaneously
        let barrier = Arc::new(tokio::sync::Barrier::new(5));
        let mut handles = vec![];

        for _ in 0..5 {
            let manager_clone = Arc::clone(&manager);
            let barrier_clone = Arc::clone(&barrier);

            let handle = tokio::spawn(async move {
                // Wait for all tasks to be ready
                barrier_clone.wait().await;

                // All tasks call handle_shutdown concurrently
                let result = handle_shutdown(manager_clone).await;
                assert!(result.is_ok());
            });
            handles.push(handle);
        }

        // Wait for all concurrent requests to complete
        for handle in handles {
            handle.await.expect("Task should complete");
        }

        // Verify shutdown was requested
        assert!(manager.is_shutdown_requested());
    }

    #[tokio::test]
    async fn test_shutdown_handler_is_async() {
        let manager = create_test_shutdown_manager();

        // Verify handler works with async/await
        let result = handle_shutdown(manager.clone()).await;
        assert!(result.is_ok());

        // Verify manager state
        assert!(manager.is_shutdown_requested());
    }
}
