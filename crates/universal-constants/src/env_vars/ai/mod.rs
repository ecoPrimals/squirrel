// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! AI provider env vars

/// Default AI provider
pub const DEFAULT_PROVIDER: &str = "AI_DEFAULT_PROVIDER";
/// HTTP provider config (JSON)
pub const HTTP_PROVIDERS: &str = "AI_HTTP_PROVIDERS";
/// AI provider socket paths (comma-separated)
pub const PROVIDER_SOCKETS: &str = "AI_PROVIDER_SOCKETS";
/// AI service host
pub const SERVICE_HOST: &str = "AI_SERVICE_HOST";
/// AI service name
pub const SERVICE_NAME: &str = "AI_SERVICE_NAME";
/// AI request timeout (ms)
pub const REQUEST_TIMEOUT_MS: &str = "AI_REQUEST_TIMEOUT_MS";
/// AI intelligence interval (seconds)
pub const INTELLIGENCE_INTERVAL_SECS: &str = "AI_INTELLIGENCE_INTERVAL_SECS";
/// Inference endpoint (generic, primal-agnostic)
pub const INFERENCE_ENDPOINT: &str = "INFERENCE_ENDPOINT";
/// AI inference endpoint (prefixed variant)
pub const AI_INFERENCE_ENDPOINT: &str = "AI_INFERENCE_ENDPOINT";

pub mod anthropic;
pub mod gemini;
pub mod huggingface;
pub mod local;
pub mod ollama;
pub mod openai;
