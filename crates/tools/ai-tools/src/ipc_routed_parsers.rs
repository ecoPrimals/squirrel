// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! JSON response parsing for IPC-routed vendor HTTP clients.

use serde_json::{Map, Value};

use crate::common::types::{ChatChoice, ChatResponse, MessageRole, UsageInfo};
use crate::error::{Error, Result};

#[inline]
pub fn json_u64_as_u32_saturating(n: u64) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

pub fn parse_openai_chat_response(body: &str) -> Result<ChatResponse> {
    let v: Value = serde_json::from_str(body).map_err(|e| Error::Parse(e.to_string()))?;
    let id = v["id"].as_str().unwrap_or("openai-ipc").to_string();
    let model = v["model"].as_str().unwrap_or("unknown").to_string();
    let choice0 = &v["choices"][0];
    let content = choice0["message"]["content"].as_str().map(str::to_string);
    let finish = choice0["finish_reason"].as_str().map(str::to_string);
    let usage = v["usage"]
        .as_object()
        .map(|u: &Map<String, Value>| UsageInfo {
            prompt_tokens: json_u64_as_u32_saturating(u["prompt_tokens"].as_u64().unwrap_or(0)),
            completion_tokens: json_u64_as_u32_saturating(
                u["completion_tokens"].as_u64().unwrap_or(0),
            ),
            total_tokens: json_u64_as_u32_saturating(u["total_tokens"].as_u64().unwrap_or(0)),
        });
    Ok(ChatResponse {
        id,
        model,
        choices: vec![ChatChoice {
            index: 0,
            role: MessageRole::Assistant,
            content,
            finish_reason: finish,
            tool_calls: None,
        }],
        usage,
    })
}

pub fn parse_anthropic_chat_response(body: &str) -> Result<ChatResponse> {
    let v: Value = serde_json::from_str(body).map_err(|e| Error::Parse(e.to_string()))?;
    let id = v["id"].as_str().unwrap_or("anthropic-ipc").to_string();
    let model = v["model"].as_str().unwrap_or("unknown").to_string();
    let mut content: Option<String> = None;
    if let Some(arr) = v["content"].as_array() {
        for block in arr {
            if block["type"].as_str() == Some("text") {
                content = block["text"].as_str().map(str::to_string);
                break;
            }
        }
    }
    let usage = v["usage"].as_object().map(|u: &Map<String, Value>| {
        let input = json_u64_as_u32_saturating(u["input_tokens"].as_u64().unwrap_or(0));
        let output = json_u64_as_u32_saturating(u["output_tokens"].as_u64().unwrap_or(0));
        UsageInfo {
            prompt_tokens: input,
            completion_tokens: output,
            total_tokens: input.saturating_add(output),
        }
    });
    Ok(ChatResponse {
        id,
        model,
        choices: vec![ChatChoice {
            index: 0,
            role: MessageRole::Assistant,
            content,
            finish_reason: v["stop_reason"].as_str().map(str::to_string),
            tool_calls: None,
        }],
        usage,
    })
}

pub fn parse_gemini_chat_response(body: &str, model: &str) -> Result<ChatResponse> {
    let v: Value = serde_json::from_str(body).map_err(|e| Error::Parse(e.to_string()))?;
    let mut text = String::new();
    if let Some(c) = v["candidates"][0]["content"]["parts"].as_array() {
        for p in c {
            if let Some(t) = p["text"].as_str() {
                text.push_str(t);
            }
        }
    }
    Ok(ChatResponse {
        id: "gemini-ipc".to_string(),
        model: model.to_string(),
        choices: vec![ChatChoice {
            index: 0,
            role: MessageRole::Assistant,
            content: Some(text),
            finish_reason: None,
            tool_calls: None,
        }],
        usage: None,
    })
}
