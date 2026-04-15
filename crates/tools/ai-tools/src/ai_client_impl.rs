// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Concrete [`AiClientImpl`] enum for static dispatch over AI client backends.
//!
//! The [`AiClientImpl::Mock`] variant is **not** part of default production builds: it is
//! compiled only under `cfg(test)` or the `testing` Cargo feature (see crate `Cargo.toml`).
//! [`AiClientImpl::RouterHarness`] is `cfg(test)` only.

use crate::common::capability::AICapabilities;
use crate::common::client::AIClient;
use crate::common::types::{ChatRequest, ChatResponse, ChatResponseStream};
use crate::ipc_routed_providers::IpcRoutedVendorClient;
use crate::neural_http::NeuralHttpClient;

#[cfg(any(test, feature = "testing"))]
use crate::common::clients::mock::MockAIClient;

#[cfg(test)]
use crate::router::harness::RouterHarnessClient;

/// Runtime-selected AI client implementation (enum dispatch, no `dyn`).
#[derive(Debug)]
#[cfg_attr(
    test,
    expect(
        clippy::large_enum_variant,
        reason = "large RouterHarness variant is test-only"
    )
)]
pub enum AiClientImpl {
    /// IPC-delegated vendor HTTP (`OpenAI` / `Anthropic` / `Gemini` shapes).
    IpcRouted(Box<IpcRoutedVendorClient<NeuralHttpClient>>),
    #[cfg(any(test, feature = "testing"))]
    /// Mock client for tests and the `testing` feature.
    Mock(MockAIClient),
    #[cfg(test)]
    /// Configurable harness for router unit tests (`optimization`, `dispatch`).
    RouterHarness(RouterHarnessClient),
}

impl AIClient for AiClientImpl {
    fn provider_name(&self) -> &str {
        match self {
            Self::IpcRouted(c) => c.provider_name(),
            #[cfg(any(test, feature = "testing"))]
            Self::Mock(c) => c.provider_name(),
            #[cfg(test)]
            Self::RouterHarness(c) => c.provider_name(),
        }
    }

    async fn get_capabilities(&self, model: &str) -> crate::Result<AICapabilities> {
        match self {
            Self::IpcRouted(c) => c.get_capabilities(model).await,
            #[cfg(any(test, feature = "testing"))]
            Self::Mock(c) => c.get_capabilities(model).await,
            #[cfg(test)]
            Self::RouterHarness(c) => c.get_capabilities(model).await,
        }
    }

    async fn chat(&self, request: ChatRequest) -> crate::Result<ChatResponse> {
        match self {
            Self::IpcRouted(c) => c.chat(request).await,
            #[cfg(any(test, feature = "testing"))]
            Self::Mock(c) => c.chat(request).await,
            #[cfg(test)]
            Self::RouterHarness(c) => c.chat(request).await,
        }
    }

    async fn chat_stream(&self, request: ChatRequest) -> crate::Result<ChatResponseStream> {
        match self {
            Self::IpcRouted(c) => c.chat_stream(request).await,
            #[cfg(any(test, feature = "testing"))]
            Self::Mock(c) => c.chat_stream(request).await,
            #[cfg(test)]
            Self::RouterHarness(c) => c.chat_stream(request).await,
        }
    }

    async fn list_models(&self) -> crate::Result<Vec<String>> {
        match self {
            Self::IpcRouted(c) => c.list_models().await,
            #[cfg(any(test, feature = "testing"))]
            Self::Mock(c) => c.list_models().await,
            #[cfg(test)]
            Self::RouterHarness(c) => c.list_models().await,
        }
    }

    async fn is_available(&self) -> bool {
        match self {
            Self::IpcRouted(c) => c.is_available().await,
            #[cfg(any(test, feature = "testing"))]
            Self::Mock(c) => c.is_available().await,
            #[cfg(test)]
            Self::RouterHarness(c) => c.is_available().await,
        }
    }

    fn default_model(&self) -> &str {
        match self {
            Self::IpcRouted(c) => c.default_model(),
            #[cfg(any(test, feature = "testing"))]
            Self::Mock(c) => c.default_model(),
            #[cfg(test)]
            Self::RouterHarness(c) => c.default_model(),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        match self {
            Self::IpcRouted(c) => c.as_any(),
            #[cfg(any(test, feature = "testing"))]
            Self::Mock(c) => c.as_any(),
            #[cfg(test)]
            Self::RouterHarness(c) => c.as_any(),
        }
    }

    fn capabilities(&self) -> AICapabilities {
        match self {
            Self::IpcRouted(c) => c.capabilities(),
            #[cfg(any(test, feature = "testing"))]
            Self::Mock(c) => c.capabilities(),
            #[cfg(test)]
            Self::RouterHarness(c) => c.capabilities(),
        }
    }

    fn priority(&self) -> u32 {
        match self {
            Self::IpcRouted(c) => c.priority(),
            #[cfg(any(test, feature = "testing"))]
            Self::Mock(c) => c.priority(),
            #[cfg(test)]
            Self::RouterHarness(c) => c.priority(),
        }
    }

    fn routing_preferences(&self) -> crate::common::capability::RoutingPreferences {
        match self {
            Self::IpcRouted(c) => c.routing_preferences(),
            #[cfg(any(test, feature = "testing"))]
            Self::Mock(c) => c.routing_preferences(),
            #[cfg(test)]
            Self::RouterHarness(c) => c.routing_preferences(),
        }
    }
}
