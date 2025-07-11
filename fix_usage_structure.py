# Read the file
with open('code/crates/tools/ai-tools/src/common/mod.rs', 'r') as f:
    lines = f.readlines()

# Find and fix the broken UsageInfo structure around line 770
for i, line in enumerate(lines):
    if 'usage: Some(UsageInfo {' in line:
        # Replace the broken section with a proper one
        lines[i:i+15] = [
            '            usage: Some(UsageInfo {\n',
            '                prompt_tokens: response_json["usage"]["prompt_tokens"]\n',
            '                    .as_u64()\n',
            '                    .unwrap_or(0) as u32,\n',
            '                completion_tokens: response_json["usage"]["completion_tokens"]\n',
            '                    .as_u64()\n',
            '                    .unwrap_or(0) as u32,\n',
            '                total_tokens: response_json["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32,\n',
            '            }),\n',
            '        })\n',
            '    }\n',
            '\n',
            '    fn name(&self) -> &str {\n',
            '        "openai"\n',
            '    }\n'
        ]
        break

# Write back
with open('code/crates/tools/ai-tools/src/common/mod.rs', 'w') as f:
    f.writelines(lines)

print("Fixed UsageInfo structure")
