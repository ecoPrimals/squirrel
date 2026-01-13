//! RPC Handler implementations for protocol router
//!
//! These handlers wire the protocol router to actual implementations.

use crate::error::PrimalError;
use serde_json::Value;

/// Handle tarpc request - wire to actual handlers
#[cfg(feature = "tarpc-rpc")]
pub async fn handle_request(method: &str, params: Value) -> Result<Value, PrimalError> {
    tracing::debug!("tarpc handler: method={}", method);

    // Wire to actual RPC handlers
    match method {
        "query_ai" => {
            let request = serde_json::from_value(params)
                .map_err(|e| PrimalError::InvalidInput(format!("Invalid params: {}", e)))?;

            let response = crate::rpc::handlers_internal::handle_query_ai_internal(request).await?;
            Ok(serde_json::to_value(&response)?)
        }
        "list_providers" => {
            let response = crate::rpc::handlers_internal::handle_list_providers_internal().await?;
            Ok(serde_json::to_value(&response)?)
        }
        "health_check" => {
            let response = crate::rpc::handlers_internal::handle_health_check_internal().await?;
            Ok(serde_json::to_value(&response)?)
        }
        _ => Err(PrimalError::InvalidInput(format!(
            "Unknown method: {}",
            method
        ))),
    }
}

/// Handle JSON-RPC request - wire to actual handlers
pub async fn handle_jsonrpc_request(method: &str, params: Value) -> Result<Value, PrimalError> {
    tracing::debug!("JSON-RPC handler: method={}", method);

    // Wire to actual RPC handlers
    match method {
        "query_ai" => {
            let request = serde_json::from_value(params)
                .map_err(|e| PrimalError::InvalidInput(format!("Invalid params: {}", e)))?;

            let response = crate::rpc::handlers_internal::handle_query_ai_internal(request).await?;
            Ok(serde_json::to_value(&response)?)
        }
        "list_providers" => {
            let response = crate::rpc::handlers_internal::handle_list_providers_internal().await?;
            Ok(serde_json::to_value(&response)?)
        }
        "health_check" => {
            let response = crate::rpc::handlers_internal::handle_health_check_internal().await?;
            Ok(serde_json::to_value(&response)?)
        }
        _ => Err(PrimalError::InvalidInput(format!(
            "Unknown method: {}",
            method
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_jsonrpc_handler() {
        let params = serde_json::json!({
            "prompt": "test"
        });

        let result = handle_jsonrpc_request("query_ai", params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[cfg(feature = "tarpc-rpc")]
    async fn test_tarpc_handler() {
        let params = serde_json::json!({
            "prompt": "test"
        });

        let result = handle_request("query_ai", params).await;
        assert!(result.is_ok());
    }
}
