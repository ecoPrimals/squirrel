---
description: Guide for AI teams developing for Squirrel
version: 1.0.0
last_updated: 2026-03-16
status: historical
owner: Core Team
---

# AI Development Guide for Squirrel

> **Note (April 2026):** Gen2-era guide (March 2026). Some patterns and prerequisites are outdated — Rust 1.94+ / edition 2024 is now required. See `README.md` and `CURRENT_STATUS.md` for current standards.

## Overview

This guide provides essential information for AI teams working on the Squirrel project. It covers development workflows, key architectural patterns, best practices, and integration points specific to AI components.

## Getting Started

### Prerequisites

- Rust 1.94 or newer (edition 2024)

### Development Environment Setup

1. Clone the ecoPrimals repository and navigate to the Squirrel workspace:
   ```bash
   git clone <ecoPrimals-repo-url>
   cd squirrel
   ```

2. Build the workspace:
   ```bash
   cargo build
   ```

3. Set environment variables as needed (see `crates/universal-constants/src/deployment.rs` for available overrides):
   ```bash
   export SQUIRREL_SERVER_PORT=3000
   export LOCAL_AI_ENDPOINT=http://localhost:11434
   ```

## Project Structure for AI Components

The AI components are primarily located in the following directories:

```
squirrel/
├── crates/
│   ├── tools/
│   │   └── ai-tools/               # Primary AI integration tools
│   ├── core/
│   │   ├── mcp/                    # Machine Context Protocol
│   │   └── context/                # Context management + learning
│   ├── main/
│   │   └── src/
│   │       └── ai/                 # AI routing, provider selection
│   └── universal-constants/        # Primal identity, deployment config
└── specs/
    └── active/
        └── mcp-protocol/           # MCP protocol specifications
```

## Key AI Integration Points

### 1. Machine Context Protocol (MCP)

The MCP provides the core framework for AI agent interactions:

```rust
// Example MCP AI Tool Handler
#[derive(Debug)]
pub struct AIToolHandler {
    config: AIToolConfig,
    client: Arc<AIClient>,
}

impl ToolHandler for AIToolHandler {
    fn handle_tool(&self, ctx: &ToolContext, req: ToolRequest) -> Result<ToolResponse> {
        // Implement AI tool handling logic
    }
}
```

### 2. PyO3 Bindings

For integrating Python ML models:

```rust
#[pyclass]
struct ModelManager {
    models: HashMap<String, PyObject>,
}

#[pymethods]
impl ModelManager {
    #[new]
    fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    fn load_model(&mut self, name: String, model_path: String, py: Python<'_>) -> PyResult<()> {
        // Load model implementation
    }

    fn predict(&self, name: String, input_data: PyObject, py: Python<'_>) -> PyResult<PyObject> {
        // Prediction implementation
    }
}
```

### 3. AI Tools Framework

For implementing custom AI tools:

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct AIToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: AIToolParameters,
    pub requires_auth: bool,
}

pub trait AIToolProvider: Send + Sync {
    fn get_tool_definitions(&self) -> Vec<AIToolDefinition>;
    fn execute_tool(&self, tool_name: &str, parameters: Value) -> Result<Value>;
}
```

## AI Development Workflow

1. **Specification First**: Begin by updating or creating specifications in `specs/tools/ai-tools/`
2. **Implementation**: Implement the AI functionality in `crates/tools/ai-tools/`
3. **Testing**: Write tests in `tests/py/` and `tests/rs/`
4. **Integration**: Integrate with the MCP using the ToolHandler interface
5. **Ecosystem Integration**: Wire capability discovery for AI-dependent operations

## Best Practices for AI Development

### 1. Model Management

- Store models in a dedicated directory (`data/models/`)
- Use versioned model files (e.g., `model_v1.0.onnx`)
- Support model hot-swapping where possible
- Implement graceful fallbacks for missing models

### 2. Performance Considerations

- Implement lazy loading for large models
- Use batching where appropriate
- Consider hardware acceleration (CUDA, Metal, etc.)
- Add appropriate caching for inference results
- Monitor and optimize memory usage

### 3. Error Handling

- Provide informative error messages for AI-specific failures
- Distinguish between model errors and infrastructure errors
- Implement graceful degradation when AI services are unavailable
- Log detailed diagnostic information for debugging

### 4. Security Considerations

- Validate all inputs to AI models
- Implement rate limiting for resource-intensive operations
- Consider privacy implications of data used for inference
- Apply appropriate access controls to sensitive models

## Common Patterns

### Agent-Based Tool Execution

```rust
// Example pattern for implementing agent-based tools
pub struct AgentToolExecutor {
    tools: HashMap<String, Box<dyn ToolHandler>>,
    agent_config: AgentConfig,
}

