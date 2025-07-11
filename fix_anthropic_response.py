# Read the file
with open('code/crates/tools/ai-tools/src/common/mod.rs', 'r') as f:
    content = f.read()

# Fix the broken Anthropic ChatResponse
old_anthropic = '''        Ok(ChatResponse {
            id: response_json["id"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            content,
                            tool_calls: None,
            }],
            model: request.model.clone().unwrap_or_else(|| "unknown".to_string()),
            usage: Some(UsageInfo {
                prompt_tokens: response_json["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
                completion_tokens: response_json["usage"]["output_tokens"]
                    .as_u64()
                    .unwrap_or(0) as u32,
                total_tokens: (response_json["usage"]["input_tokens"].as_u64().unwrap_or(0)
                    + response_json["usage"]["output_tokens"]
                        .as_u64()
                        .unwrap_or(0)) as u32,
            }),
            finish_reason: response_json["stop_reason"].as_str().map(|s| s.to_string()),
        })'''

new_anthropic = '''        Ok(ChatResponse {
            id: response_json["id"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            model: request.model.clone().unwrap_or_else(|| "unknown".to_string()),
            choices: vec![ChatChoice {
                index: 0,
                role: MessageRole::Assistant,
                content: Some(content),
                finish_reason: response_json["stop_reason"].as_str().map(|s| s.to_string()),
                tool_calls: None,
            }],
            usage: Some(UsageInfo {
                prompt_tokens: response_json["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
                completion_tokens: response_json["usage"]["output_tokens"]
                    .as_u64()
                    .unwrap_or(0) as u32,
                total_tokens: (response_json["usage"]["input_tokens"].as_u64().unwrap_or(0)
                    + response_json["usage"]["output_tokens"]
                        .as_u64()
                        .unwrap_or(0)) as u32,
            }),
        })'''

content = content.replace(old_anthropic, new_anthropic)

# Write back
with open('code/crates/tools/ai-tools/src/common/mod.rs', 'w') as f:
    f.write(content)

print("Fixed Anthropic ChatResponse")
