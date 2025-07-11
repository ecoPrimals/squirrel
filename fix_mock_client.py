import re

# Read the file
with open('code/crates/tools/ai-tools/src/common/mod.rs', 'r') as f:
    content = f.read()

# Fix the MockAIClient ChatResponse structure
old_pattern = r'''Ok\(ChatResponse \{
            id: format!\("mock-\{\}", uuid::Uuid::new_v4\(\)\),
            choices: vec!\[ChatChoice \{
                index: 0,
                role: MessageRole::Assistant,
                content: Some\(format!\(
                "Mock response to: \{\}",
                request
                    \.messages
                    \.last\(\)
                    \.map\(\|m\| m\.content\.as_str\(\)\)
                    \.unwrap_or\("No message"\)
            \),
                            tool_calls: None,
            \}\],
            model: request\.model\.clone\(\)\.unwrap_or_else\(\|\| "unknown"\.to_string\(\)\),
            usage: Some\(UsageInfo \{
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            \}\),
            finish_reason: Some\("stop"\.to_string\(\)\),
        \}\)'''

new_pattern = '''Ok(ChatResponse {
            id: format!("mock-{}", uuid::Uuid::new_v4()),
            choices: vec![ChatChoice {
                index: 0,
                role: MessageRole::Assistant,
                content: Some(format!(
                    "Mock response to: {}",
                    request
                        .messages
                        .last()
                        .and_then(|m| m.content.as_ref())
                        .unwrap_or(&"No message".to_string())
                )),
                finish_reason: Some("stop".to_string()),
                tool_calls: None,
            }],
            model: request.model.clone().unwrap_or_else(|| "unknown".to_string()),
            usage: Some(UsageInfo {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            }),
        })'''

content = re.sub(old_pattern, new_pattern, content, flags=re.MULTILINE | re.DOTALL)

# Write back
with open('code/crates/tools/ai-tools/src/common/mod.rs', 'w') as f:
    f.write(content)

print("Fixed MockAIClient ChatResponse structure")
