//! Batch operations for the Context-MCP adapter
//!
//! This module contains the batch processing functionality for the Context-MCP adapter,
//! including methods for processing multiple contexts in parallel and model evaluation.

use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{error, info};
use futures::future;

use crate::context_mcp::errors::Result;
use crate::context_mcp::errors::ContextMcpError;
use crate::context_mcp::ContextMcpAdapter;
use crate::context_mcp::config::ContextAiEnhancementOptions;
use serde_json::Value;

/// Find contexts by matching tags
///
/// This function retrieves contexts from MCP that match the given tags.
/// It can match contexts that have either any of the provided tags or all of them.
///
/// # Parameters
/// * `tags` - A slice of tag strings to match against
/// * `match_all` - If true, contexts must have all tags to match; otherwise, any tag will match
///
/// # Returns
/// * `Result<Vec<Value>>` - A list of context values that match the criteria
pub async fn find_contexts_by_tags(_tags: &[String], _match_all: bool) -> Result<Vec<Value>> {
    // Mock implementation that returns an empty vec
    // In a real implementation, this would call the MCP adapter
    Ok(Vec::new())
}

// Implementation of batch processing helpers
impl ContextMcpAdapter {
    /// Helper function for error handling in batch operations
    fn handle_batch_error(&self, context_id: &str, error: ContextMcpError) -> ContextMcpError {
        error!("Error processing context {}: {:?}", context_id, error);
        error
    }
    
    /// Batch enhance multiple contexts with AI
    ///
    /// This method allows for efficiently enhancing multiple contexts with the same AI enhancement options.
    /// It processes contexts in parallel for better throughput.
    ///
    /// # Parameters
    /// * `context_ids` - A vector of context IDs to enhance
    /// * `options` - The AI enhancement options to apply to all contexts
    /// * `max_concurrent` - Maximum number of concurrent enhancements (default: 5)
    ///
    /// # Returns
    /// * `Ok(Vec<(String, Result<()>)>)` - A vector of tuples with context ID and result
    /// * `Err(ContextMcpError)` - If there was an error in the batch operation itself
    pub async fn batch_enhance_contexts(
        &self,
        context_ids: Vec<String>,
        options: ContextAiEnhancementOptions,
        max_concurrent: Option<usize>,
    ) -> Result<Vec<(String, Result<()>)>> {
        info!("Batch enhancing {} contexts", context_ids.len());
        
        // Determine max concurrent operations
        let max_concurrent = max_concurrent.unwrap_or(5);
        
        // Create a semaphore to limit concurrency
        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        
        // Process contexts in parallel with limited concurrency
        let futures = context_ids.into_iter().map(|context_id| {
            let adapter = self.clone();
            let options = options.clone();
            let semaphore = semaphore.clone();
            let context_id_clone = context_id.clone();
            
            async move {
                let _permit = semaphore.acquire().await.unwrap();
                let result = adapter.apply_ai_enhancements(&context_id_clone, options).await;
                (context_id_clone, result)
            }
        });
        
        // Wait for all enhancements to complete
        let results = future::join_all(futures).await;
        
        info!("Batch enhancement completed for {} contexts", results.len());
        Ok(results)
    }
    
    /// Batch enhance multiple contexts with the same type of enhancement
    ///
    /// Convenience method for batch enhancing multiple contexts with the same
    /// enhancement type, provider, and API key.
    ///
    /// # Parameters
    /// * `context_ids` - A vector of context IDs to enhance
    /// * `enhancement_type` - The type of enhancement to apply
    /// * `provider` - The AI provider to use
    /// * `api_key` - The API key for the provider
    /// * `model` - Optional model to use
    /// * `max_concurrent` - Maximum number of concurrent enhancements (default: 5)
    pub async fn batch_enhance_with_type(
        &self,
        context_ids: Vec<String>,
        enhancement_type: crate::context_mcp::ContextEnhancementType,
        provider: impl Into<String>,
        api_key: impl Into<String>,
        model: Option<String>,
        max_concurrent: Option<usize>,
    ) -> Result<Vec<(String, Result<()>)>> {
        // Create options
        let options = crate::context_mcp::ContextAiEnhancementOptions::new(
            enhancement_type,
            provider,
            api_key,
        );
        
        // Apply model if provided
        let options = if let Some(model) = model {
            options.with_model(model)
        } else {
            options
        };
        
        // Use the batch enhancement method
        self.batch_enhance_contexts(context_ids, options, max_concurrent).await
    }
}