impl AgentToolExecutor {
    pub async fn execute_with_agent(&self, request: ToolRequest) -> Result<ToolResponse> {
        // Planning phase
        let plan = self.generate_plan(&request).await?;
        
        // Execution phase
        let mut results = Vec::new();
        for step in plan.steps {
            let step_result = self.execute_step(&step).await?;
            results.push(step_result);
        }
        
        // Synthesis phase
        self.synthesize_results(results).await
    }
}
```

### Model Registry Pattern

```rust
// Example pattern for managing multiple models
pub struct ModelRegistry {
    models: RwLock<HashMap<String, Arc<dyn Model>>>,
    loader_factory: Arc<dyn ModelLoaderFactory>,
}

impl ModelRegistry {
    pub async fn get_model(&self, model_id: &str) -> Result<Arc<dyn Model>> {
        // Try to get from cache first
        {
            let models = self.models.read().await;
            if let Some(model) = models.get(model_id) {
                return Ok(Arc::clone(model));
            }
        }
        
        // Load the model if not found
        let model = self.loader_factory.create_loader(model_id).load().await?;
        
        // Store in cache and return
        let model = Arc::new(model);
        {
            let mut models = self.models.write().await;
            models.insert(model_id.to_string(), Arc::clone(&model));
        }
        
        Ok(model)
    }
}
```

## Testing AI Components

### Unit Testing

Use mocks for external AI services:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;
    
    mock! {
        AIClient {}
        impl AIClient for AIClient {
            fn generate(&self, prompt: &str) -> Result<String>;
        }
    }
    
    #[test]
    fn test_ai_tool_handler() {
        let mut mock_client = MockAIClient::new();
        mock_client
            .expect_generate()
            .with(eq("test prompt"))
            .returning(|_| Ok("test response".to_string()));
            
        let handler = AIToolHandler::new(AIToolConfig::default(), Arc::new(mock_client));
        
        // Test implementation
    }
}
```

### Integration Testing

Test the full AI integration chain:

```python
# tests/py/test_ai_integration.py
import pytest
from squirrel.mcp import MCP
from squirrel.tools import AIToolRegistry

def test_ai_tool_execution():
    # Setup
    mcp = MCP()
    registry = AIToolRegistry(mcp)
    
    # Execute
    result = registry.execute_tool("text-generation", {"prompt": "Hello, world!"})
    
    # Verify
    assert result is not None
    assert isinstance(result, dict)
    assert "generated_text" in result
```

### Performance Testing

Benchmark AI operations:

```rust
#[bench]
fn bench_model_inference(b: &mut Bencher) {
    let model = setup_test_model();
    let input = create_test_input();
    
    b.iter(|| {
        model.predict(&input).unwrap()
    });
}
```

## Debugging AI Components

1. **Logging**: Use structured logging with the `tracing` crate:
   ```rust
   tracing::info!(model = %model_name, "Loading AI model");
   ```

2. **Instrumentation**: Add performance metrics:
   ```rust
   let timer = Instant::now();
   let result = model.predict(&input)?;
   metrics::histogram!("ai.inference.latency", timer.elapsed().as_millis() as f64);
   ```

3. **Visual Tools**: For complex AI workflows, generate visual debug outputs:
   ```rust
   if cfg!(debug_assertions) {
       visualize_attention_weights(&weights, "debug_attention.png")?;
   }
   ```

## Resources and References

### Internal Documentation

- [MCP Protocol Specification](../active/mcp-protocol/MCP_SPECIFICATION.md)
- [Capability Registry](../../capability_registry.toml)
- [CURRENT_STATUS.md](../../CURRENT_STATUS.md)

### External References

- [Rust AI Ecosystem Guide](https://github.com/vaaaaanquish/Awesome-Rust-MachineLearning)
- [Rust Tokenizers](https://github.com/huggingface/tokenizers)

## Support and Contact

See the [ecoPrimals](https://github.com/ecoPrimals) organization for contribution guidelines.

---

*Last revision: March 17, 2026.* 