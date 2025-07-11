# Read the file
with open('code/crates/tools/ai-tools/src/common/mod.rs', 'r') as f:
    content = f.read()

# Fix the broken Ollama ChatResponse
old_ollama = '''        Ok(ChatResponse {
            id: format!("ollama-{}", uuid::Uuid::new_v4()),
            content,
                            tool_calls: None,
            }],
            model: request.model.clone().unwrap_or_else(|| "unknown".to_string()),
            usage: Some(UsageInfo {
                prompt_tokens: response_json["prompt_eval_count"].as_u64().unwrap_or(0) as u32,
                completion_tokens: response_json["eval_count"].as_u64().unwrap_or(0) as u32,
                total_tokens: (response_json["prompt_eval_count"].as_u64().unwrap_or(0)
                    + response_json["eval_count"].as_u64().unwrap_or(0))
                    as u32,
            }),
            finish_reason: if response_json["done"].as_bool().unwrap_or(false) {
                Some("stop".to_string())
            } else {
                None
            },
        })'''

new_ollama = '''        Ok(ChatResponse {
            id: format!("ollama-{}", uuid::Uuid::new_v4()),
            model: request.model.clone().unwrap_or_else(|| "unknown".to_string()),
            choices: vec![ChatChoice {
                index: 0,
                role: MessageRole::Assistant,
                content: Some(content),
                finish_reason: if response_json["done"].as_bool().unwrap_or(false) {
                    Some("stop".to_string())
                } else {
                    None
                },
                tool_calls: None,
            }],
            usage: Some(UsageInfo {
                prompt_tokens: response_json["prompt_eval_count"].as_u64().unwrap_or(0) as u32,
                completion_tokens: response_json["eval_count"].as_u64().unwrap_or(0) as u32,
                total_tokens: (response_json["prompt_eval_count"].as_u64().unwrap_or(0)
                    + response_json["eval_count"].as_u64().unwrap_or(0))
                    as u32,
            }),
        })'''

content = content.replace(old_ollama, new_ollama)

# Write back
with open('code/crates/tools/ai-tools/src/common/mod.rs', 'w') as f:
    f.write(content)

print("Fixed Ollama ChatResponse")
