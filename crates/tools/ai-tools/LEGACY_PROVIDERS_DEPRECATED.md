# Legacy AI Providers - Deprecation Notice

**Date**: January 19, 2026  
**Status**: DEPRECATED - Do Not Use in New Code  
**Removal Target**: Version 2.0.0

---

## ⚠️ Deprecation Notice

The following AI provider modules are **DEPRECATED** and will be removed in a future release:

### Deprecated Modules

1. **`openai/`** - OpenAI HTTP client
   - Uses reqwest for direct API calls
   - Pulls in `ring` C dependency via TLS
   - Blocks TRUE ecoBin certification

2. **`anthropic/`** - Anthropic HTTP client
   - Uses reqwest for direct API calls
   - Pulls in `ring` C dependency via TLS
   - Blocks TRUE ecoBin certification

3. **`gemini/`** - Google Gemini HTTP client
   - Uses reqwest for direct API calls
   - Pulls in `ring` C dependency via TLS
   - Blocks TRUE ecoBin certification

4. **`local/ollama.rs`** - Local Ollama HTTP client
   - Uses reqwest for local API calls
   - Not needed with capability delegation

5. **`common/clients/ollama.rs`** - Alternative Ollama client
   - Uses reqwest for local API calls
   - Duplicate functionality

---

## 🚀 Migration Path

### Use `capability_ai` Instead!

**Old Code** (DEPRECATED):
```rust
use squirrel_ai_tools::openai::OpenAIClient;

let client = OpenAIClient::new(api_key);
let response = client.chat(request).await?;
```

**New Code** (RECOMMENDED):
```rust
use squirrel_ai_tools::capability_ai::{AiClient, ChatMessage};

let client = AiClient::from_env()?;
let messages = vec![
    ChatMessage::system("You are helpful"),
    ChatMessage::user("Hello"),
];
let response = client.chat_completion("gpt-4", messages, None).await?;
```

---

## 📋 Why Migrate?

### Problems with Old Providers

❌ **C Dependencies**: reqwest → rustls → ring → C code  
❌ **Cross-Compilation**: Blocked on some architectures  
❌ **TRUE ecoBin**: Cannot achieve certification  
❌ **Not Ecological**: Each primal implements HTTP  
❌ **Maintenance**: Duplicate TLS/retry/error logic

### Benefits of New Pattern

✅ **Pure Rust**: Zero C dependencies (Unix sockets only)  
✅ **Cross-Platform**: Compiles to all architectures  
✅ **TRUE ecoBin**: Certified compliance  
✅ **Ecological**: Songbird handles all HTTP  
✅ **Simpler**: Less code, fewer bugs

---

## 📚 Complete Migration Guide

See **`docs/CAPABILITY_AI_MIGRATION_GUIDE.md`** for:
- Quick start examples
- API reference
- Migration checklist
- Testing strategies
- Common issues and solutions

---

## ⏰ Timeline

### Version 1.4.1 (Now)
- ✅ Modules marked as deprecated
- ✅ Warnings on compilation
- ✅ Migration guide available
- ✅ New pattern proven and documented

### Version 1.5.0 (Upcoming)
- 🚧 Core usages migrated
- 🚧 Tests updated
- 🚧 Examples updated
- 📝 Final migration notices

### Version 2.0.0 (Future)
- ❌ Deprecated modules removed
- ✅ Only capability-based clients remain
- ✅ TRUE ecoBin fully achieved
- ✅ Cross-compilation validated

---

## 🔍 Finding Usages

### Check Your Code

```bash
# Find OpenAI usages
rg "use.*openai" --type rust

# Find Anthropic usages
rg "use.*anthropic" --type rust

# Find Gemini usages
rg "use.*gemini" --type rust

# Find all old provider usages
rg "OpenAIClient|AnthropicClient|GeminiClient" --type rust
```

### Update Strategy

1. **Identify** all usages in your crate
2. **Read** migration guide
3. **Update** one usage at a time
4. **Test** each change
5. **Remove** old imports
6. **Verify** zero reqwest in `Cargo.lock`

---

## 💡 Common Questions

### Q: Can I still use these modules?

**A**: Yes, but they're deprecated. You'll see compiler warnings. They will be removed in v2.0.0.

### Q: What if I need a feature not in capability_ai?

**A**: File an issue! We'll add it. The capability pattern is extensible.

### Q: Can I keep using reqwest in my own code?

**A**: You can, but it defeats the purpose. Delegate HTTP to Songbird for TRUE ecoBin compliance.

### Q: What about local Ollama?

**A**: Use `capability_ai` which can route to local providers via Songbird configuration.

### Q: How do I test without Songbird running?

**A**: Mock the Unix socket. See migration guide for examples.

---

## 📊 Migration Progress

Track migration progress:

```bash
# Old provider usages remaining
rg "OpenAIClient|AnthropicClient|GeminiClient" --count

# New capability usages
rg "capability_ai::AiClient" --count

# Goal: 0 old usages, all new!
```

---

## 🆘 Need Help?

1. **Read**: `docs/CAPABILITY_AI_MIGRATION_GUIDE.md`
2. **Example**: See `capability_ai.rs` source
3. **Issues**: File on GitHub with "migration" label
4. **Chat**: #ecoPrimals channel

---

## 🎯 The Goal

**TRUE ecoBin #5 Certification**:
- ✅ Zero C dependencies
- ✅ 100% Pure Rust
- ✅ Cross-compiles everywhere
- ✅ Ecological architecture
- ✅ Deploy like an infant 🍼

**We're Almost There!**

Current Status:
- ✅ Core services: 100% migrated
- ✅ Security providers: 100% migrated
- 🚧 Old AI clients: Migration ongoing
- ⏳ Full TRUE ecoBin: In progress

---

*Deprecated: January 19, 2026*  
*Removal Target: Version 2.0.0*  
*Migration Guide: docs/CAPABILITY_AI_MIGRATION_GUIDE.md*  
*Philosophy: Progress over perfection! 🌍🦀✨*

