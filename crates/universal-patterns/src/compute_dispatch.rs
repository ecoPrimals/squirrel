// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Typed `compute.dispatch` client for ToadStool GPU routing.
//!
//! Pattern absorbed from coralReef v0.4.18: a strongly-typed client
//! for requesting GPU compute via JSON-RPC `compute.dispatch`, so
//! callers don't hand-build JSON.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Request payload for `compute.dispatch`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeDispatchRequest {
    /// The compute task type (e.g. "inference", "embedding", "shader").
    pub task_type: String,
    /// Model or program identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    /// Input data for the compute task.
    pub input: Value,
    /// Priority hint (0 = best-effort, 10 = critical).
    #[serde(default)]
    pub priority: u8,
    /// Maximum time the caller is willing to wait (milliseconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    /// Whether to prefer GPU over CPU.
    #[serde(default = "default_true")]
    pub prefer_gpu: bool,
}

fn default_true() -> bool {
    true
}

/// Response payload from `compute.dispatch`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeDispatchResponse {
    /// Compute task result.
    pub result: Value,
    /// Execution backend ("gpu", "cpu", "hybrid").
    pub backend: String,
    /// Wall-clock execution time in milliseconds.
    pub execution_ms: u64,
    /// GPU device used (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_device: Option<String>,
}

impl ComputeDispatchRequest {
    /// Build an inference request.
    #[must_use]
    pub fn inference(model_id: &str, input: Value) -> Self {
        Self {
            task_type: "inference".to_string(),
            model_id: Some(model_id.to_string()),
            input,
            priority: 5,
            timeout_ms: None,
            prefer_gpu: true,
        }
    }

    /// Build an embedding request.
    #[must_use]
    pub fn embedding(model_id: &str, texts: &[&str]) -> Self {
        Self {
            task_type: "embedding".to_string(),
            model_id: Some(model_id.to_string()),
            input: serde_json::json!({ "texts": texts }),
            priority: 5,
            timeout_ms: None,
            prefer_gpu: true,
        }
    }

    /// Convert to JSON-RPC params for `compute.dispatch`.
    #[must_use]
    pub fn to_params(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    /// Set priority.
    #[must_use]
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Set timeout.
    #[must_use]
    pub fn with_timeout_ms(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inference_request_builder() {
        let req =
            ComputeDispatchRequest::inference("llama2:7b", serde_json::json!({"prompt": "hi"}));
        assert_eq!(req.task_type, "inference");
        assert_eq!(req.model_id.as_deref(), Some("llama2:7b"));
        assert!(req.prefer_gpu);
        assert_eq!(req.priority, 5);
    }

    #[test]
    fn embedding_request_builder() {
        let req = ComputeDispatchRequest::embedding("bge-small", &["hello", "world"]);
        assert_eq!(req.task_type, "embedding");
        let texts = req.input["texts"].as_array().unwrap();
        assert_eq!(texts.len(), 2);
    }

    #[test]
    fn request_to_params_roundtrip() {
        let req = ComputeDispatchRequest::inference("test", serde_json::json!("input"))
            .with_priority(10)
            .with_timeout_ms(5000);
        let params = req.to_params();
        let deser: ComputeDispatchRequest = serde_json::from_value(params).unwrap();
        assert_eq!(deser.priority, 10);
        assert_eq!(deser.timeout_ms, Some(5000));
    }

    #[test]
    fn response_serde() {
        let resp = ComputeDispatchResponse {
            result: serde_json::json!({"text": "hello"}),
            backend: "gpu".to_string(),
            execution_ms: 42,
            gpu_device: Some("cuda:0".to_string()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let deser: ComputeDispatchResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.backend, "gpu");
        assert_eq!(deser.execution_ms, 42);
    }

    #[test]
    fn prefer_gpu_default_is_true() {
        let json = r#"{"task_type":"test","input":null,"priority":0}"#;
        let req: ComputeDispatchRequest = serde_json::from_str(json).unwrap();
        assert!(req.prefer_gpu);
    }
}
