import re

# Read the file
with open('code/crates/tools/ai-tools/src/common/mod.rs', 'r') as f:
    content = f.read()

# Fix broken ChatResponse pattern - this is complex so let's do simpler fixes first
# Fix the simple field issues
content = re.sub(r'request\.model\.clone\(\)', 'request.model.clone().unwrap_or_else(|| "unknown".to_string())', content)
content = re.sub(r'request\.model', 'request.model.clone().unwrap_or_else(|| "unknown".to_string())', content)

# Write back
with open('code/crates/tools/ai-tools/src/common/mod.rs', 'w') as f:
    f.write(content)

print("Fixed model field issues")
