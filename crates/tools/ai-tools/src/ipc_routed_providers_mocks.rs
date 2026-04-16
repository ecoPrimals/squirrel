// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Test-only HTTP delegate mock for IPC-routed vendor client tests.

use super::IpcHttpDelegate;
use crate::neural_http::HttpResponse;

/// Deterministic mock for [`IpcHttpDelegate`] (FIFO responses per operation).
pub struct MockNeuralHttp {
    post_json_responses:
        std::sync::Arc<tokio::sync::Mutex<std::collections::VecDeque<HttpResponse>>>,
    get_responses: std::sync::Arc<tokio::sync::Mutex<std::collections::VecDeque<HttpResponse>>>,
}

impl MockNeuralHttp {
    pub fn new() -> Self {
        Self {
            post_json_responses: std::sync::Arc::new(tokio::sync::Mutex::new(
                std::collections::VecDeque::new(),
            )),
            get_responses: std::sync::Arc::new(tokio::sync::Mutex::new(
                std::collections::VecDeque::new(),
            )),
        }
    }

    pub async fn push_post_json(&self, body: impl Into<String>) {
        self.post_json_responses
            .lock()
            .await
            .push_back(HttpResponse {
                status: 200,
                headers: vec![],
                body: body.into(),
            });
    }

    pub async fn push_get(&self, body: impl Into<String>) {
        self.get_responses.lock().await.push_back(HttpResponse {
            status: 200,
            headers: vec![],
            body: body.into(),
        });
    }
}

impl IpcHttpDelegate for MockNeuralHttp {
    async fn post_json(
        &self,
        _url: &str,
        _headers: Vec<(String, String)>,
        _body: &str,
    ) -> anyhow::Result<HttpResponse> {
        self.post_json_responses
            .lock()
            .await
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("MockNeuralHttp: no post_json response queued"))
    }

    async fn get(
        &self,
        _url: &str,
        _headers: Vec<(String, String)>,
    ) -> anyhow::Result<HttpResponse> {
        self.get_responses
            .lock()
            .await
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("MockNeuralHttp: no get response queued"))
    }
}
