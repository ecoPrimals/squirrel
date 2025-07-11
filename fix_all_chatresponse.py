import re

# Read the file
with open('code/crates/tools/ai-tools/src/common/mod.rs', 'r') as f:
    content = f.read()

# Fix all ChatResponse constructions to use proper structure
# Pattern for OpenAI response around line 755
content = re.sub(
    r'Ok\(ChatResponse \{\s*id: response_json\["id"\]\s*\.as_str\(\)\s*\.unwrap_or\("unknown"\)\s*\.to_string\(\),\s*content,\s*tool_calls: None,\s*\}\],\s*model: request\.model\.clone\(\)\.unwrap_or_else\(\|\| "unknown"\.to_string\(\)\),\s*usage: Some\(UsageInfo \{[^}]*\}\),\s*finish_reason: response_json\["choices"\]\[0\]\["finish_reason"\][^}]*\}\),\s*\}\)',
    '''Ok(ChatResponse {
            id: response_json["id"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            model: request.model.clone().unwrap_or_else(|| "unknown".to_string()),
            choices: vec![ChatChoice {
                index: 0,
                role: MessageRole::Assistant,
                content: Some(content),
                finish_reason: response_json["choices"][0]["finish_reason"]
                    .as_str()
                    .map(|s| s.to_string()),
                tool_calls: None,
            }],
            usage: Some(UsageInfo {
                prompt_tokens: response_json["usage"]["prompt_tokens"]
                    .as_u64()
                    .unwrap_or(0) as u32,
                completion_tokens: response_json["usage"]["completion_tokens"]
                    .as_u64()
                    .unwrap_or(0) as u32,
                total_tokens: response_json["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32,
            }),
        })''',
    content,
    flags=re.MULTILINE | re.DOTALL
)

# Find and fix all other ChatResponse constructions with similar pattern
# Look for patterns with content, model, usage, finish_reason directly on ChatResponse
patterns = [
    (
        r'Ok\(ChatResponse \{\s*id: [^,]*,\s*content,\s*model: [^,]*,\s*usage: [^,]*,\s*finish_reason: [^}]*\}\)',
        'PLACEHOLDER_FOR_PROPER_STRUCTURE'
    )
]

# Write back
with open('code/crates/tools/ai-tools/src/common/mod.rs', 'w') as f:
    f.write(content)

print("Fixed ChatResponse structures")
