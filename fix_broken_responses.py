# Read the file
with open('code/crates/tools/ai-tools/src/common/mod.rs', 'r') as f:
    lines = f.readlines()

# Fix the broken ChatResponse around line 755-770
for i, line in enumerate(lines):
    if i >= 754 and i <= 770:  # Around line 755
        if 'Ok(ChatResponse {' in line:
            # Replace this entire broken section
            new_section = '''        Ok(ChatResponse {
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
        })
'''
            # Find end of broken section and replace
            j = i
            while j < len(lines) and '})' not in lines[j]:
                j += 1
            if j < len(lines):
                # Replace from i to j+1
                lines[i:j+1] = [new_section + '\n']
            break

# Write back
with open('code/crates/tools/ai-tools/src/common/mod.rs', 'w') as f:
    f.writelines(lines)

print("Fixed broken ChatResponse around line 755")
